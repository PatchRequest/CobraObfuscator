use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use cobra_obfuscator::config::ObfuscatorConfig;

/// CobraObfuscator — x86-64 binary obfuscator (COFF .obj and PE .exe/.dll)
#[derive(Parser, Debug)]
#[command(name = "cobra-obfuscator", version, about)]
struct Args {
    /// Input file (.obj, .exe, or .dll)
    #[arg(short, long)]
    input: PathBuf,

    /// Output file
    #[arg(short, long)]
    output: PathBuf,

    /// Number of obfuscation iterations
    #[arg(long, default_value = "1")]
    iterations: u32,

    /// Passes to disable (comma-separated names)
    #[arg(long, value_delimiter = ',')]
    disable: Vec<String>,

    /// RNG seed for reproducible transforms
    #[arg(long)]
    seed: Option<u64>,

    /// Junk insertion density (0.0–1.0)
    #[arg(long, default_value = "0.3")]
    junk_density: f64,

    /// Encrypt strings in .rdata (XOR + startup decryptor)
    #[arg(long)]
    encrypt_strings: bool,

    /// Input format: "auto" (default), "coff", or "pe"
    #[arg(long, default_value = "auto")]
    format: String,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let config = ObfuscatorConfig {
        iterations: args.iterations,
        disabled_passes: args.disable.into_iter().collect::<HashSet<_>>(),
        seed: args.seed,
        junk_density: args.junk_density.clamp(0.0, 1.0),
        encrypt_strings: args.encrypt_strings,
    };

    let input_data =
        std::fs::read(&args.input).with_context(|| format!("Failed to read {:?}", args.input))?;

    log::info!("Input: {:?} ({} bytes)", args.input, input_data.len());

    let is_pe = match args.format.as_str() {
        "pe" => true,
        "coff" => false,
        "auto" | _ => {
            let detected = cobra_obfuscator::is_pe(&input_data);
            log::info!(
                "Auto-detected format: {}",
                if detected { "PE" } else { "COFF" }
            );
            detected
        }
    };

    let (output_data, stats) = if is_pe {
        if cobra_obfuscator::pe::reader::is_go_binary(&input_data) {
            log::info!("Detected Go binary — using in-place obfuscation mode");
            let (data, stats) = cobra_obfuscator::obfuscate_pe_inplace(&input_data, &config)?;
            (data, Some(stats))
        } else {
            let (data, stats) = cobra_obfuscator::obfuscate_pe(&input_data, &config)?;
            (data, Some(stats))
        }
    } else {
        (cobra_obfuscator::obfuscate(&input_data, &config)?, None)
    };

    std::fs::write(&args.output, &output_data)
        .with_context(|| format!("Failed to write {:?}", args.output))?;

    println!(
        "Obfuscated {:?} -> {:?} ({} -> {} bytes, format: {})",
        args.input,
        args.output,
        input_data.len(),
        output_data.len(),
        if is_pe { "PE" } else { "COFF" },
    );

    if let Some(stats) = stats {
        println!("{}", stats);
    }

    Ok(())
}
