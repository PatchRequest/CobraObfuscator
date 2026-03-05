use anyhow::{Context, Result};
use object::pe;
use object::read::coff::{CoffHeader, ImageSymbol};
use object::LittleEndian as LE;

use super::types::*;

/// Parse a COFF `.obj` file from raw bytes.
pub fn read_coff(data: &[u8]) -> Result<CoffObject> {
    let mut offset = 0u64;
    let header = pe::ImageFileHeader::parse(data, &mut offset).context("Failed to parse COFF header")?;
    let machine = header.machine.get(LE);
    let characteristics = header.characteristics.get(LE);

    let sections_table = header
        .sections(data, 0)
        .context("Failed to read section headers")?;

    let symbol_table = header.symbols(data).context("Failed to read symbol table")?;

    let mut code_sections = Vec::new();
    let mut raw_sections = Vec::new();

    for (section_idx, section_header) in sections_table.iter().enumerate() {
        let name = {
            if let Ok(name_bytes) = section_header.name(symbol_table.strings()) {
                String::from_utf8_lossy(name_bytes).into_owned()
            } else {
                let raw_name = section_header.raw_name();
                let end = raw_name.iter().position(|&b| b == 0).unwrap_or(raw_name.len());
                String::from_utf8_lossy(&raw_name[..end]).into_owned()
            }
        };

        let section_data = section_header
            .coff_data(data)
            .unwrap_or(&[])
            .to_vec();

        let chars = section_header.characteristics.get(LE);

        let mut relocations = Vec::new();
        if let Ok(reloc_iter) = section_header.coff_relocations(data) {
            for reloc in reloc_iter {
                relocations.push(CoffRelocation {
                    offset: reloc.virtual_address.get(LE) as u64,
                    symbol_index: reloc.symbol_table_index.get(LE),
                    typ: reloc.typ.get(LE),
                });
            }
        }

        let alignment = alignment_from_characteristics(chars);

        let is_code = (chars & pe::IMAGE_SCN_CNT_CODE != 0)
            || (chars & pe::IMAGE_SCN_MEM_EXECUTE != 0);

        let virtual_address = section_header.virtual_address.get(LE) as u64;

        if is_code && !section_data.is_empty() {
            code_sections.push(CodeSection {
                name,
                data: section_data,
                virtual_address,
                characteristics: chars,
                relocations,
                section_index: section_idx,
                alignment,
            });
        } else {
            raw_sections.push(RawSection {
                name,
                data: section_data,
                characteristics: chars,
                relocations,
                section_index: section_idx,
                alignment,
            });
        }
    }

    let symbols = read_symbols(data, header)?;

    Ok(CoffObject {
        machine,
        code_sections,
        raw_sections,
        symbols,
        characteristics,
    })
}

/// Read symbol table from raw COFF data.
fn read_symbols(data: &[u8], header: &pe::ImageFileHeader) -> Result<Vec<CoffSymbol>> {
    let mut symbols = Vec::new();

    let symbol_table = header.symbols(data).context("Failed to read symbol table")?;
    let strings = symbol_table.strings();

    let mut i = 0;
    while i < symbol_table.len() {
        let index = object::SymbolIndex(i);
        let symbol = symbol_table
            .symbol(index)
            .context("Failed to read symbol")?;

        let name = match symbol.name(strings) {
            Ok(name) => String::from_utf8_lossy(name).into_owned(),
            Err(_) => format!("__unknown_sym_{}", i),
        };

        let num_aux = symbol.number_of_aux_symbols;
        let mut aux_data = Vec::new();
        for j in 1..=num_aux as usize {
            if i + j < symbol_table.len() {
                // Aux records are 18-byte raw entries; store empty for now
                aux_data.push(vec![0u8; 18]);
            }
        }

        symbols.push(CoffSymbol {
            name,
            value: symbol.value.get(LE),
            section_number: symbol.section_number.get(LE) as i16,
            typ: symbol.typ.get(LE),
            storage_class: symbol.storage_class,
            number_of_aux_symbols: num_aux,
            aux_data,
        });

        i += 1 + num_aux as usize;
    }

    Ok(symbols)
}

/// Extract alignment from COFF section characteristics.
fn alignment_from_characteristics(chars: u32) -> u32 {
    let align_field = (chars & pe::IMAGE_SCN_ALIGN_MASK) >> 20;
    if align_field == 0 {
        1
    } else {
        1u32 << (align_field - 1)
    }
}
