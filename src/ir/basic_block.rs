use super::instruction::IrInsn;

/// A basic block: a straight-line sequence of instructions with
/// single entry and single exit (branches only at the end).
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique block ID.
    pub id: u64,
    /// Instructions in this block.
    pub instructions: Vec<IrInsn>,
    /// IDs of successor blocks.
    pub successors: Vec<u64>,
    /// IDs of predecessor blocks.
    pub predecessors: Vec<u64>,
    /// Starting address/offset of this block (for label resolution).
    pub start_address: u64,
    /// Whether this block is an entry point (function start).
    pub is_entry: bool,
}

impl BasicBlock {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            successors: Vec::new(),
            predecessors: Vec::new(),
            start_address: 0,
            is_entry: false,
        }
    }

    /// Check if the last instruction is a branch/jump.
    pub fn has_terminator(&self) -> bool {
        self.instructions
            .last()
            .map(|insn| {
                let flow = insn.instruction.flow_control();
                matches!(
                    flow,
                    iced_x86::FlowControl::UnconditionalBranch
                        | iced_x86::FlowControl::ConditionalBranch
                        | iced_x86::FlowControl::Return
                        | iced_x86::FlowControl::IndirectBranch
                        | iced_x86::FlowControl::IndirectCall
                )
            })
            .unwrap_or(false)
    }
}
