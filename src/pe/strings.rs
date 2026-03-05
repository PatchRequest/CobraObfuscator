use anyhow::Result;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::types::{PeFile, PeSectionInfo};

/// IMAGE_SCN_MEM_WRITE
const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;
/// IMAGE_SCN_MEM_EXECUTE
const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
/// IMAGE_SCN_CNT_CODE
const IMAGE_SCN_CNT_CODE: u32 = 0x00000020;

/// A string in .rdata that will be encrypted.
#[derive(Debug)]
pub struct EncryptedString {
    /// RVA of the string in .rdata.
    pub rva: u32,
    /// Number of content bytes to encrypt (not including null terminator).
    pub length: u32,
    /// XOR key for encryption (non-zero).
    pub key: u8,
}

fn is_printable(b: u8) -> bool {
    matches!(b, 0x20..=0x7E | b'\n' | b'\r' | b'\t')
}

/// Measure length of a null-terminated printable string. Returns None if invalid.
fn string_length(bytes: &[u8], max_len: usize) -> Option<usize> {
    let limit = bytes.len().min(max_len);
    for i in 0..limit {
        if bytes[i] == 0 {
            return if i >= 4 { Some(i) } else { None };
        }
        if !is_printable(bytes[i]) {
            return None;
        }
    }
    None
}

fn find_rdata_sections(pe: &PeFile) -> Vec<&PeSectionInfo> {
    pe.sections
        .iter()
        .filter(|s| s.name == ".rdata" || s.name == ".rodata")
        .collect()
}

/// Check if an RVA falls within any PE data directory range (imports, exports, debug, etc.).
fn is_in_data_directory(pe: &PeFile, rva: u32, len: u32) -> bool {
    for i in 0..pe.number_of_rva_and_sizes as usize {
        let offset = pe.data_directory_offset + i * 8;
        if offset + 8 > pe.data.len() {
            break;
        }
        let dir_rva = u32::from_le_bytes(pe.data[offset..offset + 4].try_into().unwrap());
        let dir_size = u32::from_le_bytes(pe.data[offset + 4..offset + 8].try_into().unwrap());
        if dir_rva == 0 || dir_size == 0 {
            continue;
        }
        // Check overlap: [rva, rva+len) overlaps [dir_rva, dir_rva+dir_size)
        if rva < dir_rva + dir_size && rva + len > dir_rva {
            return true;
        }
    }
    false
}

/// Discover strings in .rdata referenced by LEA [rip+disp32] instructions in code.
pub fn discover_string_refs(pe: &PeFile) -> Vec<EncryptedString> {
    let rdata_sections = find_rdata_sections(pe);
    if rdata_sections.is_empty() {
        return vec![];
    }

    let mut targets = std::collections::HashSet::new();

    // Scan all functions for LEA reg, [rip+disp32] targeting .rdata
    for func in &pe.functions {
        let va = pe.image_base + func.start_rva as u64;
        let mut decoder = iced_x86::Decoder::with_ip(
            64,
            &func.code,
            va,
            iced_x86::DecoderOptions::NONE,
        );
        while decoder.can_decode() {
            let insn = decoder.decode();
            if insn.mnemonic() == iced_x86::Mnemonic::Lea
                && insn.op1_kind() == iced_x86::OpKind::Memory
                && insn.memory_base() == iced_x86::Register::RIP
                && insn.memory_index() == iced_x86::Register::None
            {
                let target_va = insn.memory_displacement64();
                if target_va >= pe.image_base {
                    let target_rva = (target_va - pe.image_base) as u32;
                    if rdata_sections.iter().any(|s| s.contains_rva(target_rva)) {
                        targets.insert(target_rva);
                    }
                }
            }
        }
    }

    // Validate each target as a printable string
    let mut strings = Vec::new();
    for &rva in &targets {
        for section in &rdata_sections {
            if let Some(offset) = section.rva_to_offset(rva) {
                if offset < pe.data.len() {
                    let bytes = &pe.data[offset..];
                    if let Some(len) = string_length(bytes, 4096) {
                        // Skip strings in PE data directories (import names, export names, etc.)
                        if !is_in_data_directory(pe, rva, len as u32) {
                            strings.push(EncryptedString {
                                rva,
                                length: len as u32,
                                key: 0,
                            });
                        }
                    }
                }
                break;
            }
        }
    }

    // Sort by RVA, deduplicate, remove overlaps
    strings.sort_by_key(|s| s.rva);
    strings.dedup_by_key(|s| s.rva);

    let mut filtered = Vec::new();
    let mut last_end: u32 = 0;
    for s in strings {
        if s.rva >= last_end {
            last_end = s.rva + s.length;
            filtered.push(s);
        }
    }

    filtered
}

