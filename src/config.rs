use std::collections::HashSet;

/// Configuration for the obfuscation pipeline.
#[derive(Debug, Clone)]
pub struct ObfuscatorConfig {
    /// Number of times to run the full pass pipeline.
    pub iterations: u32,
    /// Passes to disable by name.
    pub disabled_passes: HashSet<String>,
    /// RNG seed for reproducible transforms.
    pub seed: Option<u64>,
    /// Junk insertion density (0.0–1.0): probability of inserting junk before each instruction.
    pub junk_density: f64,
    /// Encrypt strings in .rdata with XOR + startup decryptor.
    pub encrypt_strings: bool,
    /// PE Fluctuation: encrypt .text at runtime, decrypt on-demand via VEH.
    pub fluctuate: bool,
    /// PE Fluctuation delay in milliseconds before re-encrypting .text.
    pub fluctuation_delay_ms: u32,
}

impl Default for ObfuscatorConfig {
    fn default() -> Self {
        Self {
            iterations: 1,
            disabled_passes: HashSet::new(),
            seed: None,
            junk_density: 0.3,
            encrypt_strings: false,
            fluctuate: false,
            fluctuation_delay_ms: 2000,
        }
    }
}
