use anyhow::{bail, Context, Result};

use super::types::PeFile;
use crate::pipeline::ObfuscatedFunction;

/// Section header size.
const SECTION_HEADER_SIZE: usize = 40;

/// IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ
const COBRA_SECTION_CHARACTERISTICS: u32 = 0x60000020;

/// Align a value up to the given alignment.
fn align_up(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

/// Calculate the RVA and layout for the new .cobra section.
pub fn calculate_cobra_section(pe: &PeFile) -> Result<CobraSectionLayout> {
    // Find the end of the last section (both in VA space and file space)
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

    // First section's raw data starts at the earliest raw_offset
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

    Ok(CobraSectionLayout {
        header_offset: new_header_offset,
        virtual_address: next_va,
        raw_offset: next_raw,
    })
}

/// Layout information for the .cobra section.
pub struct CobraSectionLayout {
    pub header_offset: usize,
    pub virtual_address: u32,
    pub raw_offset: u32,
}

/// Write the obfuscated PE: add .cobra section, patch trampolines, fix headers.
pub fn write_pe(
    pe: &PeFile,
    obfuscated: &[ObfuscatedFunction],
    layout: &CobraSectionLayout,
) -> Result<Vec<u8>> {
    if obfuscated.is_empty() {
        bail!("No functions were obfuscated");
    }

    // Concatenate all obfuscated code
    let mut cobra_code = Vec::new();
    let mut function_offsets: Vec<(u32, u32, u32)> = Vec::new(); // (original_rva, original_size, offset_in_cobra)

    for func in obfuscated {
        let offset = cobra_code.len() as u32;
        function_offsets.push((func.original_rva, func.original_size, offset));
        cobra_code.extend_from_slice(&func.code);
    }

    let cobra_virtual_size = cobra_code.len() as u32;
    let cobra_raw_size = align_up(cobra_virtual_size, pe.file_alignment);

    // Pad cobra code to file alignment
    cobra_code.resize(cobra_raw_size as usize, 0xCC); // int3 padding

    // Build output: original data up to insertion point, then .cobra, then any overlay/trailing data
    let insert_at = layout.raw_offset as usize;
    let mut output = Vec::with_capacity(pe.data.len() + cobra_code.len());
    output.extend_from_slice(&pe.data[..insert_at]);
    output.extend_from_slice(&cobra_code);
    if insert_at < pe.data.len() {
        output.extend_from_slice(&pe.data[insert_at..]);
    }

    // 1. Write .cobra section header
    let mut header = [0u8; SECTION_HEADER_SIZE];
    // Name: ".cobra\0\0"
    header[0..6].copy_from_slice(b".cobra");
    // VirtualSize
    header[8..12].copy_from_slice(&cobra_virtual_size.to_le_bytes());
    // VirtualAddress
    header[12..16].copy_from_slice(&layout.virtual_address.to_le_bytes());
    // SizeOfRawData
    header[16..20].copy_from_slice(&cobra_raw_size.to_le_bytes());
    // PointerToRawData
    header[20..24].copy_from_slice(&layout.raw_offset.to_le_bytes());
    // PointerToRelocations, PointerToLinenumbers, NumberOfRelocations, NumberOfLinenumbers = 0
    // Characteristics
    header[36..40].copy_from_slice(&COBRA_SECTION_CHARACTERISTICS.to_le_bytes());

    output[layout.header_offset..layout.header_offset + SECTION_HEADER_SIZE]
        .copy_from_slice(&header);

    // 2. .cobra section data already spliced in at raw_offset

    // 3. Patch NumberOfSections
    let new_section_count = pe.number_of_sections + 1;
    output[pe.number_of_sections_offset..pe.number_of_sections_offset + 2]
        .copy_from_slice(&new_section_count.to_le_bytes());

    // 4. Patch SizeOfImage
    let new_size_of_image = align_up(
        layout.virtual_address + cobra_virtual_size,
        pe.section_alignment,
    );
    output[pe.size_of_image_offset..pe.size_of_image_offset + 4]
        .copy_from_slice(&new_size_of_image.to_le_bytes());

    // 5. Patch SizeOfCode
    let old_size_of_code =
        u32::from_le_bytes(output[pe.size_of_code_offset..pe.size_of_code_offset + 4].try_into().unwrap());
    let new_size_of_code = old_size_of_code + cobra_raw_size;
    output[pe.size_of_code_offset..pe.size_of_code_offset + 4]
        .copy_from_slice(&new_size_of_code.to_le_bytes());

    // 6. Zero checksum
    output[pe.checksum_offset..pe.checksum_offset + 4].copy_from_slice(&0u32.to_le_bytes());

    // 7. Write trampolines over original function bodies
    for &(original_rva, original_size, offset_in_cobra) in &function_offsets {
        let cobra_rva = layout.virtual_address + offset_in_cobra;

        // Find which section contains this function and convert RVA to file offset
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

        // jmp rel32: the displacement is relative to the end of the jmp instruction (5 bytes)
        let jmp_source_rva = original_rva + 5; // RVA after the jmp instruction
        let rel32 = (cobra_rva as i64 - jmp_source_rva as i64) as i32;

        // Write E9 <rel32>
        output[file_offset] = 0xE9;
        output[file_offset + 1..file_offset + 5].copy_from_slice(&rel32.to_le_bytes());

        // Fill rest with int3 (0xCC)
        let remaining = original_size as usize - 5;
        for i in 0..remaining {
            output[file_offset + 5 + i] = 0xCC;
        }
    }

    // Warn about Authenticode
    log::warn!("PE checksum zeroed — Authenticode signature (if present) is invalidated");

    log::info!(
        "Wrote .cobra section: VA=0x{:x}, size={}, {} functions trampolined",
        layout.virtual_address,
        cobra_code.len(),
        function_offsets.len()
    );

    Ok(output)
}