/// Generate x86-64 decryptor machine code.
///
/// Layout:
///   push rcx / push rdx / push r8    (prologue — preserve DLL params)
///   [per-string decrypt blocks]
///   pop r8 / pop rdx / pop rcx        (epilogue)
///   jmp original_entry
pub fn generate_decryptor(
    strings: &[EncryptedString],
    decryptor_rva: u32,
    original_entry_rva: u32,
) -> Vec<u8> {
    let mut code = Vec::new();

    // Prologue: save registers (needed for DLL entry points)
    code.push(0x51); // push rcx
    code.push(0x52); // push rdx
    code.extend_from_slice(&[0x41, 0x50]); // push r8

    let prologue_size = 4u32;

    // Per-string block (22 bytes each):
    //   lea rax, [rip+disp32]     7 bytes
    //   mov ecx, <length>         5 bytes
    //   xor byte [rax], <key>     3 bytes  (loop start)
    //   inc rax                   3 bytes
    //   dec ecx                   2 bytes
    //   jnz loop                  2 bytes
    for (i, s) in strings.iter().enumerate() {
        let block_offset = prologue_size + (i as u32) * 22;

        // LEA RAX, [rip+disp32]
        let lea_ip_after = decryptor_rva + block_offset + 7;
        let disp = (s.rva as i64 - lea_ip_after as i64) as i32;
        code.push(0x48);
        code.push(0x8D);
        code.push(0x05);
        code.extend_from_slice(&disp.to_le_bytes());

        // MOV ECX, length
        code.push(0xB9);
        code.extend_from_slice(&s.length.to_le_bytes());

        // .loop: XOR BYTE [RAX], key
        code.push(0x80);
        code.push(0x30);
        code.push(s.key);

        // INC RAX
        code.push(0x48);
        code.push(0xFF);
        code.push(0xC0);

        // DEC ECX
        code.push(0xFF);
        code.push(0xC9);

        // JNZ .loop (-10)
        code.push(0x75);
        code.push(0xF6u8);
    }

    // Epilogue: restore registers
    code.extend_from_slice(&[0x41, 0x58]); // pop r8
    code.push(0x5A); // pop rdx
    code.push(0x59); // pop rcx

    // JMP original_entry (E9 rel32)
    let jmp_rva = decryptor_rva + code.len() as u32;
    let jmp_ip_after = jmp_rva + 5;
    let jmp_rel = (original_entry_rva as i64 - jmp_ip_after as i64) as i32;
    code.push(0xE9);
    code.extend_from_slice(&jmp_rel.to_le_bytes());

    code
}

