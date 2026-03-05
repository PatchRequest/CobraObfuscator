pub mod coff;
pub mod config;
pub mod encode;
pub mod ir;
pub mod passes;
pub mod pe;
pub mod pipeline;

use anyhow::{Context, Result};
use config::ObfuscatorConfig;
pub use pipeline::ObfuscationStats;

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

/// Compute obfuscation statistics from PE file and obfuscation results.
fn compute_stats(
    pe_file: &pe::types::PeFile,
    obfuscated: &[pipeline::ObfuscatedFunction],
    inplace: bool,
) -> ObfuscationStats {
    let text_section_bytes: u64 = pe_file
        .sections
        .iter()
        .filter(|s| s.is_code())
        .map(|s| s.virtual_size as u64)
        .sum();

    let total_functions = pe_file.functions.len() as u32;
    let runtime_functions = pe_file.functions.iter().filter(|f| f.is_runtime).count() as u32;
    let obfuscated_functions = obfuscated.len() as u32;
    let skipped_functions = total_functions - runtime_functions - obfuscated_functions;

    let obfuscated_bytes: u64 = obfuscated.iter().map(|f| f.original_size as u64).sum();
    let output_code_bytes: u64 = obfuscated.iter().map(|f| f.code.len() as u64).sum();

    ObfuscationStats {
        text_section_bytes,
        total_functions,
        runtime_functions,
        obfuscated_functions,
        skipped_functions,
        obfuscated_bytes,
        output_code_bytes,
        inplace,
    }
}

/// Read a PE binary (.exe/.dll), obfuscate functions, and return the patched PE.
pub fn obfuscate_pe(input: &[u8], config: &ObfuscatorConfig) -> Result<(Vec<u8>, ObfuscationStats)> {
    let pe_file = pe::reader::read_pe(input).context("Failed to read PE input")?;

    log::info!(
        "Parsed PE: image_base=0x{:x}, {} sections, {} functions",
        pe_file.image_base,
        pe_file.sections.len(),
        pe_file.functions.len(),
    );

    // Validate reloc safety
    pe::reloc::validate_reloc_safety(&pe_file.sections)?;

    // Calculate .text expansion layout
    let layout =
        pe::writer::calculate_text_expansion(&pe_file).context("Failed to calculate .text expansion layout")?;

    log::info!(
        "Code section: VA=0x{:x}, raw_offset=0x{:x}",
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

    let stats = compute_stats(&pe_file, &obfuscated, false);
    log::info!("Obfuscated {} functions", obfuscated.len());

    // Write the patched PE
    let output =
        pe::writer::write_pe(&pe_file, &obfuscated, &layout).context("Failed to write PE output")?;

    log::info!("Output PE: {} bytes", output.len());
    Ok((output, stats))
}

/// Read a PE binary, obfuscate functions in-place (no new section), and return the patched PE.
///
/// This mode writes obfuscated code back at the original function addresses, preserving
/// PC-to-metadata mappings such as Go's .gopclntab. Functions that grow too large are skipped.
pub fn obfuscate_pe_inplace(input: &[u8], config: &ObfuscatorConfig) -> Result<(Vec<u8>, ObfuscationStats)> {
    let pe_file = pe::reader::read_pe(input).context("Failed to read PE input")?;

    log::info!(
        "Parsed PE (in-place mode): image_base=0x{:x}, {} sections, {} functions",
        pe_file.image_base, pe_file.sections.len(), pe_file.functions.len(),
    );

    let obfuscated = pipeline::run_pe_pipeline_inplace(
        &pe_file.functions,
        pe_file.image_base,
        config,
    ).context("PE in-place pipeline failed")?;

    let stats = compute_stats(&pe_file, &obfuscated, true);
    log::info!("Obfuscated {} functions (in-place)", obfuscated.len());

    let output = pe::writer::write_pe_inplace(&pe_file, &obfuscated)
        .context("Failed to write PE output (in-place)")?;

    log::info!("Output PE: {} bytes", output.len());
    Ok((output, stats))
}
