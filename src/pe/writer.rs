use anyhow::{bail, Context, Result};

use super::types::PeFile;
use crate::pipeline::ObfuscatedFunction;

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

    // The section maps raw_size bytes from raw_offset to virtual_address.
    // Our code will be at offset raw_size within the section's raw data,
    // which maps to VA = virtual_address + raw_size.
    // We use raw_size (not virtual_size) because the loader maps from the file.
    let code_va = last.virtual_address + last.raw_size;

    // Append right after the last section's raw data in the file
    let raw_offset = last.raw_offset + last.raw_size;

    Ok(TextExpansionLayout {
        section_index: last_idx,
        virtual_address: code_va,
        raw_offset,
    })
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
