use std::collections::HashMap;

use super::basic_block::BasicBlock;

/// A function: a named symbol with an ordered list of basic blocks forming a CFG.
#[derive(Debug, Clone)]
pub struct Function {
    /// Symbol name.
    pub name: String,
    /// Symbol index in the COFF symbol table.
    pub symbol_index: usize,
    /// Ordered list of basic blocks (first block is the entry).
    pub blocks: Vec<BasicBlock>,
    /// Next available block ID for creating new blocks.
    pub next_block_id: u64,
    /// Next available instruction ID.
    pub next_insn_id: u64,
}

impl Function {
    pub fn new(name: String, symbol_index: usize) -> Self {
        Self {
            name,
            symbol_index,
            blocks: Vec::new(),
            next_block_id: 0,
            next_insn_id: 0,
        }
    }

    /// Allocate a new unique block ID.
    pub fn alloc_block_id(&mut self) -> u64 {
        let id = self.next_block_id;
        self.next_block_id += 1;
        id
    }

    /// Allocate a new unique instruction ID.
    pub fn alloc_insn_id(&mut self) -> u64 {
        let id = self.next_insn_id;
        self.next_insn_id += 1;
        id
    }

    /// Get the entry block (first block).
    pub fn entry_block(&self) -> Option<&BasicBlock> {
        self.blocks.first()
    }

    /// Get a block by ID.
    pub fn block(&self, id: u64) -> Option<&BasicBlock> {
        self.blocks.iter().find(|b| b.id == id)
    }

    /// Get a mutable block by ID.
    pub fn block_mut(&mut self, id: u64) -> Option<&mut BasicBlock> {
        self.blocks.iter_mut().find(|b| b.id == id)
    }

    /// Flatten all blocks' instructions into a single ordered list.
    pub fn all_instructions(&self) -> Vec<&super::instruction::IrInsn> {
        self.blocks
            .iter()
            .flat_map(|b| b.instructions.iter())
            .collect()
    }

    /// Build a map from block ID to block index for fast lookup.
    pub fn block_index_map(&self) -> HashMap<u64, usize> {
        self.blocks
            .iter()
            .enumerate()
            .map(|(i, b)| (b.id, i))
            .collect()
    }
}
