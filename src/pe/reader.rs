use anyhow::{bail, Context, Result};
use object::read::pe::PeFile64;
use object::Object;

use super::pdata;
use super::types::{PeFile, PeFunction, PeSectionInfo};

/// Detect if a PE file is a Go binary (has Go build ID or go.buildid marker).
pub fn is_go_binary(data: &[u8]) -> bool {
    // Go embeds "Go build ID:" or "go.buildid" in every binary
    data.windows(12).any(|w| w == b"Go build ID:" || w == b"go.buildid\x00\x00")
}

/// Parse a PE file from raw bytes.
pub fn read_pe(data: &[u8]) -> Result<PeFile> {
    // Validate MZ header
    if data.len() < 2 || data[0] != b'M' || data[1] != b'Z' {
        bail!("Not a valid PE file (missing MZ header)");
    }

    let pe = PeFile64::parse(data).context("Failed to parse PE64")?;
    let image_base = pe.relative_address_base();

    // Parse the PE header offsets manually for patching
    let pe_offset = u32::from_le_bytes(data[0x3C..0x40].try_into().unwrap()) as usize;

    // COFF header starts at pe_offset + 4 (after "PE\0\0" signature)
    let coff_header_offset = pe_offset + 4;
    let number_of_sections =
        u16::from_le_bytes(data[coff_header_offset + 2..coff_header_offset + 4].try_into().unwrap());
    let size_of_optional_header =
        u16::from_le_bytes(data[coff_header_offset + 16..coff_header_offset + 18].try_into().unwrap());

    let optional_header_offset = coff_header_offset + 20;

    // PE32+ optional header offsets (relative to optional header start)
    let size_of_code_offset = optional_header_offset + 4;
    let size_of_image_offset = optional_header_offset + 56;
    let checksum_offset = optional_header_offset + 64;
    let size_of_headers = u32::from_le_bytes(
        data[optional_header_offset + 60..optional_header_offset + 64]
            .try_into()
            .unwrap(),
    );

    // Entry point RVA
    let entry_point_rva = u32::from_le_bytes(
        data[optional_header_offset + 16..optional_header_offset + 20]
            .try_into()
            .unwrap(),
    );

    // Section alignment and file alignment
    let section_alignment = u32::from_le_bytes(
        data[optional_header_offset + 32..optional_header_offset + 36]
            .try_into()
            .unwrap(),
    );
    let file_alignment = u32::from_le_bytes(
        data[optional_header_offset + 36..optional_header_offset + 40]
            .try_into()
            .unwrap(),
    );

    // Parse section headers
    let section_table_offset = optional_header_offset + size_of_optional_header as usize;
    let mut sections = Vec::new();

    for i in 0..number_of_sections as usize {
        let sh_offset = section_table_offset + i * 40;
        if sh_offset + 40 > data.len() {
            bail!("Section header {} extends past end of file", i);
        }

        let name_bytes = &data[sh_offset..sh_offset + 8];
        let name = std::str::from_utf8(name_bytes)
            .unwrap_or("")
            .trim_end_matches('\0')
            .to_string();

        let virtual_size = u32::from_le_bytes(data[sh_offset + 8..sh_offset + 12].try_into().unwrap());
        let virtual_address =
            u32::from_le_bytes(data[sh_offset + 12..sh_offset + 16].try_into().unwrap());
        let raw_size = u32::from_le_bytes(data[sh_offset + 16..sh_offset + 20].try_into().unwrap());
        let raw_offset = u32::from_le_bytes(data[sh_offset + 20..sh_offset + 24].try_into().unwrap());
        let characteristics =
            u32::from_le_bytes(data[sh_offset + 36..sh_offset + 40].try_into().unwrap());

        sections.push(PeSectionInfo {
            name,
            virtual_address,
            virtual_size,
            raw_offset,
            raw_size,
            characteristics,
            header_offset: sh_offset,
        });
    }

    // Find .pdata section
    let pdata_section = sections.iter().find(|s| s.name == ".pdata");
    let pdata_section = match pdata_section {
        Some(s) => s,
        None => bail!("No .pdata section found — cannot discover functions without it"),
    };

    // Use virtual_size for actual data length (raw_size is file-aligned and may be larger)
    let pdata_actual_size = pdata_section.virtual_size.min(pdata_section.raw_size);
    let pdata_data = &data[pdata_section.raw_offset as usize
        ..(pdata_section.raw_offset + pdata_actual_size) as usize];
    let runtime_functions = pdata::parse_pdata(pdata_data).context("Failed to parse .pdata")?;

    log::info!("Found {} RUNTIME_FUNCTION entries in .pdata", runtime_functions.len());

    // Find .text section for extracting code
    let text_section = sections
        .iter()
        .find(|s| s.name == ".text")
        .context("No .text section found")?;

    // Build export name map from the object crate
    let mut export_names: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    for export in pe.exports().unwrap_or_default() {
        let name_bytes = export.name();
        let address = export.address();
        if !name_bytes.is_empty() {
            let rva = (address - image_base) as u32;
            if let Ok(name) = std::str::from_utf8(name_bytes) {
                export_names.insert(rva, name.to_string());
            }
        }
    }

    // Build PeFunction list
    let mut functions = Vec::new();
    for (i, rf) in runtime_functions.iter().enumerate() {
        let func_size = rf.end_address - rf.begin_address;

        // Extract code bytes
        let code = match text_section.rva_to_offset(rf.begin_address) {
            Some(offset) => {
                let end = offset + func_size as usize;
                if end <= data.len() {
                    data[offset..end].to_vec()
                } else {
                    log::warn!("Function at RVA 0x{:x} extends past file end, skipping", rf.begin_address);
                    continue;
                }
            }
            None => {
                // Function might be in another code section
                let mut found = None;
                for sec in &sections {
                    if sec.is_code() {
                        if let Some(offset) = sec.rva_to_offset(rf.begin_address) {
                            let end = offset + func_size as usize;
                            if end <= data.len() {
                                found = Some(data[offset..end].to_vec());
                                break;
                            }
                        }
                    }
                }
                match found {
                    Some(code) => code,
                    None => {
                        log::warn!(
                            "Cannot find code for function at RVA 0x{:x}, skipping",
                            rf.begin_address
                        );
                        continue;
                    }
                }
            }
        };

        let name = export_names
            .get(&rf.begin_address)
            .cloned()
            .unwrap_or_else(|| format!("sub_{:x}", rf.begin_address));

        functions.push(PeFunction {
            name,
            start_rva: rf.begin_address,
            end_rva: rf.end_address,
            code,
            pdata_index: i,
            is_runtime: false, // will be set below
        });
    }

    // Identify user functions by tracing the call graph from main().
    // Everything NOT reachable from main is marked as runtime/CRT.
    let user_rvas = identify_user_functions(entry_point_rva, &functions);
    for func in &mut functions {
        if !user_rvas.contains(&func.start_rva) {
            func.is_runtime = true;
        }
    }

    let user_count = functions.iter().filter(|f| !f.is_runtime).count();
    log::info!(
        "Discovered {} functions ({} user, {} CRT/runtime, {} from exports)",
        functions.len(),
        user_count,
        functions.len() - user_count,
        export_names.len()
    );

    Ok(PeFile {
        data: data.to_vec(),
        image_base,
        section_alignment,
        file_alignment,
        sections,
        functions,
        number_of_sections_offset: coff_header_offset + 2,
        size_of_image_offset,
        size_of_code_offset,
        checksum_offset,
        number_of_sections,
        size_of_headers,
        entry_point_rva,
    })
}

