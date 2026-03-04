use anyhow::Result;
use iced_x86::{Code, Instruction, Register};
use rand::Rng;
use rand::rngs::StdRng;

use super::pass_trait::{ObfuscationPass, PassContext};
use crate::ir::function::Function;
use crate::ir::instruction::IrInsn;

/// Junk insertion pass: inserts semantically-inert instruction sequences
/// between real instructions.
pub struct JunkInsertion;

impl ObfuscationPass for JunkInsertion {
    fn name(&self) -> &str {
        "junk-insertion"
    }

    fn run_on_function(&self, func: &mut Function, ctx: &mut PassContext) -> Result<bool> {
        let mut changed = false;
        let density = ctx.junk_density;

        for block in &mut func.blocks {
            let mut new_insns = Vec::with_capacity(block.instructions.len() * 2);

            for insn in block.instructions.drain(..) {
                if ctx.rng.gen_bool(density) {
                    let junk = generate_junk(&mut ctx.rng, &mut ctx.next_insn_id);
                    if !junk.is_empty() {
                        changed = true;
                    }
                    new_insns.extend(junk);
                }
                new_insns.push(insn);
            }

            block.instructions = new_insns;
        }

        Ok(changed)
    }
}

fn alloc_id(next_id: &mut u64) -> u64 {
    let id = *next_id;
    *next_id += 1;
    id
}

fn generate_junk(rng: &mut StdRng, next_id: &mut u64) -> Vec<IrInsn> {
    // NOTE: push/pop pairs are intentionally excluded because they temporarily
    // change RSP, which breaks CFF's RSP-relative displacement adjustment.
    let choice = rng.gen_range(0..4);
    match choice {
        0 => {
            let nop = Instruction::with(Code::Nopd);
            vec![IrInsn::synthetic(nop, alloc_id(next_id))]
        }
        1 => {
            // xchg reg, reg (same register = nop)
            let reg = pick_volatile_reg(rng);
            if let Ok(xchg) = Instruction::with2(Code::Xchg_rm64_r64, reg, reg) {
                vec![IrInsn::synthetic(xchg, alloc_id(next_id))]
            } else {
                let nop = Instruction::with(Code::Nopd);
                vec![IrInsn::synthetic(nop, alloc_id(next_id))]
            }
        }
        2 => {
            let nop = Instruction::with(Code::Nopq);
            vec![IrInsn::synthetic(nop, alloc_id(next_id))]
        }
        3 => {
            // lea reg, [reg] (identity operation, doesn't change reg or flags)
            let reg = pick_volatile_reg(rng);
            if let Ok(lea) = Instruction::with2(Code::Lea_r64_m, reg, iced_x86::MemoryOperand::new(reg, Register::None, 1, 0, 8, false, Register::None)) {
                vec![IrInsn::synthetic(lea, alloc_id(next_id))]
            } else {
                let nop = Instruction::with(Code::Nopd);
                vec![IrInsn::synthetic(nop, alloc_id(next_id))]
            }
        }
        _ => unreachable!(),
    }
}

fn pick_volatile_reg(rng: &mut impl Rng) -> Register {
    let regs = [
        Register::RAX,
        Register::RCX,
        Register::RDX,
        Register::R8,
        Register::R9,
        Register::R10,
        Register::R11,
    ];
    regs[rng.gen_range(0..regs.len())]
}
