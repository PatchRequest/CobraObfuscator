use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::types::PeFile;

const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;
const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
const IMAGE_SCN_CNT_CODE: u32 = 0x00000020;

/// A parsed import entry.
#[derive(Debug, Clone)]
struct ImportEntry {
    func_name: String,
    iat_slot_rva: u32,
    is_ordinal: bool,
    ordinal: u16,
}

/// A DLL with its imported functions.
#[derive(Debug)]
struct ImportedDll {
    name: String,
    functions: Vec<ImportEntry>,
}

/// A code location that references an IAT slot.
#[derive(Debug)]
struct IatReference {
    /// File offset of the displacement bytes (last 4 bytes of instruction).
    disp_file_offset: usize,
    /// RVA of the instruction end (for computing new displacement).
    insn_end_rva: u32,
    /// The IAT slot RVA being referenced.
    iat_slot_rva: u32,
}

fn align_up(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

fn rva_to_file_offset(pe: &PeFile, rva: u32) -> Option<usize> {
    pe.sections.iter().find_map(|s| s.rva_to_offset(rva))
}

fn read_null_string(data: &[u8], offset: usize) -> String {
    let mut end = offset;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }
    String::from_utf8_lossy(&data[offset..end]).to_string()
}

/// Parse all imports from the PE import directory.
fn parse_imports(data: &[u8], pe: &PeFile) -> Result<Vec<ImportedDll>> {
    let dir_offset = pe.data_directory_offset + 1 * 8;
    if dir_offset + 8 > data.len() {
        return Ok(vec![]);
    }

    let import_rva = u32::from_le_bytes(data[dir_offset..dir_offset + 4].try_into().unwrap());
    let import_size = u32::from_le_bytes(data[dir_offset + 4..dir_offset + 8].try_into().unwrap());

    if import_rva == 0 || import_size == 0 {
        return Ok(vec![]);
    }

    let import_file_offset = rva_to_file_offset(pe, import_rva)
        .context("Cannot find import directory in file")?;

    let mut dlls = Vec::new();
    let mut pos = import_file_offset;

    loop {
        if pos + 20 > data.len() {
            break;
        }

        let ilt_rva = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        let name_rva = u32::from_le_bytes(data[pos + 12..pos + 16].try_into().unwrap());
        let iat_rva = u32::from_le_bytes(data[pos + 16..pos + 20].try_into().unwrap());

        if ilt_rva == 0 && name_rva == 0 && iat_rva == 0 {
            break;
        }

        let name_offset = rva_to_file_offset(pe, name_rva)
            .context("Cannot find DLL name in file")?;
        let dll_name = read_null_string(data, name_offset);

        let lookup_rva = if ilt_rva != 0 { ilt_rva } else { iat_rva };
        let lookup_offset = rva_to_file_offset(pe, lookup_rva)
            .context("Cannot find ILT/IAT in file")?;

        let mut functions = Vec::new();
        let mut idx = 0u32;

        loop {
            let entry_off = lookup_offset + (idx as usize) * 8;
            if entry_off + 8 > data.len() {
                break;
            }

            let entry = u64::from_le_bytes(data[entry_off..entry_off + 8].try_into().unwrap());
            if entry == 0 {
                break;
            }

            let slot_rva = iat_rva + idx * 8;

            if entry & (1u64 << 63) != 0 {
                functions.push(ImportEntry {
                    func_name: String::new(),
                    iat_slot_rva: slot_rva,
                    is_ordinal: true,
                    ordinal: (entry & 0xFFFF) as u16,
                });
            } else {
                let hint_name_rva = (entry & 0x7FFFFFFF) as u32;
                let hint_name_offset = rva_to_file_offset(pe, hint_name_rva)
                    .context("Cannot find hint-name entry")?;
                let func_name = read_null_string(data, hint_name_offset + 2);
                functions.push(ImportEntry {
                    func_name,
                    iat_slot_rva: slot_rva,
                    is_ordinal: false,
                    ordinal: 0,
                });
            }

            idx += 1;
        }

        dlls.push(ImportedDll {
            name: dll_name,
            functions,
        });

        pos += 20;
    }

    Ok(dlls)
}

