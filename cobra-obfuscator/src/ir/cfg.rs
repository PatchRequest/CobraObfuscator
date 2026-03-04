use std::collections::{HashMap, HashSet};

use anyhow::Result;
use iced_x86::FlowControl;

use super::basic_block::BasicBlock;
use super::function::Function;
use super::instruction::IrInsn;

/// Build a CFG from a flat list of IR instructions.
///
/// Leader detection: an instruction is a leader if:
/// 1. It is the first instruction.
/// 2. It is the target of a branch.
/// 3. It immediately follows a branch or terminator.
///
/// Returns a Function with basic blocks and edges.
pub fn build_cfg(
    insns: Vec<IrInsn>,
    name: String,
    symbol_index: usize,
) -> Result<Function> {
    if insns.is_empty() {
        let func = Function::new(name, symbol_index);
        return Ok(func);
    }

    // Step 1: Identify leaders (addresses that start basic blocks)
    let mut leaders: HashSet<u64> = HashSet::new();
    let mut addr_to_idx: HashMap<u64, usize> = HashMap::new();

    for (i, insn) in insns.iter().enumerate() {
        let ip = insn.instruction.ip();
        addr_to_idx.insert(ip, i);

        if i == 0 {
            leaders.insert(ip);
        }

        let flow = insn.instruction.flow_control();
        match flow {
            FlowControl::ConditionalBranch => {
                // Target is a leader
                let target = insn.instruction.near_branch_target();
                leaders.insert(target);
                // Fallthrough is a leader
                if i + 1 < insns.len() {
                    leaders.insert(insns[i + 1].instruction.ip());
                }
            }
            FlowControl::UnconditionalBranch => {
                let target = insn.instruction.near_branch_target();
                leaders.insert(target);
                // Next instruction is a leader (if reachable via other paths)
                if i + 1 < insns.len() {
                    leaders.insert(insns[i + 1].instruction.ip());
                }
            }
            FlowControl::Return | FlowControl::IndirectBranch => {
                if i + 1 < insns.len() {
                    leaders.insert(insns[i + 1].instruction.ip());
                }
            }
            _ => {}
        }
    }

    // Step 2: Partition instructions into basic blocks
    let mut func = Function::new(name, symbol_index);
    let mut current_block = BasicBlock::new(func.alloc_block_id());
    current_block.is_entry = true;

    // Map from instruction address to block ID for edge creation
    let mut addr_to_block: HashMap<u64, u64> = HashMap::new();
    let mut blocks: Vec<BasicBlock> = Vec::new();

    for (i, insn) in insns.into_iter().enumerate() {
        let ip = insn.instruction.ip();

        if i > 0 && leaders.contains(&ip) {
            // Start a new block — save the current one
            blocks.push(current_block);
            current_block = BasicBlock::new(func.alloc_block_id());
        }

        if current_block.instructions.is_empty() {
            current_block.start_address = ip;
            addr_to_block.insert(ip, current_block.id);
        }

        // Track the max insn ID
        if insn.id >= func.next_insn_id {
            func.next_insn_id = insn.id + 1;
        }

        current_block.instructions.push(insn);
    }

    // Don't forget the last block
    if !current_block.instructions.is_empty() {
        blocks.push(current_block);
    }

    // Step 3: Create edges
    for i in 0..blocks.len() {
        let last_insn = match blocks[i].instructions.last() {
            Some(insn) => insn,
            None => continue,
        };

        let flow = last_insn.instruction.flow_control();
        let _block_id = blocks[i].id;

        match flow {
            FlowControl::ConditionalBranch => {
                let target = last_insn.instruction.near_branch_target();
                // Add branch target edge
                if let Some(&target_block_id) = addr_to_block.get(&target) {
                    blocks[i].successors.push(target_block_id);
                }
                // Add fallthrough edge
                if i + 1 < blocks.len() {
                    let fallthrough_id = blocks[i + 1].id;
                    blocks[i].successors.push(fallthrough_id);
                }
            }
            FlowControl::UnconditionalBranch => {
                let target = last_insn.instruction.near_branch_target();
                if let Some(&target_block_id) = addr_to_block.get(&target) {
                    blocks[i].successors.push(target_block_id);
                }
            }
            FlowControl::Return | FlowControl::IndirectBranch => {
                // No successors
            }
            _ => {
                // Fallthrough to next block
                if i + 1 < blocks.len() {
                    let fallthrough_id = blocks[i + 1].id;
                    blocks[i].successors.push(fallthrough_id);
                }
            }
        }
    }

    // Step 4: Build predecessor edges from successor edges
    let successor_map: Vec<(u64, Vec<u64>)> = blocks
        .iter()
        .map(|b| (b.id, b.successors.clone()))
        .collect();

    for (block_id, successors) in &successor_map {
        for succ_id in successors {
            if let Some(succ_block) = blocks.iter_mut().find(|b| b.id == *succ_id) {
                succ_block.predecessors.push(*block_id);
            }
        }
    }

    func.blocks = blocks;
    Ok(func)
}
