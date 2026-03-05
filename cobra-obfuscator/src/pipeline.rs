use anyhow::{Context, Result};
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::coff::types::{CoffObject, CoffRelocation, CodeSection};
use crate::config::ObfuscatorConfig;
use crate::encode::assembler;
use crate::encode::reloc_fixup;
use crate::ir;
use crate::ir::cfg;
use crate::passes;
use crate::passes::pass_trait::{ObfuscationPass, PassContext};
use crate::pe::types::PeFunction;

/// Run the full obfuscation pipeline on a COFF object.
///
/// Returns the transformed code for each code section as (bytes, relocations).
pub fn run_pipeline(
    coff: &CoffObject,
    config: &ObfuscatorConfig,
) -> Result<Vec<(Vec<u8>, Vec<CoffRelocation>)>> {
    let seed = config.seed.unwrap_or_else(|| rand::random());
    let rng = StdRng::seed_from_u64(seed);
    log::info!("Using seed: {}", seed);

    // Gather enabled passes
    let all_passes = passes::default_pass_list();
    let enabled_passes: Vec<Box<dyn ObfuscationPass>> = all_passes
        .into_iter()
        .filter(|p| !config.disabled_passes.contains(p.name()))
        .collect();

    log::info!(
        "Enabled passes: {:?}",
        enabled_passes.iter().map(|p| p.name()).collect::<Vec<_>>()
    );

    let mut ctx = PassContext {
        rng,
        next_block_id: 10000, // start high to avoid collisions
        next_insn_id: 100000,
        junk_density: config.junk_density,
    };

    let mut results = Vec::new();

    for code_section in &coff.code_sections {
        let (code, relocs) =
            process_code_section(code_section, &enabled_passes, &mut ctx, config)?;
        results.push((code, relocs));
    }

    Ok(results)
}

/// Process a single code section through the pipeline.
fn process_code_section(
    section: &CodeSection,
    passes: &[Box<dyn ObfuscationPass>],
    ctx: &mut PassContext,
    config: &ObfuscatorConfig,
) -> Result<(Vec<u8>, Vec<CoffRelocation>)> {
    log::info!("Processing section: {}", section.name);

    // Step 1: Decode section into IR instructions
    let insns = ir::decode_section(section).context("Failed to decode section")?;
    log::info!("  Decoded {} instructions", insns.len());

    // Step 2: Build CFG
    // For .obj files, we typically have one function per section, or we split by symbols.
    // For now, treat the entire section as one function.
    let mut func = cfg::build_cfg(
        insns,
        section.name.clone(),
        0, // symbol index — we'll refine later
    )
    .context("Failed to build CFG")?;

    log::info!(
        "  Built CFG: {} blocks",
        func.blocks.len()
    );

    // Sync IDs with context
    if func.next_block_id > ctx.next_block_id {
        ctx.next_block_id = func.next_block_id;
    }
    if func.next_insn_id > ctx.next_insn_id {
        ctx.next_insn_id = func.next_insn_id;
    }

    // Step 3: Run passes for N iterations
    for iteration in 0..config.iterations {
        log::info!("  Iteration {}/{}", iteration + 1, config.iterations);

        for pass in passes {
            let changed = pass
                .run_on_function(&mut func, ctx)
                .with_context(|| format!("Pass '{}' failed", pass.name()))?;

            if changed {
                log::debug!("    Pass '{}' made changes", pass.name());
            }
        }
    }

    // Sync IDs back
    func.next_block_id = ctx.next_block_id;
    func.next_insn_id = ctx.next_insn_id;

    // Step 4: Encode back to bytes
    let encoded = assembler::encode_function(&func, section.virtual_address)
        .context("Failed to encode function")?;

    // Step 5: Validate relocations
    reloc_fixup::validate_relocations(&encoded.relocations, encoded.code.len())
        .context("Relocation validation failed")?;

    let mut relocs = encoded.relocations;
    reloc_fixup::sort_relocations(&mut relocs);

    log::info!(
        "  Encoded: {} bytes, {} relocations",
        encoded.code.len(),
        relocs.len()
    );

    Ok((encoded.code, relocs))
}