/// Scan executable sections for instructions referencing IAT slots.
fn scan_iat_references(
    output: &[u8],
    pe: &PeFile,
    iat_rvas: &HashSet<u32>,
) -> Vec<IatReference> {
    let mut refs = Vec::new();

    for section in &pe.sections {
        let sh = section.header_offset;
        let chars = u32::from_le_bytes(output[sh + 36..sh + 40].try_into().unwrap());
        if chars & IMAGE_SCN_MEM_EXECUTE == 0 && chars & IMAGE_SCN_CNT_CODE == 0 {
            continue;
        }

        let raw_size = u32::from_le_bytes(output[sh + 16..sh + 20].try_into().unwrap());
        let end = (section.raw_offset + raw_size) as usize;
        if end > output.len() {
            continue;
        }

        let section_data = &output[section.raw_offset as usize..end];
        let section_va = pe.image_base + section.virtual_address as u64;

        let mut decoder = iced_x86::Decoder::with_ip(
            64,
            section_data,
            section_va,
            iced_x86::DecoderOptions::NONE,
        );

        while decoder.can_decode() {
            let insn = decoder.decode();
            if insn.is_invalid() {
                continue;
            }

            if insn.memory_base() != iced_x86::Register::RIP
                || insn.memory_index() != iced_x86::Register::None
            {
                continue;
            }

            let target_va = insn.memory_displacement64();
            if target_va < pe.image_base {
                continue;
            }

            let target_rva = (target_va - pe.image_base) as u32;
            if !iat_rvas.contains(&target_rva) {
                continue;
            }

            let insn_rva = (insn.ip() - pe.image_base) as u32;
            let insn_end_rva = insn_rva + insn.len() as u32;
            if let Some(insn_file_offset) = section.rva_to_offset(insn_rva) {
                let disp_file_offset = insn_file_offset + insn.len() - 4;
                refs.push(IatReference {
                    disp_file_offset,
                    insn_end_rva,
                    iat_slot_rva: target_rva,
                });
            }
        }
    }

    refs
}

/// Build XOR-encrypted names blob containing all DLL and function names.
fn build_names_blob(
    dlls: &[ImportedDll],
    xor_key: u8,
) -> (Vec<u8>, Vec<u32>, Vec<Vec<u32>>) {
    let mut blob = Vec::new();
    let mut dll_offsets = Vec::new();
    let mut func_offsets = Vec::new();

    for dll in dlls {
        dll_offsets.push(blob.len() as u32);
        for b in dll.name.as_bytes() {
            blob.push(b ^ xor_key);
        }
        blob.push(xor_key); // encrypted null terminator

        let mut func_offs = Vec::new();
        for func in &dll.functions {
            if func.is_ordinal {
                func_offs.push(0);
            } else {
                func_offs.push(blob.len() as u32);
                for b in func.func_name.as_bytes() {
                    blob.push(b ^ xor_key);
                }
                blob.push(xor_key);
            }
        }
        func_offsets.push(func_offs);
    }

    (blob, dll_offsets, func_offsets)
}

/// Helper for emitting x86-64 machine code with RIP-relative addressing.
struct CodeBuilder {
    code: Vec<u8>,
    base_rva: u32,
}

impl CodeBuilder {
    fn new(base_rva: u32) -> Self {
        Self {
            code: Vec::new(),
            base_rva,
        }
    }

    fn current_rva(&self) -> u32 {
        self.base_rva + self.code.len() as u32
    }

    fn emit(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    fn rip_disp(&self, insn_size: u32, target_rva: u32) -> i32 {
        let ip_after = self.current_rva() + insn_size;
        (target_rva as i64 - ip_after as i64) as i32
    }

    /// lea rcx, [rip+disp32] — 7 bytes
    fn emit_lea_rcx_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x48, 0x8D, 0x0D]);
        self.emit(&disp.to_le_bytes());
    }

    /// lea rdx, [rip+disp32] — 7 bytes
    fn emit_lea_rdx_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x48, 0x8D, 0x15]);
        self.emit(&disp.to_le_bytes());
    }

    /// lea rax, [rip+disp32] — 7 bytes
    fn emit_lea_rax_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x48, 0x8D, 0x05]);
        self.emit(&disp.to_le_bytes());
    }

    /// call qword [rip+disp32] — 6 bytes
    fn emit_call_qword_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(6, target_rva);
        self.emit(&[0xFF, 0x15]);
        self.emit(&disp.to_le_bytes());
    }

    /// mov qword [rip+disp32], rax — 7 bytes
    fn emit_mov_qword_rip_rax(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x48, 0x89, 0x05]);
        self.emit(&disp.to_le_bytes());
    }

    /// jmp rel32 — 5 bytes
    fn emit_jmp_rel32(&mut self, target_rva: u32) {
        let disp = self.rip_disp(5, target_rva);
        self.emit(&[0xE9]);
        self.emit(&disp.to_le_bytes());
    }
}

