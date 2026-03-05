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

        // Probabilistic: only flatten ~40% of eligible functions
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

        // Skip recursive functions — CFF adds 16 bytes of stack frame overhead per call,
        // which compounds in recursive functions and can cause stack overflows (especially
        // in debug builds with large unoptimized stack frames).
        if function_is_recursive(func) {
            log::debug!("Skipping CFF for {} — recursive function", func.name);
            return Ok(false);
        }

        // Skip functions with non-trivial stack frames (> 128 bytes). CFF adds 16 bytes
        // per frame, and deep call chains of large-frame functions (common in Rust/Go
        // debug builds with unoptimized frames) can overflow the stack.
        if let Some(frame_size) = detect_stack_frame_size(func) {
            if frame_size > 128 {
                log::debug!(
                    "Skipping CFF for {} — large stack frame ({} bytes)",
                    func.name, frame_size
                );
                return Ok(false);
            }
        }

        let state_reg = Register::R15;
        let state_reg32 = Register::R15D;
        let num_blocks = func.blocks.len();

        // Detect frame pointer style:
        // - `mov rbp, rsp`: RBP is 16 lower → positive RBP displacements need +16
        // - `lea rbp, [rsp+N]`: only that LEA needs its displacement adjusted by +16
        // - Neither: no adjustment needed
        let fp_style = detect_frame_pointer_style(func);
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

            // Adjust displacements to compensate for CFF preamble (push r15 + sub rsp, 8).
            for insn in &mut insns {
                adjust_stack_displacements(&mut insn.instruction, &fp_style);
            }

            // Find the first instruction with a real (non-zero) IP.
            // Junk insertion may prepend synthetic NOPs with IP=0 before real instructions.
            let original_first_ip = insns
                .iter()
                .map(|i| i.instruction.ip())
                .find(|&ip| ip != 0)
                .unwrap_or(0);

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
            let block_size = (mapping.insns.len() as u64) * 15 + 64;
            ip += block_size;
        }

        // Build IP remap table: old block first IP → new block IP.
        // This is needed because passes like dead-code insert mid-block branches
        // targeting synthetic IPs. After CFF reassigns block IPs, those targets
        // would be dangling. We remap them during instruction emission.
        let ip_remap: std::collections::HashMap<u64, u64> = block_mappings
            .iter()
            .enumerate()
            .map(|(i, m)| (m.original_first_ip, block_start_ips[i]))
            .collect();

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
                } else {
                    // Clear original IPs on non-first instructions so BlockEncoder
                    // doesn't misresolve external calls/jumps to in-function instructions
                    // that happen to share the original function's entry IP.
                    ir.instruction.set_ip(0);
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
                            m.original_first_ip != 0 && m.original_first_ip == target_addr
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
                            m.original_first_ip != 0 && m.original_first_ip == target_addr
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
                        // For mid-block branches (e.g., opaque predicates from dead-code),
                        // remap targets so they point to the correct CFF block IP.
                        let mut cloned = insn.clone();
                        let f = cloned.instruction.flow_control();
                        if (f == iced_x86::FlowControl::ConditionalBranch
                            || f == iced_x86::FlowControl::UnconditionalBranch)
                            && !is_last
                        {
                            let target = cloned.instruction.near_branch_target();
                            if let Some(&new_ip) = ip_remap.get(&target) {
                                log::debug!("CFF: remapping mid-block branch 0x{:x} → 0x{:x}", target, new_ip);
                                // Re-create the branch with the remapped target
                                let code = cloned.instruction.code();
                                if let Ok(new_branch) = Instruction::with_branch(code, new_ip) {
                                    cloned.instruction = new_branch;
                                }
                            } else {
                                log::warn!("CFF: mid-block branch to 0x{:x} NOT in ip_remap!", target);
                            }
                        }
                        emit(&mut all_insns, cloned, &mut need_block_ip);
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

/// Frame pointer style detected in a function.
#[derive(Debug, Clone, PartialEq)]
enum FramePointerStyle {
    /// No frame pointer detected — no adjustments needed.
    None,
    /// `mov rbp, rsp` style: RBP is 16 lower, positive RBP displacements need +16.
    MovRbpRsp,
    /// `lea rbp, [rsp+N]` style: RBP shifts down by 16 (same as MovRbpRsp), so positive
    /// RBP displacements (shadow space / params) need +16. Negative displacements (locals)
    /// are fine because the function's local area also shifts down. The LEA itself is NOT
    /// adjusted — RBP naturally moves 16 lower, preserving the distance to locals while
    /// positive-disp adjustments fix parameter access.
    LeaRbpRsp,
}

/// Adjust displacements to compensate for the CFF preamble (push r15 + sub rsp, 8 = 16 bytes).
///
/// The key insight: RSP-relative accesses generally do NOT need adjustment because
/// the shifted RSP is self-consistent for locals, call argument setup, and other
/// RSP-relative operations. The callee's RSP is also shifted, so call args match.
///
/// For both `mov rbp, rsp` and `lea rbp, [rsp+N]` frame pointers: RBP naturally
/// shifts 16 bytes lower due to the CFF preamble. Positive RBP offsets (accessing
/// caller's shadow space / parameters) need +16 to compensate. Negative offsets
/// (locals) are fine since they're relative to wherever RBP ends up, and the
/// function's local area also shifted down. The LEA instruction is NOT adjusted —
/// letting RBP shift preserves the distance between RBP and locals/saved registers.
fn adjust_stack_displacements(instr: &mut Instruction, fp_style: &FramePointerStyle) {
    const STACK_ADJUSTMENT: u64 = 16;

    match fp_style {
        FramePointerStyle::None => return,
        FramePointerStyle::MovRbpRsp | FramePointerStyle::LeaRbpRsp => {
            // Adjust positive RBP-relative displacements only
            for op_idx in 0..instr.op_count() {
                if instr.op_kind(op_idx) == iced_x86::OpKind::Memory {
                    if instr.memory_base() == Register::RBP {
                        let disp_signed = instr.memory_displacement64() as i64;
                        if disp_signed > 0 {
                            let new_disp = instr.memory_displacement64().wrapping_add(STACK_ADJUSTMENT);
                            instr.set_memory_displacement64(new_disp);
                            let new_disp_signed = new_disp as i64;
                            if instr.memory_displ_size() <= 1
                                && !((-128..=127).contains(&new_disp_signed))
                            {
                                instr.set_memory_displ_size(4);
                            }
                        }
                    }
                    break;
                }
            }
        }
    }
}

/// Detect the frame pointer style used by a function.
fn detect_frame_pointer_style(func: &Function) -> FramePointerStyle {
    for block in &func.blocks {
        for insn in &block.instructions {
            let code = insn.instruction.code();
            // mov rbp, rsp
            if (code == Code::Mov_rm64_r64 || code == Code::Mov_r64_rm64)
                && insn.instruction.op0_register() == Register::RBP
                && insn.instruction.op1_register() == Register::RSP
            {
                return FramePointerStyle::MovRbpRsp;
            }
            // lea rbp, [rsp+N]
            if matches!(code, Code::Lea_r64_m | Code::Lea_r32_m)
                && insn.instruction.op0_register() == Register::RBP
                && insn.instruction.memory_base() == Register::RSP
                && insn.instruction.memory_index() == Register::None
            {
                return FramePointerStyle::LeaRbpRsp;
            }
        }
    }
    FramePointerStyle::None
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

/// Check if a function is self-recursive (calls its own entry address).
/// Detect the stack frame size from the prologue's `sub rsp, N` instruction.
fn detect_stack_frame_size(func: &Function) -> Option<u64> {
    let entry = func.blocks.first()?;
    // Look at the first few instructions for `sub rsp, imm`
    for insn in entry.instructions.iter().take(10) {
        let i = &insn.instruction;
        if i.mnemonic() == iced_x86::Mnemonic::Sub
            && i.op0_register() == Register::RSP
            && i.op1_kind() == iced_x86::OpKind::Immediate32to64
        {
            return Some(i.immediate32to64() as u64);
        }
    }
    None
}

fn function_is_recursive(func: &Function) -> bool {
    // Find the function's entry IP
    let entry_ip = func.blocks.first()
        .and_then(|b| b.instructions.iter().find(|i| i.instruction.ip() != 0))
        .map(|i| i.instruction.ip());

    let entry_ip = match entry_ip {
        Some(ip) => ip,
        None => return false,
    };

    for block in &func.blocks {
        for insn in &block.instructions {
            if insn.instruction.mnemonic() == iced_x86::Mnemonic::Call
                && insn.instruction.op0_kind() == iced_x86::OpKind::NearBranch64
                && insn.instruction.near_branch64() == entry_ip
            {
                return true;
            }
        }
    }
    false
}