/// Result of obfuscating a single PE function.
#[derive(Debug)]
pub struct ObfuscatedFunction {
    /// Original function RVA start.
    pub original_rva: u32,
    /// Original function size.
    pub original_size: u32,
    /// Obfuscated code bytes.
    pub code: Vec<u8>,
    /// Function name.
    pub name: String,
}

/// Minimum function size to obfuscate (need room for jmp rel32 trampoline).
const MIN_FUNCTION_SIZE: u32 = 5;

/// Check if a PE function contains indirect branches (jump tables, computed gotos).
/// Such functions cannot be safely trampolined because the jump table data in .rdata
/// still points to the original .text addresses which get overwritten.
fn pe_function_has_indirect_branches(func: &PeFunction) -> bool {
    let mut decoder = iced_x86::Decoder::with_ip(
        64,
        &func.code,
        func.start_rva as u64,
        iced_x86::DecoderOptions::NONE,
    );
    while decoder.can_decode() {
        let insn = decoder.decode();
        if insn.flow_control() == iced_x86::FlowControl::IndirectBranch {
            return true;
        }
    }
    false
}

/// Run the obfuscation pipeline on PE functions.
pub fn run_pe_pipeline(
    functions: &[PeFunction],
    image_base: u64,
    cobra_section_rva: u32,
    config: &ObfuscatorConfig,
) -> Result<Vec<ObfuscatedFunction>> {
    let seed = config.seed.unwrap_or_else(|| rand::random());
    let rng = StdRng::seed_from_u64(seed);
    log::info!("PE pipeline using seed: {}", seed);

    let all_passes = passes::default_pass_list();
    let enabled_passes: Vec<Box<dyn ObfuscationPass>> = all_passes
        .into_iter()
        .filter(|p| !config.disabled_passes.contains(p.name()))
        .collect();

    log::info!(
        "Enabled passes: {:?}",
        enabled_passes.iter().map(|p| p.name()).collect::<Vec<_>>()
    );

    let mut ctx = PassContext {
        rng,
        next_block_id: 10000,
        next_insn_id: 100000,
        junk_density: config.junk_density,
    };

    let mut results = Vec::new();
    let mut current_offset: u32 = 0;

    for func in functions {
        if func.is_runtime {
            log::info!("Skipping CRT/runtime function {}", func.name);
            continue;
        }

        if func.size() < MIN_FUNCTION_SIZE {
            log::warn!(
                "Skipping function {} (size {} < {})",
                func.name,
                func.size(),
                MIN_FUNCTION_SIZE
            );
            continue;
        }

        if pe_function_has_indirect_branches(func) {
            log::info!(
                "Skipping function {} — has indirect branches (jump table)",
                func.name
            );
            continue;
        }

        let target_rva = cobra_section_rva + current_offset;
        let target_va = image_base + target_rva as u64;

        match process_pe_function(func, image_base, target_va, &enabled_passes, &mut ctx, config) {
            Ok(code) => {
                log::info!(
                    "  {} -> {} bytes (was {})",
                    func.name,
                    code.len(),
                    func.size()
                );
                let obf = ObfuscatedFunction {
                    original_rva: func.start_rva,
                    original_size: func.size(),
                    code,
                    name: func.name.clone(),
                };
                current_offset += obf.code.len() as u32;
                results.push(obf);
            }
            Err(e) => {
                log::warn!("Skipping function {} due to error: {}", func.name, e);
            }
        }
    }

    Ok(results)
}

