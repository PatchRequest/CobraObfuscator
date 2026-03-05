use anyhow::{bail, Context, Result};

use super::types::PeFile;
use crate::pipeline::ObfuscatedFunction;

/// Section header size.
const SECTION_HEADER_SIZE: usize = 40;

/// IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ
const CODE_SECTION_CHARACTERISTICS: u32 = 0x60000020;

/// Align a value up to the given alignment.
fn align_up(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

/// Layout for the new code section appended at end of file.
pub struct TextExpansionLayout {
    pub header_offset: usize,
    pub virtual_address: u32,
    pub raw_offset: u32,
}

/// Calculate where to place the new code section.
///
/// Appends after all existing sections. Uses a normal-looking section name
/// (`.text1` for MinGW/GCC binaries, `.textbss` for MSVC-style).
pub fn calculate_text_expansion(pe: &PeFile) -> Result<TextExpansionLayout> {
    let last_section = pe
        .sections
        .last()
        .context("PE has no sections")?;

    let next_va = align_up(
        last_section.virtual_address + last_section.virtual_size,
        pe.section_alignment,
    );

    let next_raw = align_up(
        last_section.raw_offset + last_section.raw_size,
        pe.file_alignment,
    );

    // Check if there's room for a new section header
    let last_header_end = last_section.header_offset + SECTION_HEADER_SIZE;
    let new_header_offset = last_header_end;
    let new_header_end = new_header_offset + SECTION_HEADER_SIZE;

    let first_raw_data = pe
        .sections
        .iter()
        .filter(|s| s.raw_size > 0)
        .map(|s| s.raw_offset as usize)
        .min()
        .unwrap_or(pe.size_of_headers as usize);

    if new_header_end > first_raw_data {
        bail!(
            "No room for new section header: headers end at 0x{:x}, first section data at 0x{:x}",
            new_header_end,
            first_raw_data
        );
    }

    Ok(TextExpansionLayout {
        header_offset: new_header_offset,
        virtual_address: next_va,
        raw_offset: next_raw,
    })
}

/// Pick a benign-looking section name based on existing sections.
fn pick_section_name(pe: &PeFile) -> &'static [u8; 8] {
    let names: Vec<&str> = pe.sections.iter().map(|s| s.name.as_str()).collect();

    // If there's already a .text, use .text1 (common in multi-TU builds)
    if names.contains(&".text") && !names.contains(&".text1") {
        return b".text1\0\0";
    }
    // MSVC-style alternatives
    if !names.contains(&".textbss") {
        return b".textbss";
    }
    if !names.contains(&".text0") {
        return b".text0\0\0";
    }
    // Fallback
    b".rtext\0\0"
}

/// Write the obfuscated PE: add a code section with a normal-looking name,
/// patch trampolines, fix headers.
pub fn write_pe(
    pe: &PeFile,
    obfuscated: &[ObfuscatedFunction],
    layout: &TextExpansionLayout,
) -> Result<Vec<u8>> {
    if obfuscated.is_empty() {
        bail!("No functions were obfuscated");
    }

    // Concatenate all obfuscated code
    let mut code_data = Vec::new();
    let mut function_offsets: Vec<(u32, u32, u32)> = Vec::new();

    for func in obfuscated {
        let offset = code_data.len() as u32;
        function_offsets.push((func.original_rva, func.original_size, offset));
        code_data.extend_from_slice(&func.code);
    }

    let virtual_size = code_data.len() as u32;
    let raw_size = align_up(virtual_size, pe.file_alignment);

    // Pad to file alignment with int3
    code_data.resize(raw_size as usize, 0xCC);

    // Build output
    let insert_at = layout.raw_offset as usize;
    let mut output = Vec::with_capacity(pe.data.len() + code_data.len());
    output.extend_from_slice(&pe.data[..insert_at]);
    output.extend_from_slice(&code_data);
    if insert_at < pe.data.len() {
        output.extend_from_slice(&pe.data[insert_at..]);
    }

    // 1. Write section header with a normal-looking name
    let section_name = pick_section_name(pe);
    let mut header = [0u8; SECTION_HEADER_SIZE];
    header[0..8].copy_from_slice(section_name);
    header[8..12].copy_from_slice(&virtual_size.to_le_bytes());
    header[12..16].copy_from_slice(&layout.virtual_address.to_le_bytes());
    header[16..20].copy_from_slice(&raw_size.to_le_bytes());
    header[20..24].copy_from_slice(&layout.raw_offset.to_le_bytes());
    header[36..40].copy_from_slice(&CODE_SECTION_CHARACTERISTICS.to_le_bytes());

    output[layout.header_offset..layout.header_offset + SECTION_HEADER_SIZE]
        .copy_from_slice(&header);

    // 2. Patch NumberOfSections
    let new_section_count = pe.number_of_sections + 1;
    output[pe.number_of_sections_offset..pe.number_of_sections_offset + 2]
        .copy_from_slice(&new_section_count.to_le_bytes());

    // 3. Patch SizeOfImage
    let new_size_of_image = align_up(
        layout.virtual_address + virtual_size,
        pe.section_alignment,
    );
    output[pe.size_of_image_offset..pe.size_of_image_offset + 4]
        .copy_from_slice(&new_size_of_image.to_le_bytes());

    // 4. Patch SizeOfCode
    let old_size_of_code =
        u32::from_le_bytes(output[pe.size_of_code_offset..pe.size_of_code_offset + 4].try_into().unwrap());
    let new_size_of_code = old_size_of_code + raw_size;
    output[pe.size_of_code_offset..pe.size_of_code_offset + 4]
        .copy_from_slice(&new_size_of_code.to_le_bytes());

    // 5. Zero checksum
    output[pe.checksum_offset..pe.checksum_offset + 4].copy_from_slice(&0u32.to_le_bytes());

    // 6. Write trampolines over original function bodies
    for &(original_rva, original_size, offset_in_code) in &function_offsets {
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

    let name_str = std::str::from_utf8(section_name).unwrap_or("?").trim_end_matches('\0');
    log::info!(
        "Added {} section: VA=0x{:x}, size={}, {} functions trampolined",
        name_str,
        layout.virtual_address,
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
