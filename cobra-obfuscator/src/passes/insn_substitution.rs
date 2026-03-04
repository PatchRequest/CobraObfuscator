use anyhow::Result;
use iced_x86::{Code, Instruction, Register};
use rand::Rng;

use super::pass_trait::{ObfuscationPass, PassContext};
use crate::ir::function::Function;

/// Instruction substitution pass: replaces instructions with semantically
/// equivalent alternatives. Only operates on instructions without relocations.
pub struct InsnSubstitution;

impl ObfuscationPass for InsnSubstitution {
    fn name(&self) -> &str {
        "insn-substitution"
    }

    fn run_on_function(&self, func: &mut Function, ctx: &mut PassContext) -> Result<bool> {
        let mut changed = false;

        for block in &mut func.blocks {
            for insn in &mut block.instructions {
                if insn.has_relocation() {
                    continue;
                }

                if let Some(mut new_instr) = try_substitute(&insn.instruction, &mut ctx.rng) {
                    // Preserve the original instruction's IP so BlockEncoder can
                    // resolve branches targeting this instruction.
                    new_instr.set_ip(insn.instruction.ip());
                    insn.instruction = new_instr;
                    changed = true;
                }
            }
        }

        Ok(changed)
    }
}

fn try_substitute(instr: &Instruction, rng: &mut impl Rng) -> Option<Instruction> {
    if !rng.gen_bool(0.5) {
        return None;
    }

    let code = instr.code();

    // NOTE: We intentionally do NOT substitute mov↔xor for zeroing because
    // xor clobbers FLAGS while mov does not, and we cannot know if a later
    // instruction depends on FLAGS from before the mov.

    // xor reg, reg → mov reg, 0 (safe: mov doesn't clobber FLAGS, so this
    // only relaxes FLAG constraints)
    if matches!(code, Code::Xor_rm32_r32 | Code::Xor_rm64_r64)
        && instr.op0_register() == instr.op1_register()
        && instr.op0_register() != Register::None
    {
        let reg32 = to_reg32(instr.op0_register())?;
        return Instruction::with2(Code::Mov_rm32_imm32, reg32, 0u32).ok();
    }

    // inc reg → add reg, 1 (safe: add sets CF, inc doesn't, but add is
    // a superset of inc's flag behavior)
    if matches!(code, Code::Inc_rm64 | Code::Inc_rm32) {
        let reg = instr.op0_register();
        if reg != Register::None {
            let add_code = if is_reg64(reg) { Code::Add_rm64_imm8 } else { Code::Add_rm32_imm8 };
            return Instruction::with2(add_code, reg, 1i32).ok();
        }
    }

    // dec reg → sub reg, 1 (safe: same reasoning as inc→add)
    if matches!(code, Code::Dec_rm64 | Code::Dec_rm32) {
        let reg = instr.op0_register();
        if reg != Register::None {
            let sub_code = if is_reg64(reg) { Code::Sub_rm64_imm8 } else { Code::Sub_rm32_imm8 };
            return Instruction::with2(sub_code, reg, 1i32).ok();
        }
    }

    None
}


fn is_reg64(reg: Register) -> bool {
    matches!(
        reg,
        Register::RAX | Register::RBX | Register::RCX | Register::RDX
        | Register::RSI | Register::RDI | Register::RSP | Register::RBP
        | Register::R8 | Register::R9 | Register::R10 | Register::R11
        | Register::R12 | Register::R13 | Register::R14 | Register::R15
    )
}

fn to_reg32(reg: Register) -> Option<Register> {
    Some(match reg {
        Register::RAX | Register::EAX => Register::EAX,
        Register::RBX | Register::EBX => Register::EBX,
        Register::RCX | Register::ECX => Register::ECX,
        Register::RDX | Register::EDX => Register::EDX,
        Register::RSI | Register::ESI => Register::ESI,
        Register::RDI | Register::EDI => Register::EDI,
        Register::RSP | Register::ESP => Register::ESP,
        Register::RBP | Register::EBP => Register::EBP,
        Register::R8 | Register::R8D => Register::R8D,
        Register::R9 | Register::R9D => Register::R9D,
        Register::R10 | Register::R10D => Register::R10D,
        Register::R11 | Register::R11D => Register::R11D,
        Register::R12 | Register::R12D => Register::R12D,
        Register::R13 | Register::R13D => Register::R13D,
        Register::R14 | Register::R14D => Register::R14D,
        Register::R15 | Register::R15D => Register::R15D,
        _ => return None,
    })
}
