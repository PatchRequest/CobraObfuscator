use anyhow::{bail, Context, Result};

use super::types::PeFile;
use crate::pipeline::{ObfuscatedFunction, ObfuscatedFunctionWithIR};

/// IMAGE_SCN_MEM_EXECUTE
const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
/// IMAGE_SCN_CNT_CODE
const IMAGE_SCN_CNT_CODE: u32 = 0x00000020;

/// Align a value up to the given alignment.
fn align_up(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

/// Layout for appending obfuscated code to the last section.
pub struct TextExpansionLayout {
    /// Index of the last section we'll extend.
    pub section_index: usize,
    /// RVA where the new code starts (after existing virtual data of last section).
    pub virtual_address: u32,
    /// File offset where the new code is appended.
    pub raw_offset: u32,
}

/// Calculate layout for appending obfuscated code to the last section.
///
/// No new section header is created. The last section is extended to hold
/// the obfuscated code, and its characteristics are updated to include execute.
pub fn calculate_text_expansion(pe: &PeFile) -> Result<TextExpansionLayout> {
    let last_idx = pe.sections.len() - 1;
    let last = &pe.sections[last_idx];

    let code_va = last.virtual_address + last.raw_size;
    let raw_offset = last.raw_offset + last.raw_size;

    Ok(TextExpansionLayout {
        section_index: last_idx,
        virtual_address: code_va,
        raw_offset,
    })
}

/// A code cave — a region in an existing section where we can place code.
#[derive(Debug, Clone)]
struct Cave {
    /// RVA of the cave start.
    rva: u32,
    /// Size in bytes.
    size: u32,
}

/// Where an obfuscated function was placed.
#[derive(Debug)]
pub struct Placement {
    /// Index into the obfuscated functions list.
    pub func_index: usize,
    /// RVA where the code is placed.
    pub target_rva: u32,
    /// Whether this is in a cave (true) or the extension area (false).
    pub in_cave: bool,
}

/// Find code caves in the .text section: inter-function padding (CC/90/00 runs).
fn find_inter_function_caves(pe: &PeFile) -> Vec<Cave> {
    let mut caves = Vec::new();

    // Find code sections
    for section in &pe.sections {
        if !section.is_code() {
            continue;
        }

        // Get all functions in this section, sorted by start RVA
        let mut section_funcs: Vec<(u32, u32)> = pe
            .functions
            .iter()
            .filter(|f| {
                f.start_rva >= section.virtual_address
                    && f.start_rva < section.virtual_address + section.raw_size
            })
            .map(|f| (f.start_rva, f.end_rva))
            .collect();
        section_funcs.sort();

        // Check gaps between consecutive functions
        for pair in section_funcs.windows(2) {
            let gap_start = pair[0].1;
            let gap_end = pair[1].0;
            if gap_end <= gap_start {
                continue;
            }
            let gap_size = gap_end - gap_start;

            // Verify it's padding bytes
            if let Some(file_off) = section.rva_to_offset(gap_start) {
                let end = file_off + gap_size as usize;
                if end <= pe.data.len() {
                    let gap_bytes = &pe.data[file_off..end];
                    let is_padding = gap_bytes
                        .iter()
                        .all(|&b| b == 0xCC || b == 0x00 || b == 0x90);
                    if is_padding && gap_size >= 5 {
                        caves.push(Cave {
                            rva: gap_start,
                            size: gap_size,
                        });
                    }
                }
            }
        }

        // Gap after last function to end of section raw data
        if let Some(&(_, last_end)) = section_funcs.last() {
            let section_end = section.virtual_address + section.raw_size;
            if section_end > last_end {
                let trail = section_end - last_end;
                if let Some(file_off) = section.rva_to_offset(last_end) {
                    let end = file_off + trail as usize;
                    if end <= pe.data.len() {
                        let gap_bytes = &pe.data[file_off..end];
                        let is_padding = gap_bytes
                            .iter()
                            .all(|&b| b == 0xCC || b == 0x00 || b == 0x90);
                        if is_padding && trail >= 5 {
                            caves.push(Cave {
                                rva: last_end,
                                size: trail,
                            });
                        }
                    }
                }
            }
        }
    }

    caves
}

/// Allocate obfuscated functions into caves and overflow area.
///
/// Returns placements for each function. Functions are placed using best-fit:
/// - First into original function bodies of OTHER functions (biggest caves)
/// - Then into inter-function padding gaps
/// - Finally into the extension area (appended after last section)
fn allocate_scattered(
    funcs: &[ObfuscatedFunctionWithIR],
    pe: &PeFile,
    extension_rva: u32,
) -> Vec<Placement> {
    // Build cave list from original function bodies (excluding each function's own body)
    // Each original function body becomes a cave after we read its code.
    // The first 5 bytes are reserved for the trampoline JMP, so cave = body[5..].
    let mut caves: Vec<Cave> = Vec::new();

    // Add original function bodies as caves (minus 5 bytes for trampoline).
    // Only safe when no RIP-relative references from outside point into bodies.
    // MSVC binaries embed security cookies and other data referenced by
    // RIP-relative addressing near function bodies, so we check for this.
    let has_security_cookie = pe.sections.iter().any(|s| {
        s.name == "_RDATA" || s.name == ".gfids" || s.name == ".fptable"
    });

    if !has_security_cookie {
        // Safe to use function bodies as caves (GCC, Clang, Rust binaries)
        let mut seen_rvas = std::collections::HashSet::new();
        for func in funcs {
            if func.original_size > 5 {
                let cave_rva = func.original_rva + 5;
                if seen_rvas.insert(cave_rva) {
                    caves.push(Cave {
                        rva: cave_rva,
                        size: func.original_size - 5,
                    });
                }
            }
        }
    } else {
        log::info!("MSVC binary detected — skipping function body caves for safety");
    }

    // Add inter-function padding caves
    caves.extend(find_inter_function_caves(pe));

    // Sort caves by size descending for efficient allocation
    caves.sort_by(|a, b| b.size.cmp(&a.size));

    // Sort functions by obfuscated size descending (largest first = best-fit packing)
    let mut func_indices: Vec<usize> = (0..funcs.len()).collect();
    func_indices.sort_by(|&a, &b| funcs[b].code.len().cmp(&funcs[a].code.len()));

    let mut placements: Vec<Option<Placement>> = (0..funcs.len()).map(|_| None).collect();
    let mut cave_used = vec![false; caves.len()]; // track which caves are taken
    let mut extension_offset: u32 = 0;

    for &fi in &func_indices {
        let need = funcs[fi].code.len() as u32;

        // A function cannot be placed in its OWN original body cave
        // (that's where the trampoline goes). Also prevent placement in
        // caves of functions that share the same original_rva (ICF).
        let own_cave_rva = funcs[fi].original_rva + 5;

        let mut best_cave: Option<usize> = None;
        for (ci, cave) in caves.iter().enumerate() {
            if cave_used[ci] {
                continue;
            }
            if cave.rva == own_cave_rva {
                // Skip own body
                continue;
            }
            if cave.size >= need {
                // Best-fit: smallest cave that fits
                match best_cave {
                    None => best_cave = Some(ci),
                    Some(prev) => {
                        if cave.size < caves[prev].size {
                            best_cave = Some(ci);
                        }
                    }
                }
            }
        }

        if let Some(ci) = best_cave {
            cave_used[ci] = true;
            placements[fi] = Some(Placement {
                func_index: fi,
                target_rva: caves[ci].rva,
                in_cave: true,
            });
        } else {
            // Overflow to extension
            placements[fi] = Some(Placement {
                func_index: fi,
                target_rva: extension_rva + extension_offset,
                in_cave: false,
            });
            extension_offset += need;
        }
    }

    placements.into_iter().map(|p| p.unwrap()).collect()
}

/// Find the capacity of the cave at the given RVA.
///
/// Checks if it's a function body cave (original_rva + 5) or an inter-function gap.
fn find_cave_capacity_for_rva(
    rva: u32,
    funcs: &[ObfuscatedFunctionWithIR],
    pe: &PeFile,
) -> u32 {
    // Check function body caves
    for f in funcs {
        if rva == f.original_rva + 5 {
            return f.original_size - 5;
        }
    }

    // Check inter-function gaps — find the next function or section boundary after rva
    let mut next_boundary = u32::MAX;
    for f in &pe.functions {
        if f.start_rva > rva && f.start_rva < next_boundary {
            next_boundary = f.start_rva;
        }
    }
    for s in &pe.sections {
        let section_end = s.virtual_address + s.raw_size;
        if section_end > rva && section_end < next_boundary {
            next_boundary = section_end;
        }
    }

    if next_boundary == u32::MAX {
        0
    } else {
        next_boundary - rva
    }
}

/// Write the obfuscated PE with code scattered throughout .text caves.
///
/// Obfuscated functions are placed into caves (original function bodies,
/// inter-function padding) when possible, with overflow appended to the
/// last section. Trampolines at original locations jump to scattered code.
pub fn write_pe_scattered(
    pe: &PeFile,
    funcs: &mut [ObfuscatedFunctionWithIR],
    layout: &TextExpansionLayout,
) -> Result<Vec<u8>> {
    if funcs.is_empty() {
        bail!("No functions were obfuscated");
    }

    let last = &pe.sections[layout.section_index];

    // Phase 1: Allocate placements
    let mut placements = allocate_scattered(funcs, pe, layout.virtual_address);

    // Phase 2: Re-encode each function at its actual placement VA.
    // Re-encoding can change code size (branch encoding differs by VA).
    // If a cave-placed function grows too large, move it to extension.
    let image_base = pe.image_base;

    // First pass: re-encode cave placements, evict if they grew
    let mut evicted: Vec<usize> = Vec::new();
    for placement in placements.iter() {
        let fi = placement.func_index;
        let target_va = image_base + placement.target_rva as u64;
        let new_code = crate::pipeline::reencode_at_va(&funcs[fi].ir, target_va)
            .with_context(|| format!("Re-encode failed for {}", funcs[fi].name))?;
        funcs[fi].code = new_code;

        if placement.in_cave {
            // Check if re-encoded code still fits in the cave
            // Cave size = original_size - 5 for function body caves
            // We need to check against the cave we allocated into.
            // Since we used best-fit, let's check against the original function
            // body that this cave came from. We'll just check the file bounds.
            let cave_capacity = find_cave_capacity_for_rva(placement.target_rva, funcs, pe);
            if funcs[fi].code.len() as u32 > cave_capacity {
                log::debug!(
                    "  {} re-encoded to {} bytes, exceeds cave {} — evicting to extension",
                    funcs[fi].name,
                    funcs[fi].code.len(),
                    cave_capacity
                );
                evicted.push(fi);
            }
        }
    }

    // Re-assign evicted functions to extension
    if !evicted.is_empty() {
        // Calculate current extension usage
        let mut ext_offset: u32 = 0;
        for p in placements.iter() {
            if !p.in_cave {
                ext_offset += funcs[p.func_index].code.len() as u32;
            }
        }

        for &fi in &evicted {
            let p = placements.iter_mut().find(|p| p.func_index == fi).unwrap();
            p.target_rva = layout.virtual_address + ext_offset;
            p.in_cave = false;
            ext_offset += funcs[fi].code.len() as u32;
        }

        // Re-encode evicted functions at their new extension VA
        for &fi in &evicted {
            let p = placements.iter().find(|p| p.func_index == fi).unwrap();
            let target_va = image_base + p.target_rva as u64;
            let new_code = crate::pipeline::reencode_at_va(&funcs[fi].ir, target_va)
                .with_context(|| format!("Re-encode failed for {}", funcs[fi].name))?;
            funcs[fi].code = new_code;
        }
    }

    // Phase 3: Lay out extension functions and iterate until sizes converge.
    // Encoding at a specific VA can change code size (branch encoding differs),
    // so we iterate: assign offsets → encode → check sizes → repeat.
    let mut ext_indices: Vec<usize> = placements
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.in_cave)
        .map(|(i, _)| i)
        .collect();
    ext_indices.sort_by_key(|&i| placements[i].target_rva);

    for _round in 0..3 {
        // Assign sequential offsets based on current sizes
        let mut ext_offset: u32 = 0;
        for &pi in &ext_indices {
            let fi = placements[pi].func_index;
            placements[pi].target_rva = layout.virtual_address + ext_offset;
            ext_offset += funcs[fi].code.len() as u32;
        }

        // Re-encode at assigned VAs
        let mut changed = false;
        for &pi in &ext_indices {
            let fi = placements[pi].func_index;
            let target_va = image_base + placements[pi].target_rva as u64;
            let new_code = crate::pipeline::reencode_at_va(&funcs[fi].ir, target_va)
                .with_context(|| format!("Re-encode failed for {}", funcs[fi].name))?;
            if new_code.len() != funcs[fi].code.len() {
                changed = true;
            }
            funcs[fi].code = new_code;
        }

        if !changed {
            break;
        }
    }

    // Final extension size from converged sizes
    let mut extension_size: u32 = 0;
    for &pi in &ext_indices {
        extension_size += funcs[placements[pi].func_index].code.len() as u32;
    }

    let in_cave = placements.iter().filter(|p| p.in_cave).count();
    let in_ext = ext_indices.len();
    log::info!(
        "Scatter allocation: {} in caves, {} in extension (evicted {})",
        in_cave,
        in_ext,
        evicted.len()
    );

    let new_raw_size = if extension_size > 0 {
        align_up(last.raw_size + extension_size, pe.file_alignment)
    } else {
        last.raw_size
    };
    let new_virtual_size = std::cmp::max(last.virtual_size, new_raw_size);

    // Phase 4: Build output
    let append_size = (new_raw_size - last.raw_size) as usize;
    let append_at = layout.raw_offset as usize;

    let mut output = if append_size > 0 {
        let mut out = Vec::with_capacity(pe.data.len() + append_size);
        out.extend_from_slice(&pe.data[..append_at]);
        out.resize(append_at + append_size, 0xCC);
        if append_at < pe.data.len() {
            out.extend_from_slice(&pe.data[append_at..]);
        }
        out
    } else {
        pe.data.clone()
    };

    // Phase 5: Write scattered code into caves and extension
    for placement in &placements {
        let fi = placement.func_index;
        let code = &funcs[fi].code;

        let file_offset = if placement.in_cave {
            // Cave is in an existing section
            pe.sections
                .iter()
                .find_map(|s| s.rva_to_offset(placement.target_rva))
                .with_context(|| {
                    format!(
                        "Cannot find file offset for cave RVA 0x{:x} ({})",
                        placement.target_rva, funcs[fi].name
                    )
                })?
        } else {
            // Extension area: offset relative to extension start
            let ext_offset = placement.target_rva - layout.virtual_address;
            append_at + ext_offset as usize
        };

        output[file_offset..file_offset + code.len()].copy_from_slice(code);
    }

    // Phase 6: Write trampolines at original function locations
    for placement in &placements {
        let fi = placement.func_index;
        let original_rva = funcs[fi].original_rva;
        let original_size = funcs[fi].original_size;
        let target_rva = placement.target_rva;

        let file_offset = pe
            .sections
            .iter()
            .find_map(|s| s.rva_to_offset(original_rva))
            .with_context(|| {
                format!(
                    "Cannot find file offset for function at RVA 0x{:x}",
                    original_rva
                )
            })?;

        // Write JMP rel32 trampoline
        let jmp_source_rva = original_rva + 5;
        let rel32 = (target_rva as i64 - jmp_source_rva as i64) as i32;

        output[file_offset] = 0xE9;
        output[file_offset + 1..file_offset + 5].copy_from_slice(&rel32.to_le_bytes());

        // Fill remaining original body with CC (unless it's a cave being used)
        // Only fill bytes 5..original_size that aren't occupied by scattered code
        let body_start = file_offset + 5;
        let body_end = file_offset + original_size as usize;

        // Check if another function was placed in this body
        let body_cave_rva = original_rva + 5;
        let occupant = placements.iter().find(|p| p.in_cave && p.target_rva == body_cave_rva);

        if let Some(occ) = occupant {
            // Another function occupies this cave. Fill only the unused tail.
            let occ_size = funcs[occ.func_index].code.len();
            let used_end = body_start + occ_size;
            for i in used_end..body_end {
                output[i] = 0xCC;
            }
        } else {
            // No occupant — fill entire body after trampoline
            for i in body_start..body_end {
                output[i] = 0xCC;
            }
        }
    }

    // Phase 7: Update headers
    if append_size > 0 {
        let sh = last.header_offset;
        output[sh + 8..sh + 12].copy_from_slice(&new_virtual_size.to_le_bytes());
        output[sh + 16..sh + 20].copy_from_slice(&new_raw_size.to_le_bytes());
        let new_chars = last.characteristics | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_CNT_CODE;
        output[sh + 36..sh + 40].copy_from_slice(&new_chars.to_le_bytes());

        let new_size_of_image = align_up(
            last.virtual_address + new_virtual_size,
            pe.section_alignment,
        );
        output[pe.size_of_image_offset..pe.size_of_image_offset + 4]
            .copy_from_slice(&new_size_of_image.to_le_bytes());

        let old_size_of_code = u32::from_le_bytes(
            output[pe.size_of_code_offset..pe.size_of_code_offset + 4]
                .try_into()
                .unwrap(),
        );
        let new_size_of_code = old_size_of_code + (new_raw_size - last.raw_size);
        output[pe.size_of_code_offset..pe.size_of_code_offset + 4]
            .copy_from_slice(&new_size_of_code.to_le_bytes());
    }

    // Phase 8: Zero checksum
    output[pe.checksum_offset..pe.checksum_offset + 4].copy_from_slice(&0u32.to_le_bytes());

    log::warn!("PE checksum zeroed — Authenticode signature (if present) is invalidated");
    log::info!(
        "Scattered {} functions: {} in .text caves, {} in extension (+{} bytes)",
        funcs.len(),
        in_cave,
        in_ext,
        append_size
    );

    Ok(output)
}

