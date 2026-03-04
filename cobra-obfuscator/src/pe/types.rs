/// Parsed PE file ready for obfuscation.
#[derive(Debug)]
pub struct PeFile {
    /// Raw PE bytes (mutable during patching).
    pub data: Vec<u8>,
    /// Image base from optional header.
    pub image_base: u64,
    /// Section alignment (in memory).
    pub section_alignment: u32,
    /// File alignment (on disk).
    pub file_alignment: u32,
    /// Parsed section info.
    pub sections: Vec<PeSectionInfo>,
    /// Functions discovered from .pdata.
    pub functions: Vec<PeFunction>,
    /// Offset of NumberOfSections in the COFF header.
    pub number_of_sections_offset: usize,
    /// Offset of SizeOfImage in the optional header.
    pub size_of_image_offset: usize,
    /// Offset of SizeOfCode in the optional header.
    pub size_of_code_offset: usize,
    /// Offset of CheckSum in the optional header.
    pub checksum_offset: usize,
    /// Current number of sections.
    pub number_of_sections: u16,
    /// Size of all headers (aligned).
    pub size_of_headers: u32,
    /// Entry point RVA.
    pub entry_point_rva: u32,
}

/// A function discovered from .pdata RUNTIME_FUNCTION entries.
#[derive(Debug, Clone)]
pub struct PeFunction {
    /// Function name (from exports or synthesized).
    pub name: String,
    /// RVA of function start.
    pub start_rva: u32,
    /// RVA of function end.
    pub end_rva: u32,
    /// Raw code bytes of the function.
    pub code: Vec<u8>,
    /// Index into the .pdata table.
    pub pdata_index: usize,
    /// Whether this is likely a CRT/runtime function (should be skipped).
    pub is_runtime: bool,
}

impl PeFunction {
    pub fn size(&self) -> u32 {
        self.end_rva - self.start_rva
    }
}

/// Info about a PE section.
#[derive(Debug, Clone)]
pub struct PeSectionInfo {
    /// Section name (e.g. ".text").
    pub name: String,
    /// Virtual address (RVA).
    pub virtual_address: u32,
    /// Virtual size.
    pub virtual_size: u32,
    /// Offset in the file.
    pub raw_offset: u32,
    /// Size in the file.
    pub raw_size: u32,
    /// Section characteristics flags.
    pub characteristics: u32,
    /// Offset of this section's header in the file.
    pub header_offset: usize,
}

impl PeSectionInfo {
    /// Check if section contains executable code.
    pub fn is_code(&self) -> bool {
        // IMAGE_SCN_CNT_CODE (0x20) or IMAGE_SCN_MEM_EXECUTE (0x20000000)
        self.characteristics & 0x20000020 != 0
    }

    /// Check if an RVA falls within this section.
    pub fn contains_rva(&self, rva: u32) -> bool {
        rva >= self.virtual_address && rva < self.virtual_address + self.virtual_size
    }

    /// Convert an RVA to a file offset.
    pub fn rva_to_offset(&self, rva: u32) -> Option<usize> {
        if self.contains_rva(rva) {
            Some((self.raw_offset + (rva - self.virtual_address)) as usize)
        } else {
            None
        }
    }
}
