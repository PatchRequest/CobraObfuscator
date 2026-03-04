use anyhow::Result;
use rand::rngs::StdRng;

use crate::ir::function::Function;

/// Context shared across passes during a pipeline run.
pub struct PassContext {
    /// Seeded RNG for reproducible transforms.
    pub rng: StdRng,
    /// Next available block ID (global across all functions).
    pub next_block_id: u64,
    /// Next available instruction ID.
    pub next_insn_id: u64,
    /// Junk insertion density.
    pub junk_density: f64,
}

impl PassContext {
    pub fn alloc_block_id(&mut self) -> u64 {
        let id = self.next_block_id;
        self.next_block_id += 1;
        id
    }

    pub fn alloc_insn_id(&mut self) -> u64 {
        let id = self.next_insn_id;
        self.next_insn_id += 1;
        id
    }
}

/// Trait for obfuscation passes.
pub trait ObfuscationPass {
    /// Human-readable name of this pass.
    fn name(&self) -> &str;

    /// Run the pass on a single function. Returns true if any changes were made.
    fn run_on_function(&self, func: &mut Function, ctx: &mut PassContext) -> Result<bool>;
}
