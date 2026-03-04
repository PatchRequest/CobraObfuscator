pub mod coff;
pub mod config;
pub mod encode;
pub mod ir;
pub mod passes;
pub mod pe;
pub mod pipeline;

use anyhow::{Context, Result};
use config::ObfuscatorConfig;

/// Detect whether the input is a PE binary (MZ header) or a COFF object.
pub fn is_pe(input: &[u8]) -> bool {
    input.len() >= 2 && input[0] == b'M' && input[1] == b'Z'
}

/// Read a COFF object file, run the obfuscation pipeline, and return the transformed bytes.
pub fn obfuscate(input: &[u8], config: &ObfuscatorConfig) -> Result<Vec<u8>> {
    let coff_obj = coff::reader::read_coff(input).context("Failed to read COFF input")?;

    log::info!(
        "Parsed COFF: machine=0x{:x}, {} code sections, {} raw sections, {} symbols",
        coff_obj.machine,
        coff_obj.code_sections.len(),
        coff_obj.raw_sections.len(),
        coff_obj.symbols.len(),
    );

    let transformed = pipeline::run_pipeline(&coff_obj, config)
        .context("Pipeline failed")?;

    let output = coff::writer::write_coff(&coff_obj, &transformed)
        .context("Failed to write COFF output")?;

    log::info!("Output: {} bytes", output.len());
    Ok(output)
}

/// Read a PE binary (.exe/.dll), obfuscate functions, and return the patched PE.
pub fn obfuscate_pe(input: &[u8], config: &ObfuscatorConfig) -> Result<Vec<u8>> {
    let pe_file = pe::reader::read_pe(input).context("Failed to read PE input")?;

    log::info!(
        "Parsed PE: image_base=0x{:x}, {} sections, {} functions",
        pe_file.image_base,
        pe_file.sections.len(),
        pe_file.functions.len(),
    );

    // Validate reloc safety
    pe::reloc::validate_reloc_safety(&pe_file.sections)?;

    // Calculate .cobra section layout
    let layout =
        pe::writer::calculate_cobra_section(&pe_file).context("Failed to calculate .cobra layout")?;

    log::info!(
        ".cobra section: VA=0x{:x}, raw_offset=0x{:x}",
        layout.virtual_address,
        layout.raw_offset,
    );

    // Run the PE obfuscation pipeline
    let obfuscated = pipeline::run_pe_pipeline(
        &pe_file.functions,
        pe_file.image_base,
        layout.virtual_address,
        config,
    )
    .context("PE pipeline failed")?;

    log::info!("Obfuscated {} functions", obfuscated.len());

    // Write the patched PE
    let output =
        pe::writer::write_pe(&pe_file, &obfuscated, &layout).context("Failed to write PE output")?;

    log::info!("Output PE: {} bytes", output.len());
    Ok(output)
}
