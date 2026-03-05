/// Represents a parsed COFF object file.
#[derive(Debug, Clone)]
pub struct CoffObject {
    /// Machine type (e.g., IMAGE_FILE_MACHINE_AMD64).
    pub machine: u16,
    /// Code sections (.text) that we will transform.
    pub code_sections: Vec<CodeSection>,
    /// Non-code sections passed through verbatim.
    pub raw_sections: Vec<RawSection>,
    /// Symbol table.
    pub symbols: Vec<CoffSymbol>,
    /// File-level characteristics/flags.
    pub characteristics: u16,
}

/// A code section (.text) we decode and transform.
#[derive(Debug, Clone)]
pub struct CodeSection {
    /// Section name (e.g., ".text").
    pub name: String,
    /// Raw machine code bytes.
    pub data: Vec<u8>,
    /// Virtual address (usually 0 in .obj files).
    pub virtual_address: u64,
    /// Section characteristics (executable, readable, etc).
    pub characteristics: u32,
    /// Relocations targeting this section.
    pub relocations: Vec<CoffRelocation>,
    /// Original section index in the COFF file (for symbol references).
    pub section_index: usize,
    /// Section alignment (power of 2).
    pub alignment: u32,
}

/// A non-code section passed through verbatim.
#[derive(Debug, Clone)]
pub struct RawSection {
    /// Section name.
    pub name: String,
    /// Raw data bytes.
    pub data: Vec<u8>,
    /// Section characteristics.
    pub characteristics: u32,
    /// Relocations (preserved as-is).
    pub relocations: Vec<CoffRelocation>,
    /// Original section index.
    pub section_index: usize,
    /// Section alignment.
    pub alignment: u32,
}

/// A COFF relocation entry.
#[derive(Debug, Clone)]
pub struct CoffRelocation {
    /// Byte offset within the section.
    pub offset: u64,
    /// Index into the symbol table.
    pub symbol_index: u32,
    /// Relocation type (architecture-specific).
    pub typ: u16,
}

/// A COFF symbol table entry.
#[derive(Debug, Clone)]
pub struct CoffSymbol {
    /// Symbol name.
    pub name: String,
    /// Value (offset within section, or other meaning depending on storage class).
    pub value: u32,
    /// Section number (1-based, 0 = external, -1 = absolute, -2 = debug).
    pub section_number: i16,
    /// Symbol type.
    pub typ: u16,
    /// Storage class.
    pub storage_class: u8,
    /// Number of auxiliary symbol records.
    pub number_of_aux_symbols: u8,
    /// Auxiliary data (raw bytes for aux records).
    pub aux_data: Vec<Vec<u8>>,
}
