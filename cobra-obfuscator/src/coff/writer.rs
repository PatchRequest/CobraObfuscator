use anyhow::{Context, Result};
use object::write::{Object, Symbol, SymbolSection};
use object::{
    Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
};

use super::types::*;

/// Emit a COFF object file from our data structures.
///
/// `transformed_code` contains the new bytes and relocations for each code section,
/// indexed by the code section's position in `coff.code_sections`.
pub fn write_coff(
    coff: &CoffObject,
    transformed_code: &[(Vec<u8>, Vec<CoffRelocation>)],
) -> Result<Vec<u8>> {
    let mut obj = Object::new(BinaryFormat::Coff, Architecture::X86_64, Endianness::Little);

    // Map from old section index to new section ID
    let mut section_map: std::collections::HashMap<usize, object::write::SectionId> =
        std::collections::HashMap::new();

    // Create code sections with transformed data
    for (i, code_section) in coff.code_sections.iter().enumerate() {
        let (ref new_data, _) = transformed_code[i];

        let section_id = obj.add_section(
            Vec::new(),
            code_section.name.as_bytes().to_vec(),
            SectionKind::Text,
        );

        let section = obj.section_mut(section_id);
        section.set_data(new_data.clone(), code_section.alignment as u64);
        section.flags = object::SectionFlags::Coff {
            characteristics: code_section.characteristics,
        };

        section_map.insert(code_section.section_index, section_id);
    }

    // Create raw (non-code) sections with original data
    for raw_section in &coff.raw_sections {
        let kind = guess_section_kind(&raw_section.name, raw_section.characteristics);

        let section_id = obj.add_section(
            Vec::new(),
            raw_section.name.as_bytes().to_vec(),
            kind,
        );

        let section = obj.section_mut(section_id);
        if !raw_section.data.is_empty() {
            section.set_data(raw_section.data.clone(), raw_section.alignment as u64);
        }
        section.flags = object::SectionFlags::Coff {
            characteristics: raw_section.characteristics,
        };

        section_map.insert(raw_section.section_index, section_id);
    }

    // Map from old symbol index to new symbol ID
    let mut symbol_map: std::collections::HashMap<usize, object::write::SymbolId> =
        std::collections::HashMap::new();

    // Create symbols
    for (old_idx, sym) in coff.symbols.iter().enumerate() {
        let section = if sym.section_number > 0 {
            let sec_idx = (sym.section_number as usize) - 1;
            if let Some(&sec_id) = section_map.get(&sec_idx) {
                SymbolSection::Section(sec_id)
            } else {
                SymbolSection::Undefined
            }
        } else if sym.section_number == 0 {
            SymbolSection::Undefined
        } else if sym.section_number == -1 {
            SymbolSection::Absolute
        } else {
            SymbolSection::Undefined
        };

        let (kind, scope) = classify_symbol(sym);

        let symbol_id = obj.add_symbol(Symbol {
            name: sym.name.as_bytes().to_vec(),
            value: sym.value as u64,
            size: 0,
            kind,
            scope,
            weak: false,
            section,
            flags: SymbolFlags::None,
        });

        symbol_map.insert(old_idx, symbol_id);
    }

    // Add relocations to code sections
    for (i, code_section) in coff.code_sections.iter().enumerate() {
        let (_, ref new_relocs) = transformed_code[i];
        let section_id = section_map[&code_section.section_index];

        for reloc in new_relocs {
            if let Some(&symbol_id) = symbol_map.get(&(reloc.symbol_index as usize)) {
                obj.add_relocation(
                    section_id,
                    object::write::Relocation {
                        offset: reloc.offset,
                        symbol: symbol_id,
                        addend: 0,
                        flags: object::RelocationFlags::Coff { typ: reloc.typ },
                    },
                )
                .context("Failed to add relocation")?;
            } else {
                log::warn!(
                    "Relocation references unknown symbol index {}",
                    reloc.symbol_index
                );
            }
        }
    }

    // Add relocations to raw sections
    for raw_section in &coff.raw_sections {
        if raw_section.relocations.is_empty() {
            continue;
        }
        let section_id = section_map[&raw_section.section_index];

        for reloc in &raw_section.relocations {
            if let Some(&symbol_id) = symbol_map.get(&(reloc.symbol_index as usize)) {
                obj.add_relocation(
                    section_id,
                    object::write::Relocation {
                        offset: reloc.offset,
                        symbol: symbol_id,
                        addend: 0,
                        flags: object::RelocationFlags::Coff { typ: reloc.typ },
                    },
                )
                .context("Failed to add raw section relocation")?;
            }
        }
    }

    let mut buffer = Vec::new();
    obj.emit(&mut buffer)
        .context("Failed to emit COFF object file")?;
    Ok(buffer)
}

fn guess_section_kind(name: &str, characteristics: u32) -> SectionKind {
    use object::pe;

    if characteristics & pe::IMAGE_SCN_CNT_CODE != 0 {
        return SectionKind::Text;
    }
    if characteristics & pe::IMAGE_SCN_CNT_UNINITIALIZED_DATA != 0 {
        return SectionKind::UninitializedData;
    }

    match name {
        ".data" => SectionKind::Data,
        ".rdata" | ".rodata" => SectionKind::ReadOnlyData,
        ".bss" => SectionKind::UninitializedData,
        ".pdata" => SectionKind::ReadOnlyData,
        ".xdata" => SectionKind::ReadOnlyData,
        ".drectve" => SectionKind::Linker,
        _ => SectionKind::Data,
    }
}

fn classify_symbol(sym: &CoffSymbol) -> (SymbolKind, SymbolScope) {
    use object::pe;

    let kind = match sym.storage_class {
        pe::IMAGE_SYM_CLASS_EXTERNAL => {
            if sym.section_number > 0 {
                SymbolKind::Text
            } else {
                SymbolKind::Unknown
            }
        }
        pe::IMAGE_SYM_CLASS_STATIC => SymbolKind::Label,
        pe::IMAGE_SYM_CLASS_LABEL => SymbolKind::Label,
        pe::IMAGE_SYM_CLASS_SECTION => SymbolKind::Section,
        pe::IMAGE_SYM_CLASS_FILE => SymbolKind::File,
        _ => SymbolKind::Unknown,
    };

    let scope = match sym.storage_class {
        pe::IMAGE_SYM_CLASS_EXTERNAL => SymbolScope::Linkage,
        pe::IMAGE_SYM_CLASS_STATIC => SymbolScope::Compilation,
        _ => SymbolScope::Unknown,
    };

    (kind, scope)
}
