use anyhow::Result;

use super::instruction::{InsnRelocation, IrInsn};
use crate::coff::types::CoffRelocation;

/// Attach COFF relocations to the instructions they target.
///
/// For each relocation, we find the instruction whose byte range contains the
/// relocation offset and attach it. The `offset_in_insn` field records how far
/// into the instruction the relocation points.
pub fn attach_relocations(
    mut insns: Vec<IrInsn>,
    relocs: &[CoffRelocation],
) -> Result<Vec<IrInsn>> {
    for reloc in relocs {
        let target_offset = reloc.offset;

        // Find the instruction that contains this relocation offset
        let mut found = false;
        for insn in insns.iter_mut() {
            let insn_start = insn.original_offset;
            let insn_end = insn_start + insn.original_length as u64;

            if target_offset >= insn_start && target_offset < insn_end {
                let offset_in_insn = (target_offset - insn_start) as u32;
                insn.relocation = Some(InsnRelocation {
                    symbol_index: reloc.symbol_index,
                    typ: reloc.typ,
                    offset_in_insn,
                });
                found = true;
                break;
            }
        }

        if !found {
            log::warn!(
                "Relocation at offset 0x{:x} (sym={}, type=0x{:x}) does not map to any instruction",
                target_offset,
                reloc.symbol_index,
                reloc.typ
            );
        }
    }

    Ok(insns)
}