/// Identify user functions by tracing the call graph from main().
///
/// Strategy:
/// 1. Find entry point → __tmainCRTStartup → main (first E8 call chain)
/// 2. From main, recursively follow all E8 (call rel32) targets
/// 3. Only functions reachable from main are "user" functions
/// 4. Everything else is CRT/runtime
fn identify_user_functions(
    entry_rva: u32,
    functions: &[PeFunction],
) -> std::collections::HashSet<u32> {
    let mut user_rvas = std::collections::HashSet::new();

    // Build a map from RVA to function for quick lookup
    let func_by_rva: std::collections::HashMap<u32, &PeFunction> =
        functions.iter().map(|f| (f.start_rva, f)).collect();
    let func_rvas: std::collections::HashSet<u32> = functions.iter().map(|f| f.start_rva).collect();

    // Try the MSVC/MinGW heuristic: entry → CRT startup → main candidates.
    // If the entry point isn't in .pdata (Go, etc.), skip straight to fallback.
    let mut main_rva: Option<u32> = None;
    let mut candidates: Vec<u32> = Vec::new();
    let mut call_targets: Vec<u32> = Vec::new();
    let mut crt_startup_rva: Option<u32> = None;

    if let Some(entry_func) = func_by_rva.get(&entry_rva) {
        log::info!("CRT: entry point {} at RVA 0x{:x}", entry_func.name, entry_rva);

        // Find __tmainCRTStartup (first E8 call from entry point)
        if let Some(startup_rva) = find_first_e8_target(&entry_func.code, entry_rva) {
            if func_rvas.contains(&startup_rva) {
                crt_startup_rva = Some(startup_rva);
                if let Some(crt_startup) = func_by_rva.get(&startup_rva) {
                    log::info!("CRT: startup {} at RVA 0x{:x}", crt_startup.name, startup_rva);

                    // Find main: among all E8 call targets from CRT startup, main is the one
                    // that reaches the most local functions via BFS.
                    call_targets = find_all_e8_targets(&crt_startup.code, startup_rva);

                    if call_targets.is_empty() {
                        // MSVC fallback: __scrt_common_main_seh is split across multiple
                        // .pdata entries (SEH chaining). Extend the scan.
                        let startup_end = startup_rva + crt_startup.code.len() as u32;
                        let mut extended_code = crt_startup.code.clone();
                        let mut current_end = startup_end;
                        const MAX_EXTENSION: usize = 1024;

                        let mut sorted_funcs: Vec<&PeFunction> = functions.iter().collect();
                        sorted_funcs.sort_by_key(|f| f.start_rva);

                        for func in &sorted_funcs {
                            if extended_code.len() >= MAX_EXTENSION {
                                break;
                            }
                            if func.start_rva >= current_end && func.start_rva <= current_end + 64 {
                                let gap = (func.start_rva - current_end) as usize;
                                extended_code.extend(std::iter::repeat(0xCC).take(gap));
                                extended_code.extend_from_slice(&func.code);
                                current_end = func.start_rva + func.code.len() as u32;
                            } else if func.start_rva > current_end + 64 {
                                break;
                            }
                        }

                        if extended_code.len() > crt_startup.code.len() {
                            log::info!(
                                "Extended CRT startup scan: {} -> {} bytes (MSVC SEH chain)",
                                crt_startup.code.len(),
                                extended_code.len()
                            );
                            call_targets = find_all_e8_targets(&extended_code, startup_rva);
                        }
                    }

                    candidates = call_targets
                        .iter()
                        .filter(|t| func_rvas.contains(t) && **t != entry_rva && **t != startup_rva)
                        .copied()
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect();

                    let mut best_reach = 0;
                    for candidate in &candidates {
                        let reach = bfs_reachable_count(*candidate, &func_by_rva, &func_rvas);
                        log::debug!("  candidate 0x{:x}: reaches {} local functions", candidate, reach);
                        if reach > best_reach {
                            best_reach = reach;
                            main_rva = Some(*candidate);
                        }
                    }

                    if let Some(rva) = main_rva {
                        log::info!("Detected main at RVA 0x{:x} (reaches {} functions)", rva, best_reach);
                    }
                }
            }
        }
    } else {
        log::info!(
            "Entry point 0x{:x} not in .pdata — using global reachability fallback",
            entry_rva
        );
    }

    // Fallback for non-MSVC toolchains (Rust, Go, etc.):
    // Scan ALL functions and pick the one with the highest BFS reachability.
    if main_rva.is_none() {
        log::info!("MSVC main() heuristic failed — trying global reachability fallback");
        let mut global_best_rva = None;
        let mut global_best_reach = 0;
        for func in functions {
            if func.start_rva == entry_rva || Some(func.start_rva) == crt_startup_rva {
                continue;
            }
            let reach = bfs_reachable_count(func.start_rva, &func_by_rva, &func_rvas);
            if reach > global_best_reach {
                global_best_reach = reach;
                global_best_rva = Some(func.start_rva);
            }
        }
        match global_best_rva {
            Some(rva) if global_best_reach >= 3 => {
                log::info!(
                    "Fallback: detected main at RVA 0x{:x} (reaches {} functions)",
                    rva, global_best_reach
                );
                main_rva = Some(rva);
            }
            _ => {
                log::warn!("Could not identify main() — marking all as runtime");
                return user_rvas;
            }
        }
    }

    let main_rva = main_rva.unwrap();

    // Build the set of CRT startup's direct call targets (excluding main).
    // In MinGW, main() calls __main() which is a CRT function that initializes
    // global constructors. We must NOT follow calls into CRT startup targets
    // during BFS from main, otherwise we'd trampoline CRT functions and break them.
    // Only apply this exclusion when the MSVC heuristic succeeded.
    let crt_targets: std::collections::HashSet<u32> = if !candidates.is_empty() {
        call_targets
            .iter()
            .filter(|t| func_rvas.contains(t) && **t != main_rva)
            .copied()
            .collect()
    } else {
        // Fallback path — only exclude entry point and CRT startup
        let mut s = std::collections::HashSet::new();
        s.insert(entry_rva);
        if let Some(rva) = crt_startup_rva {
            s.insert(rva);
        }
        s
    };

    log::debug!("CRT startup direct targets (excluded from user BFS): {:?}",
        crt_targets.iter().map(|r| format!("0x{:x}", r)).collect::<Vec<_>>());

    // BFS from main to find all reachable user functions,
    // but do NOT follow calls to CRT startup's direct targets.
    let mut queue: std::collections::VecDeque<(u32, u32)> = std::collections::VecDeque::new();
    queue.push_back((main_rva, 0));
    user_rvas.insert(main_rva);

    // Limit BFS depth for safety: deep callees are likely library internals.
    // Use a higher depth for fallback detection (Rust/Go have deeper call chains).
    let max_bfs_depth: u32 = if candidates.is_empty() { 6 } else { 3 };

    while let Some((rva, depth)) = queue.pop_front() {
        if depth >= max_bfs_depth {
            continue;
        }
        if let Some(func) = func_by_rva.get(&rva) {
            let targets = find_all_e8_targets(&func.code, func.start_rva);
            for target in targets {
                if func_rvas.contains(&target)
                    && !user_rvas.contains(&target)
                    && !crt_targets.contains(&target)
                {
                    user_rvas.insert(target);
                    queue.push_back((target, depth + 1));
                }
            }
        }
    }

    log::info!(
        "Identified {} user functions reachable from main (out of {})",
        user_rvas.len(),
        functions.len()
    );
    user_rvas
}

