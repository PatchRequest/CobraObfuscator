use anyhow::Result;
use iced_x86::{Code, Instruction, Register};
use rand::seq::SliceRandom;
use rand::Rng;

use super::pass_trait::{ObfuscationPass, PassContext};
use crate::ir::basic_block::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::IrInsn;

/// Control flow flattening pass: replaces the natural CFG with a dispatcher-based
/// structure where each block sets a "state" variable and jumps to a central dispatcher
/// that routes to the correct next block via `cmp`/`je` chains.
///
/// Uses R15 as the state register. R15 is callee-saved, so we save/restore it
/// using `push r15; sub rsp, 8` (16 bytes total to preserve 16-byte alignment)
/// and adjust all RSP-relative memory operands by +16 to compensate.
pub struct ControlFlowFlatten;

impl ObfuscationPass for ControlFlowFlatten {
    fn name(&self) -> &str {
        "control-flow-flatten"
    }

    fn run_on_function(&self, func: &mut Function, ctx: &mut PassContext) -> Result<bool> {
        // Need at least 3 blocks to make flattening worthwhile
        if func.blocks.len() < 3 {
            return Ok(false);
        }

        // Only flatten with 40% probability to keep some functions natural
        if !ctx.rng.gen_bool(0.4) {
            return Ok(false);
        }

        // Check if the function uses R15 — we need it as the state register.
        if function_uses_register(func, Register::R15) {
            log::debug!("Skipping CFF for {} — function uses R15", func.name);
            return Ok(false);
        }

        // Skip functions with indirect branches (jump tables, computed gotos).
        if function_has_indirect_branches(func) {
            log::debug!("Skipping CFF for {} — function has indirect branches", func.name);
            return Ok(false);
        }

        let state_reg = Register::R15;
        let state_reg32 = Register::R15D;
        let num_blocks = func.blocks.len();

        // Detect if the function uses RBP as a frame pointer (push rbp; mov rbp, rsp).
        let has_frame_pointer = detect_frame_pointer(func);

        // Assign random state numbers to each block
        let mut state_numbers: Vec<u32> = (0..num_blocks as u32).collect();
        state_numbers.shuffle(&mut ctx.rng);

        let _dispatcher_id = ctx.alloc_block_id();

        struct BlockMapping {
            state: u32,
            insns: Vec<IrInsn>,
            has_terminator: bool,
            successors: Vec<u64>,
            #[allow(dead_code)]
            original_id: u64,
            /// IP of the first instruction before CFF reassigns it.
            original_first_ip: u64,
        }

        let mut block_mappings: Vec<BlockMapping> = Vec::new();

        for (i, block) in func.blocks.iter().enumerate() {
            let mut insns = block.instructions.clone();

            // Adjust RSP-relative memory operands by +16 to account for
            // our preamble (push r15 + sub rsp, 8 = 16 bytes on stack).
            // Also adjust RBP-relative operands if the function uses a frame pointer,
            // since RBP will also be 16 bytes lower than expected.
            for insn in &mut insns {
                adjust_stack_displacements(&mut insn.instruction, has_frame_pointer);
            }

            let original_first_ip = insns.first().map(|i| i.instruction.ip()).unwrap_or(0);
            block_mappings.push(BlockMapping {
                state: state_numbers[i],
                insns,
                has_terminator: block.has_terminator(),
                successors: block.successors.clone(),
                original_id: block.id,
                original_first_ip,
            });
        }

        // Build a block_id->index map for successor lookup
        let block_id_to_index: std::collections::HashMap<u64, usize> = func
            .blocks
            .iter()
            .enumerate()
            .map(|(i, b)| (b.id, i))
            .collect();

        // Layout:
        //   push r15
        //   sub rsp, 8           ; alignment padding (total 16 bytes, preserves alignment)
        //   mov r15d, STATE_0
        //   DISPATCHER:
        //     cmp r15d, STATE_i; je block_i  (for each block)
        //     int3
        //   BLOCK_0: insns; mov r15d, NEXT; jmp DISPATCHER
        //   BLOCK_1: insns; mov r15d, NEXT; jmp DISPATCHER
        //   ...
        //   (before each ret: add rsp, 8; pop r15)

        let mut all_insns: Vec<IrInsn> = Vec::new();

        // Preamble: save R15 with alignment-preserving sequence
        let push_r15 = Instruction::with1(Code::Push_r64, state_reg).unwrap();
        all_insns.push(IrInsn::synthetic(push_r15, ctx.alloc_insn_id()));

        let sub_rsp_8 = Instruction::with2(Code::Sub_rm64_imm8, Register::RSP, 8i32).unwrap();
        all_insns.push(IrInsn::synthetic(sub_rsp_8, ctx.alloc_insn_id()));

        let initial_state = state_numbers[0];
        let mov_state = Instruction::with2(Code::Mov_rm32_imm32, state_reg32, initial_state).unwrap();
        all_insns.push(IrInsn::synthetic(mov_state, ctx.alloc_insn_id()));

        // Calculate approximate layout for branch target IPs.
        // These IPs only need to be unique and consistent between je targets
        // and the instructions they reference — BlockEncoder resolves the rest.
        let dummy_base = 0x1000_0000u64;
        let mut current_ip = dummy_base;

        current_ip += 2;  // push r15
        current_ip += 4;  // sub rsp, 8
        current_ip += 6;  // mov r15d, imm32

        let dispatcher_ip = current_ip;

        // Dispatcher: num_blocks * (cmp r15d,imm32 + je rel32) + int3
        let dispatcher_size = (num_blocks as u64) * 12 + 1;
        let blocks_start_ip = dispatcher_ip + dispatcher_size;

        // Calculate block start IPs (approximate, only needs to be unique)
        let mut block_start_ips: Vec<u64> = Vec::new();
        let mut ip = blocks_start_ip;
        for mapping in &block_mappings {
            block_start_ips.push(ip);
            let block_size = (mapping.insns.len() as u64) * 5 + 20;
            ip += block_size;
        }

        // Emit dispatcher: cmp/je pairs
        let mut dispatcher_insns: Vec<IrInsn> = Vec::new();
        for (i, mapping) in block_mappings.iter().enumerate() {
            let mut cmp = Instruction::with2(Code::Cmp_rm32_imm32, state_reg32, mapping.state).unwrap();
            if i == 0 {
                cmp.set_ip(dispatcher_ip);
            }
            dispatcher_insns.push(IrInsn::synthetic(cmp, ctx.alloc_insn_id()));

            let target = block_start_ips[i];
            let je = Instruction::with_branch(Code::Je_rel32_64, target).unwrap();
            dispatcher_insns.push(IrInsn::synthetic(je, ctx.alloc_insn_id()));
        }
        let int3 = Instruction::with(Code::Int3);
        dispatcher_insns.push(IrInsn::synthetic(int3, ctx.alloc_insn_id()));

        all_insns.extend(dispatcher_insns);

        // Emit each block's instructions, replacing terminators with state transitions.
        for (i, mapping) in block_mappings.iter().enumerate() {
            let block_ip = block_start_ips[i];
            let mut need_block_ip = true;

            // Helper: push an instruction, stamping block_ip on the first one
            let emit = |all: &mut Vec<IrInsn>, mut ir: IrInsn, need_ip: &mut bool| {
                if *need_ip {
                    ir.instruction.set_ip(block_ip);
                    *need_ip = false;
                }
                all.push(ir);
            };

            let num_insns = mapping.insns.len();
            for (insn_idx, insn) in mapping.insns.iter().enumerate() {
                let flow = insn.instruction.flow_control();
                let is_last = insn_idx == num_insns - 1;

                // Only transform the block's actual terminator (last instruction).
                // Mid-block branches (e.g., opaque predicates from dead-code pass)
                // must be kept as-is to avoid dropping subsequent instructions.
                // Returns always need CFF cleanup regardless of position.
                match flow {
                    iced_x86::FlowControl::Return => {
                        // Restore stack and R15 before returning:
                        //   add rsp, 8   ; remove alignment padding
                        //   pop r15      ; restore R15
                        //   ret          ; original return
                        let add_rsp_8 = Instruction::with2(
                            Code::Add_rm64_imm8, Register::RSP, 8i32,
                        ).unwrap();
                        emit(&mut all_insns, IrInsn::synthetic(add_rsp_8, ctx.alloc_insn_id()), &mut need_block_ip);

                        let pop_r15 = Instruction::with1(Code::Pop_r64, state_reg).unwrap();
                        all_insns.push(IrInsn::synthetic(pop_r15, ctx.alloc_insn_id()));

                        all_insns.push(insn.clone());
                    }
                    iced_x86::FlowControl::UnconditionalBranch if is_last => {
                        let target_addr = insn.instruction.near_branch_target();
                        let target_block_idx = block_mappings.iter().position(|m| {
                            m.insns
                                .first()
                                .map(|first| first.instruction.ip() == target_addr)
                                .unwrap_or(false)
                        });

                        if let Some(idx) = target_block_idx {
                            let next_state = block_mappings[idx].state;
                            let mov = Instruction::with2(
                                Code::Mov_rm32_imm32, state_reg32, next_state,
                            ).unwrap();
                            emit(&mut all_insns, IrInsn::synthetic(mov, ctx.alloc_insn_id()), &mut need_block_ip);

                            let jmp = Instruction::with_branch(
                                Code::Jmp_rel32_64, dispatcher_ip,
                            ).unwrap();
                            all_insns.push(IrInsn::synthetic(jmp, ctx.alloc_insn_id()));
                        } else {
                            // External jump (tail call, etc.) — restore stack
                            // and R15 before jumping, since we won't return here.
                            let add_rsp_8 = Instruction::with2(
                                Code::Add_rm64_imm8, Register::RSP, 8i32,
                            ).unwrap();
                            emit(&mut all_insns, IrInsn::synthetic(add_rsp_8, ctx.alloc_insn_id()), &mut need_block_ip);

                            let pop_r15 = Instruction::with1(Code::Pop_r64, state_reg).unwrap();
                            all_insns.push(IrInsn::synthetic(pop_r15, ctx.alloc_insn_id()));

                            all_insns.push(insn.clone());
                        }
                    }
                    iced_x86::FlowControl::ConditionalBranch if is_last => {
                        let target_addr = insn.instruction.near_branch_target();
                        let target_block_idx = block_mappings.iter().position(|m| {
                            m.insns
                                .first()
                                .map(|first| first.instruction.ip() == target_addr)
                                .unwrap_or(false)
                        });
                        let fallthrough_idx = if i + 1 < block_mappings.len() {
                            Some(i + 1)
                        } else {
                            None
                        };

                        if let (Some(taken_idx), Some(fall_idx)) =
                            (target_block_idx, fallthrough_idx)
                        {
                            let taken_state = block_mappings[taken_idx].state;
                            let fall_state = block_mappings[fall_idx].state;

                            // mov r15d, taken_state (speculatively set taken path)
                            let mov_taken = Instruction::with2(
                                Code::Mov_rm32_imm32, state_reg32, taken_state,
                            ).unwrap();
                            emit(&mut all_insns, IrInsn::synthetic(mov_taken, ctx.alloc_insn_id()), &mut need_block_ip);

                            // jcc skip_fallback (if condition true, skip the fallback mov)
                            let skip_ip = 0x7FFC_0000_0000u64 + ctx.next_insn_id * 0x100;
                            let jcc_code = insn.instruction.code();
                            let jcc = Instruction::with_branch(jcc_code, skip_ip).unwrap();
                            all_insns.push(IrInsn::synthetic(jcc, ctx.alloc_insn_id()));

                            // mov r15d, fall_state (overwrite with fallthrough path)
                            let mov_fall = Instruction::with2(
                                Code::Mov_rm32_imm32, state_reg32, fall_state,
                            ).unwrap();
                            all_insns.push(IrInsn::synthetic(mov_fall, ctx.alloc_insn_id()));

                            // skip_fallback: jmp dispatcher
                            let mut jmp = Instruction::with_branch(
                                Code::Jmp_rel32_64, dispatcher_ip,
                            ).unwrap();
                            jmp.set_ip(skip_ip);
                            all_insns.push(IrInsn::synthetic(jmp, ctx.alloc_insn_id()));
                        } else {
                            // Can't resolve both targets — keep original
                            emit(&mut all_insns, insn.clone(), &mut need_block_ip);
                        }
                    }
                    _ => {
                        emit(&mut all_insns, insn.clone(), &mut need_block_ip);
                    }
                }
            }

            // If block doesn't end with a terminator, add state transition to next block
            if !mapping.has_terminator && i + 1 < block_mappings.len() {
                let next_state = if !mapping.successors.is_empty() {
                    let succ_id = mapping.successors[0];
                    block_id_to_index
                        .get(&succ_id)
                        .map(|&idx| state_numbers[idx])
                        .unwrap_or(state_numbers.get(i + 1).copied().unwrap_or(0))
                } else {
                    state_numbers.get(i + 1).copied().unwrap_or(0)
                };

                let mov =
                    Instruction::with2(Code::Mov_rm32_imm32, state_reg32, next_state).unwrap();
                all_insns.push(IrInsn::synthetic(mov, ctx.alloc_insn_id()));

                let jmp =
                    Instruction::with_branch(Code::Jmp_rel32_64, dispatcher_ip).unwrap();
                all_insns.push(IrInsn::synthetic(jmp, ctx.alloc_insn_id()));
            }
        }

        // Replace function's blocks with a single flattened block
        let mut flat_block = BasicBlock::new(ctx.alloc_block_id());
        flat_block.is_entry = true;
        flat_block.instructions = all_insns;

        func.blocks = vec![flat_block];

        log::info!("CFF applied to {} ({} blocks → dispatcher)", func.name, num_blocks);

        Ok(true)
    }
}

