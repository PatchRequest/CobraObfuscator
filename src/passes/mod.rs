pub mod pass_trait;
pub mod insn_substitution;
pub mod junk_insertion;
pub mod dead_code;
pub mod control_flow_flatten;

use pass_trait::ObfuscationPass;

/// Returns the default ordered list of passes.
pub fn default_pass_list() -> Vec<Box<dyn ObfuscationPass>> {
    vec![
        Box::new(insn_substitution::InsnSubstitution),
        Box::new(junk_insertion::JunkInsertion),
        Box::new(dead_code::DeadCodeInsertion),
        Box::new(control_flow_flatten::ControlFlowFlatten),
    ]
}