/// Write the obfuscated PE by extending the last section.
///
/// Obfuscated code is appended after the last section's raw data. The last
/// section header is updated to cover the new code with execute permissions.
/// No new section is added — from the outside, it just looks like the last
/// section is larger.
pub fn write_pe(
    pe: &PeFile,
    obfuscated: &[ObfuscatedFunction],
    layout: &TextExpansionLayout,
) -> Result<Vec<u8>> {
    if obfuscated.is_empty() {
        bail!("No functions were obfuscated");
    }

    let last = &pe.sections[layout.section_index];

    // Concatenate all obfuscated code
    let mut code_data = Vec::new();
    let mut function_offsets: Vec<(u32, u32, u32)> = Vec::new();

    for func in obfuscated {
        let offset = code_data.len() as u32;
        function_offsets.push((func.original_rva, func.original_size, offset));
        code_data.extend_from_slice(&func.code);
    }

    let code_size = code_data.len() as u32;

    // New raw size = old raw size + code, file-aligned
    let new_raw_size = align_up(last.raw_size + code_size, pe.file_alignment);
    // New virtual size must cover at least the new raw size
    let new_virtual_size = std::cmp::max(last.virtual_size, new_raw_size);

    // Build output: append code right after last section's raw data
    let append_at = layout.raw_offset as usize;
    let append_size = (new_raw_size - last.raw_size) as usize;

    let mut output = Vec::with_capacity(pe.data.len() + append_size);
    output.extend_from_slice(&pe.data[..append_at]);
    code_data.resize(append_size, 0xCC); // pad to file alignment
    output.extend_from_slice(&code_data);
    if append_at < pe.data.len() {
        output.extend_from_slice(&pe.data[append_at..]);
    }

    // 1. Update last section header
    let sh = last.header_offset;

    // Expand virtual size
    output[sh + 8..sh + 12].copy_from_slice(&new_virtual_size.to_le_bytes());

    // Expand raw size
    output[sh + 16..sh + 20].copy_from_slice(&new_raw_size.to_le_bytes());

    // Add execute + code characteristics
    let new_chars = last.characteristics | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_CNT_CODE;
    output[sh + 36..sh + 40].copy_from_slice(&new_chars.to_le_bytes());

    // 2. Update SizeOfImage
    let new_size_of_image = align_up(
        last.virtual_address + new_virtual_size,
        pe.section_alignment,
    );
    output[pe.size_of_image_offset..pe.size_of_image_offset + 4]
        .copy_from_slice(&new_size_of_image.to_le_bytes());

    // 3. Update SizeOfCode
    let old_size_of_code =
        u32::from_le_bytes(output[pe.size_of_code_offset..pe.size_of_code_offset + 4].try_into().unwrap());
    let new_size_of_code = old_size_of_code + (new_raw_size - last.raw_size);
    output[pe.size_of_code_offset..pe.size_of_code_offset + 4]
        .copy_from_slice(&new_size_of_code.to_le_bytes());

    // 4. Zero checksum
    output[pe.checksum_offset..pe.checksum_offset + 4].copy_from_slice(&0u32.to_le_bytes());

    // 5. Write trampolines over original function bodies
    for &(original_rva, original_size, offset_in_code) in &function_offsets {
        // Code is at layout.virtual_address + offset
        let target_rva = layout.virtual_address + offset_in_code;

        let file_offset = pe
            .sections
            .iter()
            .find_map(|s| s.rva_to_offset(original_rva))
            .with_context(|| {
                format!(
                    "Cannot find file offset for function at RVA 0x{:x}",
                    original_rva
                )
            })?;

        let jmp_source_rva = original_rva + 5;
        let rel32 = (target_rva as i64 - jmp_source_rva as i64) as i32;

        output[file_offset] = 0xE9;
        output[file_offset + 1..file_offset + 5].copy_from_slice(&rel32.to_le_bytes());

        let remaining = original_size as usize - 5;
        for i in 0..remaining {
            output[file_offset + 5 + i] = 0xCC;
        }
    }

    log::warn!("PE checksum zeroed — Authenticode signature (if present) is invalidated");

    log::info!(
        "Extended {} section: +{} bytes ({} functions trampolined)",
        last.name,
        code_data.len(),
        function_offsets.len()
    );

    Ok(output)
}

