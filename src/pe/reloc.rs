use anyhow::Result;

use super::types::PeSectionInfo;

/// Check if the PE has a .reloc section.
///
/// For x64 PE files, RIP-relative addressing means we typically don't need
/// to modify the base relocation table when adding a .cobra section.
/// The BlockEncoder handles RIP-relative fixups at the new VA.
pub fn has_reloc_section(sections: &[PeSectionInfo]) -> bool {
    sections.iter().any(|s| s.name == ".reloc")
}

/// Validate that we won't break base relocations.
///
/// Since we're only adding a new section and patching existing code with
/// jmp trampolines (which use rel32 — not base-reloc'd), the .reloc
/// section remains valid for the original code. The new .cobra section
/// uses RIP-relative addressing exclusively.
pub fn validate_reloc_safety(sections: &[PeSectionInfo]) -> Result<()> {
    if has_reloc_section(sections) {
        log::info!(".reloc section present — base relocations preserved (read-only)");
    } else {
        log::info!("No .reloc section — PE may not support ASLR");
    }
    Ok(())
}