/// Pre-compute resolver code size.
fn compute_resolver_size(dlls: &[ImportedDll]) -> u32 {
    let mut size = 17 + 22 + 14; // prologue + decrypt + epilogue
    size += dlls.len() as u32 * 16; // per-DLL: lea + call + mov
    for dll in dlls {
        for func in &dll.functions {
            size += if func.is_ordinal { 21 } else { 23 };
        }
    }
    size
}

/// Generate the import resolver stub (entry-point mode).
///
/// Decrypts names, calls LoadLibraryA per DLL, GetProcAddress per function,
/// stores results in the shadow IAT, then jumps to the original entry.
fn generate_resolver(
    dlls: &[ImportedDll],
    resolver_rva: u32,
    shadow_iat_rva: u32,
    names_rva: u32,
    names_len: u32,
    xor_key: u8,
    loadlib_iat_rva: u32,
    getproc_iat_rva: u32,
    next_entry_rva: u32,
    dll_name_offsets: &[u32],
    func_name_offsets: &[Vec<u32>],
    shadow_indices: &HashMap<u32, u32>,
) -> Vec<u8> {
    let mut cb = CodeBuilder::new(resolver_rva);

    // Prologue: save DLL params + callee-saved regs, align stack
    cb.emit(&[0x51]); // push rcx
    cb.emit(&[0x52]); // push rdx
    cb.emit(&[0x41, 0x50]); // push r8
    cb.emit(&[0x53]); // push rbx
    cb.emit(&[0x55]); // push rbp
    cb.emit(&[0x48, 0x89, 0xE5]); // mov rbp, rsp
    cb.emit(&[0x48, 0x83, 0xE4, 0xF0]); // and rsp, -16
    cb.emit(&[0x48, 0x83, 0xEC, 0x20]); // sub rsp, 32

    // Decrypt names blob in-place
    cb.emit_lea_rax_rip(names_rva);
    cb.emit(&[0xB9]); // mov ecx, names_len
    cb.emit(&names_len.to_le_bytes());
    cb.emit(&[0x80, 0x30, xor_key]); // xor byte [rax], key
    cb.emit(&[0x48, 0xFF, 0xC0]); // inc rax
    cb.emit(&[0xFF, 0xC9]); // dec ecx
    cb.emit(&[0x75, 0xF6]); // jnz .loop (-10)

    // Per-DLL resolution
    for (di, dll) in dlls.iter().enumerate() {
        cb.emit_lea_rcx_rip(names_rva + dll_name_offsets[di]);
        cb.emit_call_qword_rip(loadlib_iat_rva);
        cb.emit(&[0x48, 0x89, 0xC3]); // mov rbx, rax

        for (fi, func) in dll.functions.iter().enumerate() {
            let shadow_idx = shadow_indices[&func.iat_slot_rva];
            let shadow_slot_rva = shadow_iat_rva + shadow_idx * 8;

            cb.emit(&[0x48, 0x89, 0xD9]); // mov rcx, rbx

            if func.is_ordinal {
                cb.emit(&[0xBA]); // mov edx, ordinal
                cb.emit(&(func.ordinal as u32).to_le_bytes());
            } else {
                cb.emit_lea_rdx_rip(names_rva + func_name_offsets[di][fi]);
            }

            cb.emit_call_qword_rip(getproc_iat_rva);
            cb.emit_mov_qword_rip_rax(shadow_slot_rva);
        }
    }

    // Epilogue
    cb.emit(&[0x48, 0x89, 0xEC]); // mov rsp, rbp
    cb.emit(&[0x5D]); // pop rbp
    cb.emit(&[0x5B]); // pop rbx
    cb.emit(&[0x41, 0x58]); // pop r8
    cb.emit(&[0x5A]); // pop rdx
    cb.emit(&[0x59]); // pop rcx
    cb.emit_jmp_rel32(next_entry_rva);

    cb.code
}