/// Adjust RSP-relative memory displacements by +16 to compensate for the CFF
/// preamble (push r15 + sub rsp, 8 = 16 bytes on the stack).
///
/// RBP-relative displacements are NOT adjusted because:
/// - If the function uses a frame pointer (mov rbp, rsp), rbp is set AFTER the
///   CFF preamble runs, so it already accounts for the extra 16 bytes.
/// - If rbp is a general-purpose register, it holds a function-computed value
///   and rbp-relative memory accesses reference arbitrary memory.
fn adjust_stack_displacements(instr: &mut Instruction, _adjust_rbp: bool) {
    const STACK_ADJUSTMENT: u64 = 16;

    for op_idx in 0..instr.op_count() {
        if instr.op_kind(op_idx) == iced_x86::OpKind::Memory {
            let base = instr.memory_base();
            if base == Register::RSP {
                let disp = instr.memory_displacement64();
                instr.set_memory_displacement64(disp.wrapping_add(STACK_ADJUSTMENT));
                // Ensure displacement size is large enough to encode the new value
                if instr.memory_displ_size() < 1 {
                    instr.set_memory_displ_size(4);
                }
            }
            break; // Only one memory operand per x86 instruction
        }
    }
}

/// Detect if a function uses RBP as a frame pointer (has `mov rbp, rsp` pattern).
fn detect_frame_pointer(func: &Function) -> bool {
    for block in &func.blocks {
        for insn in &block.instructions {
            let code = insn.instruction.code();
            // mov rbp, rsp can be encoded as either Mov_rm64_r64 or Mov_r64_rm64
            if (code == Code::Mov_rm64_r64 || code == Code::Mov_r64_rm64)
                && insn.instruction.op0_register() == Register::RBP
                && insn.instruction.op1_register() == Register::RSP
            {
                return true;
            }
        }
    }
    false
}

/// Check if a function uses a given register (or any sub-register of it).
fn function_uses_register(func: &Function, reg: Register) -> bool {
    let related_regs = match reg {
        Register::R15 => &[
            Register::R15,
            Register::R15D,
            Register::R15W,
            Register::R15L,
        ][..],
        _ => return false,
    };

    for block in &func.blocks {
        for insn in &block.instructions {
            let instr = &insn.instruction;
            for i in 0..instr.op_count() {
                if instr.op_kind(i) == iced_x86::OpKind::Register {
                    let op_reg = instr.op_register(i);
                    if related_regs.contains(&op_reg) {
                        return true;
                    }
                }
                if instr.op_kind(i) == iced_x86::OpKind::Memory {
                    if related_regs.contains(&instr.memory_base()) {
                        return true;
                    }
                    if related_regs.contains(&instr.memory_index()) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Check if a function has indirect branches (jump tables, computed gotos).
fn function_has_indirect_branches(func: &Function) -> bool {
    for block in &func.blocks {
        for insn in &block.instructions {
            let flow = insn.instruction.flow_control();
            if flow == iced_x86::FlowControl::IndirectBranch {
                return true;
            }
        }
    }
    false
}