/// Count how many local functions are reachable from a given start via BFS on E8 calls.
fn bfs_reachable_count(
    start_rva: u32,
    func_by_rva: &std::collections::HashMap<u32, &PeFunction>,
    func_rvas: &std::collections::HashSet<u32>,
) -> usize {
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start_rva);
    visited.insert(start_rva);

    while let Some(rva) = queue.pop_front() {
        if let Some(func) = func_by_rva.get(&rva) {
            let targets = find_all_e8_targets(&func.code, func.start_rva);
            for target in targets {
                if func_rvas.contains(&target) && !visited.contains(&target) {
                    visited.insert(target);
                    queue.push_back(target);
                }
            }
        }
    }
    visited.len()
}

/// Find the first E8 (call rel32) target in a function's bytes.
fn find_first_e8_target(code: &[u8], func_rva: u32) -> Option<u32> {
    // Use iced-x86 decoder to avoid false positives from data bytes
    let mut decoder = iced_x86::Decoder::with_ip(
        64,
        code,
        func_rva as u64,
        iced_x86::DecoderOptions::NONE,
    );
    while decoder.can_decode() {
        let insn = decoder.decode();
        if insn.mnemonic() == iced_x86::Mnemonic::Call
            && insn.op0_kind() == iced_x86::OpKind::NearBranch64
        {
            return Some(insn.near_branch64() as u32);
        }
    }
    None
}

/// Find all E8 (call rel32) targets in a function's bytes.
fn find_all_e8_targets(code: &[u8], func_rva: u32) -> Vec<u32> {
    let mut targets = Vec::new();
    let mut decoder = iced_x86::Decoder::with_ip(
        64,
        code,
        func_rva as u64,
        iced_x86::DecoderOptions::NONE,
    );
    while decoder.can_decode() {
        let insn = decoder.decode();
        if insn.mnemonic() == iced_x86::Mnemonic::Call
            && insn.op0_kind() == iced_x86::OpKind::NearBranch64
        {
            targets.push(insn.near_branch64() as u32);
        }
    }
    targets
}