/// Run the obfuscation pipeline on PE functions in-place (encode at original VA).
///
/// Unlike `run_pe_pipeline`, this encodes each function at its original virtual address
/// so that PC-to-metadata mappings (e.g. Go's .gopclntab) remain valid.
pub fn run_pe_pipeline_inplace(
    functions: &[PeFunction],
    image_base: u64,
    config: &ObfuscatorConfig,
) -> Result<Vec<ObfuscatedFunction>> {
    let seed = config.seed.unwrap_or_else(|| rand::random());
    let rng = StdRng::seed_from_u64(seed);
    log::info!("PE in-place pipeline using seed: {}", seed);

    // In-place mode: exclude size-increasing passes (CFF, junk-insertion)
    // since the obfuscated code must fit in the original function's space.
    const SIZE_INCREASING_PASSES: &[&str] = &["control-flow-flatten", "junk-insertion"];

    let all_passes = passes::default_pass_list();
    let enabled_passes: Vec<Box<dyn ObfuscationPass>> = all_passes
        .into_iter()
        .filter(|p| !config.disabled_passes.contains(p.name()))
        .filter(|p| !SIZE_INCREASING_PASSES.contains(&p.name()))
        .collect();

    log::info!(
        "In-place enabled passes: {:?}",
        enabled_passes.iter().map(|p| p.name()).collect::<Vec<_>>()
    );

    let mut ctx = PassContext {
        rng,
        next_block_id: 10000,
        next_insn_id: 100000,
        junk_density: config.junk_density,
    };

    let mut results = Vec::new();
    let mut patched = 0u32;
    let mut skipped_size = 0u32;

    for func in functions {
        if func.is_runtime {
            log::info!("Skipping CRT/runtime function {}", func.name);
            continue;
        }

        if func.size() < MIN_FUNCTION_SIZE {
            log::warn!(
                "Skipping function {} (size {} < {})",
                func.name,
                func.size(),
                MIN_FUNCTION_SIZE
            );
            continue;
        }

        if pe_function_has_indirect_branches(func) {
            log::info!(
                "Skipping function {} — has indirect branches",
                func.name
            );
            continue;
        }

        // Encode at the ORIGINAL VA so PCs stay in the original range
        let source_va = image_base + func.start_rva as u64;

        match process_pe_function(func, image_base, source_va, &enabled_passes, &mut ctx, config) {
            Ok(code) => {
                if code.len() > func.size() as usize {
                    log::debug!(
                        "  {} grew ({} > {}), skipping in-place",
                        func.name, code.len(), func.size()
                    );
                    skipped_size += 1;
                    continue;
                }
                log::info!(
                    "  {} -> {} bytes (was {})",
                    func.name,
                    code.len(),
                    func.size()
                );
                patched += 1;
                results.push(ObfuscatedFunction {
                    original_rva: func.start_rva,
                    original_size: func.size(),
                    code,
                    name: func.name.clone(),
                });
            }
            Err(e) => {
                log::warn!("Skipping function {} due to error: {}", func.name, e);
            }
        }
    }

    log::info!(
        "In-place summary: {} patched, {} skipped (grew too large)",
        patched, skipped_size
    );

    Ok(results)
}

/// Process a single PE function through decode → passes → encode.
fn process_pe_function(
    func: &PeFunction,
    image_base: u64,
    target_va: u64,
    passes: &[Box<dyn ObfuscationPass>],
    ctx: &mut PassContext,
    config: &ObfuscatorConfig,
) -> Result<Vec<u8>> {
    // Decode at the function's original VA so branch targets resolve correctly in the CFG.
    // BlockEncoder will re-encode at target_va.
    let source_va = image_base + func.start_rva as u64;
    let insns = ir::decode_raw(&func.code, source_va)
        .context("Failed to decode PE function")?;

    if insns.is_empty() {
        anyhow::bail!("No instructions decoded");
    }

    // Build CFG
    let mut ir_func = cfg::build_cfg(insns, func.name.clone(), 0)
        .context("Failed to build CFG")?;

    // Sync IDs
    if ir_func.next_block_id > ctx.next_block_id {
        ctx.next_block_id = ir_func.next_block_id;
    }
    if ir_func.next_insn_id > ctx.next_insn_id {
        ctx.next_insn_id = ir_func.next_insn_id;
    }

    // Run passes
    for iteration in 0..config.iterations {
        log::debug!("  {} iteration {}/{}", func.name, iteration + 1, config.iterations);
        for pass in passes {
            pass.run_on_function(&mut ir_func, ctx)
                .with_context(|| format!("Pass '{}' failed on {}", pass.name(), func.name))?;
        }
    }

    ir_func.next_block_id = ctx.next_block_id;
    ir_func.next_insn_id = ctx.next_insn_id;

    // Encode at target VA
    let encoded = assembler::encode_function(&ir_func, target_va)
        .context("Failed to encode PE function")?;

    Ok(encoded.code)
}