fn align_up(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

/// Apply string encryption to an already-written PE output buffer.
///
/// Post-processing step:
/// 1. Discover string references in the original PE
/// 2. Append a decryptor function to the last section
/// 3. XOR-encrypt strings in .rdata
/// 4. Make .rdata writable
/// 5. Hook entry point to decryptor
///
/// Returns the number of strings encrypted.
pub fn apply_string_encryption(
    output: &mut Vec<u8>,
    pe: &PeFile,
    seed: u64,
) -> Result<u32> {
    let mut strings = discover_string_refs(pe);
    if strings.is_empty() {
        log::info!("No string references found — skipping string encryption");
        return Ok(0);
    }

    // Assign per-string XOR keys
    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(0xDEAD_BEEF));
    for s in &mut strings {
        s.key = rng.gen_range(1..=255u8);
    }

    log::info!("Encrypting {} strings in .rdata", strings.len());

    // Read current last section state from the output buffer
    let last = &pe.sections[pe.sections.len() - 1];
    let sh = last.header_offset;
    let current_raw_size =
        u32::from_le_bytes(output[sh + 16..sh + 20].try_into().unwrap());
    let current_virtual_size =
        u32::from_le_bytes(output[sh + 8..sh + 12].try_into().unwrap());
    let current_chars =
        u32::from_le_bytes(output[sh + 36..sh + 40].try_into().unwrap());

    // Decryptor goes right after the current raw data of the last section
    let decryptor_rva = last.virtual_address + current_raw_size;
    let decryptor_file_offset = (last.raw_offset + current_raw_size) as usize;

    let decryptor = generate_decryptor(&strings, decryptor_rva, pe.entry_point_rva);
    let decryptor_size = decryptor.len() as u32;

    log::info!(
        "Decryptor: {} bytes at RVA 0x{:x} (file offset 0x{:x})",
        decryptor_size,
        decryptor_rva,
        decryptor_file_offset
    );

    // Extend last section to include decryptor
    let new_raw_size = align_up(current_raw_size + decryptor_size, pe.file_alignment);
    let new_virtual_size = std::cmp::max(current_virtual_size, new_raw_size);
    let extension_bytes = (new_raw_size - current_raw_size) as usize;

    if decryptor_file_offset >= output.len() {
        // Last section is at or past end of file — append
        output.resize(decryptor_file_offset + extension_bytes, 0xCC);
    } else {
        // Data exists after last section — insert
        let tail = output[decryptor_file_offset..].to_vec();
        output.truncate(decryptor_file_offset);
        output.resize(decryptor_file_offset + extension_bytes, 0xCC);
        output.extend_from_slice(&tail);
    }

    // Write decryptor code
    output[decryptor_file_offset..decryptor_file_offset + decryptor.len()]
        .copy_from_slice(&decryptor);

    // Encrypt strings in .rdata
    let rdata_sections = find_rdata_sections(pe);
    for s in &strings {
        for section in &rdata_sections {
            if let Some(offset) = section.rva_to_offset(s.rva) {
                if offset + s.length as usize <= output.len() {
                    for i in 0..s.length as usize {
                        output[offset + i] ^= s.key;
                    }
                }
                break;
            }
        }
    }

    // Make .rdata writable so decryptor can XOR in place
    for section in &rdata_sections {
        let rsh = section.header_offset;
        let chars = u32::from_le_bytes(output[rsh + 36..rsh + 40].try_into().unwrap());
        let new_chars = chars | IMAGE_SCN_MEM_WRITE;
        output[rsh + 36..rsh + 40].copy_from_slice(&new_chars.to_le_bytes());
    }

    // Update last section header
    output[sh + 8..sh + 12].copy_from_slice(&new_virtual_size.to_le_bytes());
    output[sh + 16..sh + 20].copy_from_slice(&new_raw_size.to_le_bytes());
    let new_chars = current_chars | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_CNT_CODE;
    output[sh + 36..sh + 40].copy_from_slice(&new_chars.to_le_bytes());

    // Update SizeOfImage
    let new_size_of_image = align_up(
        last.virtual_address + new_virtual_size,
        pe.section_alignment,
    );
    output[pe.size_of_image_offset..pe.size_of_image_offset + 4]
        .copy_from_slice(&new_size_of_image.to_le_bytes());

    // Update SizeOfCode
    let old_size_of_code = u32::from_le_bytes(
        output[pe.size_of_code_offset..pe.size_of_code_offset + 4]
            .try_into()
            .unwrap(),
    );
    let new_size_of_code = old_size_of_code + (new_raw_size - current_raw_size);
    output[pe.size_of_code_offset..pe.size_of_code_offset + 4]
        .copy_from_slice(&new_size_of_code.to_le_bytes());

    // Patch entry point to decryptor
    // AddressOfEntryPoint is at optional_header_offset + 16 = data_directory_offset - 96
    let entry_point_offset = pe.data_directory_offset - 96;
    output[entry_point_offset..entry_point_offset + 4]
        .copy_from_slice(&decryptor_rva.to_le_bytes());

    // Zero checksum
    output[pe.checksum_offset..pe.checksum_offset + 4].copy_from_slice(&0u32.to_le_bytes());

    log::info!(
        "String encryption complete: {} strings, entry point hooked 0x{:x} -> 0x{:x}",
        strings.len(),
        pe.entry_point_rva,
        decryptor_rva
    );

    Ok(strings.len() as u32)
}