/// Write obfuscated code back in-place at original function addresses.
///
/// No new section is added. Functions that grew too large are skipped.
/// This mode preserves PC-to-metadata mappings (e.g. Go's .gopclntab).
pub fn write_pe_inplace(
    pe: &PeFile,
    obfuscated: &[ObfuscatedFunction],
) -> Result<Vec<u8>> {
    let mut output = pe.data.clone();
    let mut patched_count = 0;

    for func in obfuscated {
        if func.code.len() > func.original_size as usize {
            log::warn!(
                "In-place: skipping {} — obfuscated size {} > original {}",
                func.name, func.code.len(), func.original_size
            );
            continue;
        }

        let file_offset = pe.sections.iter()
            .find_map(|s| s.rva_to_offset(func.original_rva))
            .with_context(|| format!("Cannot find file offset for RVA 0x{:x}", func.original_rva))?;

        output[file_offset..file_offset + func.code.len()]
            .copy_from_slice(&func.code);

        let remaining = func.original_size as usize - func.code.len();
        for i in 0..remaining {
            output[file_offset + func.code.len() + i] = 0xCC;
        }

        patched_count += 1;
    }

    if patched_count == 0 {
        bail!("No functions could be obfuscated in-place (all grew too large)");
    }

    output[pe.checksum_offset..pe.checksum_offset + 4].copy_from_slice(&0u32.to_le_bytes());

    log::info!("In-place: patched {} functions (skipped {})",
        patched_count, obfuscated.len() - patched_count);

    Ok(output)
}