/// Find existing LoadLibraryA and GetProcAddress IAT slot RVAs.
fn find_bootstrap_iat_slots(dlls: &[ImportedDll]) -> Result<(u32, u32)> {
    let mut loadlib = None;
    let mut getproc = None;
    for dll in dlls {
        for func in &dll.functions {
            if func.func_name == "LoadLibraryA" {
                loadlib = Some(func.iat_slot_rva);
            } else if func.func_name == "GetProcAddress" {
                getproc = Some(func.iat_slot_rva);
            }
        }
    }
    Ok((
        loadlib.context("Binary does not import LoadLibraryA")?,
        getproc.context("Binary does not import GetProcAddress")?,
    ))
}

/// Apply import hiding to an already-written PE output buffer.
///
/// Strategy:
/// 1. Parse original imports, find existing LoadLibraryA + GetProcAddress IAT slots
/// 2. Scan code for IAT references
/// 3. Append shadow IAT + encrypted names + resolver to the last section
/// 4. Patch code references to use shadow IAT
/// 5. Hook entry point to resolver
///
/// The PE loader fills the original IAT normally. The resolver then re-resolves
/// all imports via LoadLibraryA + GetProcAddress and stores results in the shadow
/// IAT. Code references are patched to read from the shadow IAT, so the original
/// IAT is unused by code at runtime.
///
/// Returns the number of hidden imports.
pub fn apply_import_hiding(
    output: &mut Vec<u8>,
    pe: &PeFile,
    seed: u64,
) -> Result<u32> {
    let dlls = parse_imports(&pe.data, pe)?;
    if dlls.is_empty() {
        log::info!("No imports found — skipping import hiding");
        return Ok(0);
    }

    let total_imports: u32 = dlls.iter().map(|d| d.functions.len() as u32).sum();
    log::info!(
        "Parsed {} DLLs with {} total imports",
        dlls.len(),
        total_imports
    );

    let (loadlib_iat_rva, getproc_iat_rva) = find_bootstrap_iat_slots(&dlls)?;
    log::info!(
        "Bootstrap IAT slots: LoadLibraryA=0x{:x}, GetProcAddress=0x{:x}",
        loadlib_iat_rva,
        getproc_iat_rva,
    );

    // Build IAT slot → shadow index mapping
    let mut shadow_indices: HashMap<u32, u32> = HashMap::new();
    let mut shadow_index = 0u32;
    for dll in &dlls {
        for func in &dll.functions {
            shadow_indices.insert(func.iat_slot_rva, shadow_index);
            shadow_index += 1;
        }
    }

    // Scan code for IAT references
    let iat_rvas: HashSet<u32> = shadow_indices.keys().copied().collect();
    let references = scan_iat_references(output, pe, &iat_rvas);

    if references.is_empty() {
        log::info!("No IAT references found in code — skipping import hiding");
        return Ok(0);
    }

    log::info!("Found {} IAT references in code", references.len());

    // Build encrypted names blob
    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(0xCAFE_BABE));
    let xor_key: u8 = rng.gen_range(1..=255u8);
    let (names_blob, dll_name_offsets, func_name_offsets) = build_names_blob(&dlls, xor_key);

    // Compute layout — extension: [shadow_iat][names_blob][resolver_code]
    let last = &pe.sections[pe.sections.len() - 1];
    let sh = last.header_offset;
    let current_raw_size =
        u32::from_le_bytes(output[sh + 16..sh + 20].try_into().unwrap());
    let current_virtual_size =
        u32::from_le_bytes(output[sh + 8..sh + 12].try_into().unwrap());
    let current_chars =
        u32::from_le_bytes(output[sh + 36..sh + 40].try_into().unwrap());

    let base_rva = align_up(last.virtual_address + current_raw_size, 8);
    let padding_before = base_rva - (last.virtual_address + current_raw_size);

    let shadow_iat_rva = base_rva;
    let shadow_iat_size = total_imports * 8;

    let names_rva = shadow_iat_rva + shadow_iat_size;
    let names_size = names_blob.len() as u32;

    let resolver_rva = align_up(names_rva + names_size, 2);
    let resolver_padding = resolver_rva - (names_rva + names_size);
    let resolver_size = compute_resolver_size(&dlls);

    let total_append =
        padding_before + shadow_iat_size + names_size + resolver_padding + resolver_size;

    // Read current entry point
    let entry_point_file_offset = pe.data_directory_offset - 96;
    let current_entry_rva = u32::from_le_bytes(
        output[entry_point_file_offset..entry_point_file_offset + 4]
            .try_into()
            .unwrap(),
    );

    // Generate resolver
    let mut resolver = generate_resolver(
        &dlls,
        resolver_rva,
        shadow_iat_rva,
        names_rva,
        names_size,
        xor_key,
        loadlib_iat_rva,
        getproc_iat_rva,
        current_entry_rva,
        &dll_name_offsets,
        &func_name_offsets,
        &shadow_indices,
    );
    assert!(
        resolver.len() <= resolver_size as usize,
        "Resolver code ({}) exceeds predicted size ({})",
        resolver.len(),
        resolver_size
    );
    resolver.resize(resolver_size as usize, 0xCC);

    // Extend last section
    let new_raw_size = align_up(current_raw_size + total_append, pe.file_alignment);
    let new_virtual_size = std::cmp::max(current_virtual_size, new_raw_size);
    let append_file_offset = (last.raw_offset + current_raw_size) as usize;
    let extension_bytes = (new_raw_size - current_raw_size) as usize;

    if append_file_offset >= output.len() {
        output.resize(append_file_offset + extension_bytes, 0);
    } else {
        let tail = output[append_file_offset..].to_vec();
        output.truncate(append_file_offset);
        output.resize(append_file_offset + extension_bytes, 0);
        output.extend_from_slice(&tail);
    }

    // Write encrypted names blob
    let names_file_offset =
        append_file_offset + padding_before as usize + shadow_iat_size as usize;
    output[names_file_offset..names_file_offset + names_blob.len()]
        .copy_from_slice(&names_blob);

    // Write resolver code
    let resolver_file_offset =
        names_file_offset + names_size as usize + resolver_padding as usize;
    output[resolver_file_offset..resolver_file_offset + resolver.len()]
        .copy_from_slice(&resolver);

    log::info!(
        "Import hiding layout: shadow_iat=0x{:x}, names=0x{:x}, resolver=0x{:x}",
        shadow_iat_rva, names_rva, resolver_rva,
    );

    // Patch code references to use shadow IAT
    for ref_ in &references {
        let shadow_idx = shadow_indices[&ref_.iat_slot_rva];
        let shadow_slot_rva = shadow_iat_rva + shadow_idx * 8;
        let new_disp = (shadow_slot_rva as i64 - ref_.insn_end_rva as i64) as i32;
        output[ref_.disp_file_offset..ref_.disp_file_offset + 4]
            .copy_from_slice(&new_disp.to_le_bytes());
    }

    // Update section headers
    output[sh + 8..sh + 12].copy_from_slice(&new_virtual_size.to_le_bytes());
    output[sh + 16..sh + 20].copy_from_slice(&new_raw_size.to_le_bytes());
    const IMAGE_SCN_MEM_DISCARDABLE: u32 = 0x02000000;
    let new_chars = (current_chars & !IMAGE_SCN_MEM_DISCARDABLE)
        | IMAGE_SCN_MEM_EXECUTE
        | IMAGE_SCN_MEM_WRITE
        | IMAGE_SCN_CNT_CODE;
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

    // Hook entry point to resolver
    output[entry_point_file_offset..entry_point_file_offset + 4]
        .copy_from_slice(&resolver_rva.to_le_bytes());

    // Zero checksum
    output[pe.checksum_offset..pe.checksum_offset + 4].fill(0);

    log::info!(
        "Import hiding complete: {} imports hidden across {} DLLs, {} code references patched, entry 0x{:x} -> 0x{:x}",
        total_imports,
        dlls.len(),
        references.len(),
        current_entry_rva,
        resolver_rva,
    );

    Ok(total_imports)
}
