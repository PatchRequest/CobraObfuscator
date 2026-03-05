pub mod instruction;
pub mod relocation;
pub mod basic_block;
pub mod cfg;
pub mod function;

pub use instruction::IrInsn;
pub use basic_block::BasicBlock;
pub use function::Function;

use anyhow::Result;
use crate::coff::types::CodeSection;

/// Decode a code section's bytes into a list of IR instructions with relocations attached.
pub fn decode_section(section: &CodeSection) -> Result<Vec<IrInsn>> {
    let insns = instruction::decode_instructions(&section.data, section.virtual_address)?;
    let insns = relocation::attach_relocations(insns, &section.relocations)?;
    Ok(insns)
}

/// Decode raw bytes into IR instructions without relocation attachment.
/// Used for PE mode where relocations are RIP-relative and handled by BlockEncoder.
pub fn decode_raw(data: &[u8], base_address: u64) -> Result<Vec<IrInsn>> {
    instruction::decode_instructions(data, base_address)
}
