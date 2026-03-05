use anyhow::Result;
use iced_x86::{Code, Instruction, Register};
use rand::Rng;

use super::pass_trait::{ObfuscationPass, PassContext};
use crate::ir::basic_block::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::IrInsn;

/// Dead code insertion pass: adds unreachable blocks with realistic-looking
/// instructions, connected via opaque predicates that never branch.
///
/// Technique: prepend `cmp rsp, 0; je dead_block` to a block.
/// RSP is never zero so the je is never taken. This clobbers FLAGS, which
/// is safe because block entry points in compiler-generated code don't
/// depend on FLAGS from predecessor blocks.
pub struct DeadCodeInsertion;

impl ObfuscationPass for DeadCodeInsertion {
    fn name(&self) -> &str {
        "dead-code"
    }

    fn run_on_function(&self, func: &mut Function, ctx: &mut PassContext) -> Result<bool> {
        if func.blocks.len() < 2 {
            return Ok(false);
        }

        let mut changed = false;
        let mut dead_blocks: Vec<BasicBlock> = Vec::new();
        let num_blocks = func.blocks.len();

        for i in 0..num_blocks {
            // 30% chance to add dead code before each non-entry block
            if i == 0 || !ctx.rng.gen_bool(0.3) {
                continue;
            }

            // Build the dead block first so we know its first instruction's IP
            let dead_block_id = ctx.alloc_block_id();
            let mut dead_block = BasicBlock::new(dead_block_id);

            // Assign a unique synthetic IP to the dead block's first instruction
            // so BlockEncoder can resolve the je target.
            // Use a high address range that won't collide with real code.
            let dead_block_ip = 0x7FFE_0000_0000u64 + (dead_block_id as u64) * 0x1000;

            // Fill dead block with realistic-looking instructions
            let num_dead_insns = ctx.rng.gen_range(3..8);
            for j in 0..num_dead_insns {
                let mut dead_instr = generate_realistic_instruction(&mut ctx.rng);
                dead_instr.set_ip(dead_block_ip + j as u64 * 8);
                dead_block
                    .instructions
                    .push(IrInsn::synthetic(dead_instr, ctx.alloc_insn_id()));
            }

            // End dead block with int3
            let mut trap = Instruction::with(Code::Int3);
            trap.set_ip(dead_block_ip + num_dead_insns as u64 * 8);
            dead_block
                .instructions
                .push(IrInsn::synthetic(trap, ctx.alloc_insn_id()));

            // Build opaque predicate sequence:
            //   cmp rsp, 0          ; RSP is never 0 → ZF=0
            //   je dead_block       ; never taken (ZF=0)
            let cmp = Instruction::with2(Code::Cmp_rm64_imm8, Register::RSP, 0i32).unwrap();
            let je = Instruction::with_branch(Code::Je_rel32_64, dead_block_ip).unwrap();

            let predicate_insns = vec![
                IrInsn::synthetic(cmp, ctx.alloc_insn_id()),
                IrInsn::synthetic(je, ctx.alloc_insn_id()),
            ];

            // Prepend to the block, preserving the original first instruction's IP
            // so that CFF and other passes can still match branch targets to this block.
            let original_first_ip = func.blocks[i]
                .instructions
                .first()
                .map(|insn| insn.instruction.ip())
                .unwrap_or(0);

            let mut existing: Vec<IrInsn> = func.blocks[i].instructions.drain(..).collect();
            func.blocks[i].instructions = predicate_insns;
            if original_first_ip != 0 {
                // Transfer the block's entry IP to the opaque predicate so branches
                // targeting this block land on the cmp instruction.
                if let Some(first) = func.blocks[i].instructions.first_mut() {
                    first.instruction.set_ip(original_first_ip);
                }
                // Clear the IP on the original first instruction to avoid duplicate IPs
                // (BlockEncoder requires unique IPs for branch resolution).
                if let Some(orig_first) = existing.first_mut() {
                    orig_first.instruction.set_ip(0);
                }
            }
            func.blocks[i].instructions.extend(existing);

            dead_blocks.push(dead_block);
            changed = true;
        }

        // Append dead blocks at the end
        func.blocks.extend(dead_blocks);

        Ok(changed)
    }
}

fn generate_realistic_instruction(rng: &mut impl Rng) -> Instruction {
    let choice = rng.gen_range(0..6);
    match choice {
        0 => {
            let imm = rng.gen_range(0..256) as u32;
            Instruction::with2(Code::Mov_rm32_imm32, Register::EAX, imm).unwrap()
        }
        1 => {
            let imm = rng.gen_range(0..64) as u32;
            Instruction::with2(Code::Add_rm64_imm8, Register::RCX, imm).unwrap()
        }
        2 => Instruction::with2(Code::Xor_rm64_r64, Register::RDX, Register::RDX).unwrap(),
        3 => {
            let imm = rng.gen_range(0..128) as u32;
            Instruction::with2(Code::Sub_rm64_imm8, Register::R8, imm).unwrap()
        }
        4 => Instruction::with2(Code::Test_rm64_r64, Register::R9, Register::R9).unwrap(),
        5 => Instruction::with(Code::Nopd),
        _ => unreachable!(),
    }
}
