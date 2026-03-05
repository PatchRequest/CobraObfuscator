use anyhow::{Result, bail};

use crate::coff::types::CoffRelocation;

/// Validate that all relocations point within the encoded code buffer.
pub fn validate_relocations(relocations: &[CoffRelocation], code_len: usize) -> Result<()> {
    for reloc in relocations {
        if reloc.offset as usize >= code_len {
            bail!(
                "Relocation at offset 0x{:x} (sym={}, type=0x{:x}) exceeds code length 0x{:x}",
                reloc.offset,
                reloc.symbol_index,
                reloc.typ,
                code_len
            );
        }
    }
    Ok(())
}

/// Sort relocations by offset (required by COFF format).
pub fn sort_relocations(relocations: &mut Vec<CoffRelocation>) {
    relocations.sort_by_key(|r| r.offset);
}
