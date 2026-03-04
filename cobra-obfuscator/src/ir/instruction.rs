use anyhow::Result;
use iced_x86::{Decoder, DecoderOptions, Instruction};

/// A relocation attached to an IR instruction.
#[derive(Debug, Clone)]
pub struct InsnRelocation {
    /// Index into the COFF symbol table.
    pub symbol_index: u32,
    /// Relocation type (COFF relocation type).
    pub typ: u16,
    /// Byte offset within the encoded instruction where the relocation applies.
    pub offset_in_insn: u32,
}

/// An IR instruction wrapping an iced-x86 `Instruction` with metadata.
#[derive(Debug, Clone)]
pub struct IrInsn {
    /// The underlying x86-64 instruction.
    pub instruction: Instruction,
    /// Optional relocation attached to this instruction.
    pub relocation: Option<InsnRelocation>,
    /// Unique ID for this instruction (used for labels/references).
    pub id: u64,
    /// Whether this instruction was synthesized by a pass (not from original code).
    pub synthetic: bool,
    /// Original byte offset within the section (before any transforms).
    pub original_offset: u64,
    /// Encoded length of the original instruction.
    pub original_length: u32,
    /// Tags for pass-specific metadata.
    pub tags: Vec<String>,
}

impl IrInsn {
    pub fn new(instruction: Instruction, id: u64, offset: u64, length: u32) -> Self {
        Self {
            instruction,
            relocation: None,
            id,
            synthetic: false,
            original_offset: offset,
            original_length: length,
            tags: Vec::new(),
        }
    }

    pub fn synthetic(instruction: Instruction, id: u64) -> Self {
        Self {
            instruction,
            relocation: None,
            id,
            synthetic: true,
            original_offset: 0,
            original_length: 0,
            tags: Vec::new(),
        }
    }

    pub fn has_relocation(&self) -> bool {
        self.relocation.is_some()
    }
}

/// Decode raw bytes into a list of IR instructions.
pub fn decode_instructions(data: &[u8], base_address: u64) -> Result<Vec<IrInsn>> {
    let mut decoder = Decoder::with_ip(64, data, base_address, DecoderOptions::NONE);
    let mut insns = Vec::new();
    let mut next_id = 0u64;

    while decoder.can_decode() {
        let offset = decoder.position() as u64;
        let instr = decoder.decode();
        let length = decoder.position() as u64 - offset;

        let ir = IrInsn::new(instr, next_id, base_address + offset, length as u32);
        insns.push(ir);
        next_id += 1;
    }

    Ok(insns)
}
