use anyhow::{Context, Result};
use iced_x86::{BlockEncoder, BlockEncoderOptions, InstructionBlock};

use crate::coff::types::CoffRelocation;
use crate::ir::function::Function;
use crate::ir::instruction::IrInsn;

/// Result of encoding a function back to bytes.
#[derive(Debug)]
pub struct EncodedFunction {
    pub code: Vec<u8>,
    pub relocations: Vec<CoffRelocation>,
}

/// Encode a function's IR back into machine code bytes.
pub fn encode_function(func: &Function, base_address: u64) -> Result<EncodedFunction> {
    let all_insns: Vec<&IrInsn> = func
        .blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .collect();

    if all_insns.is_empty() {
        return Ok(EncodedFunction {
            code: Vec::new(),
            relocations: Vec::new(),
        });
    }

    let instructions: Vec<iced_x86::Instruction> = all_insns.iter().map(|i| i.instruction).collect();

    let block = InstructionBlock::new(&instructions, base_address);
    let result = match BlockEncoder::encode(64, block, BlockEncoderOptions::NONE) {
        Ok(r) => r,
        Err(e) => {
            log::debug!(
                "BlockEncoder failed for {} ({} insns at 0x{:x}): {}",
                func.name, instructions.len(), base_address, e
            );
            anyhow::bail!("BlockEncoder failed: {}", e);
        }
    };

    let code = result.code_buffer;

    // Decode output to get per-instruction byte offsets
    let mut decoder =
        iced_x86::Decoder::with_ip(64, &code, base_address, iced_x86::DecoderOptions::NONE);
    let mut decoded_offsets: Vec<(u64, u32)> = Vec::new();

    while decoder.can_decode() {
        let pos = decoder.position() as u64;
        let _instr = decoder.decode();
        let len = decoder.position() as u64 - pos;
        decoded_offsets.push((pos, len as u32));
    }

    let mut relocations = Vec::new();

    if decoded_offsets.len() == all_insns.len() {
        for (i, ir_insn) in all_insns.iter().enumerate() {
            if let Some(ref reloc) = ir_insn.relocation {
                let (byte_offset, _) = decoded_offsets[i];
                relocations.push(CoffRelocation {
                    offset: byte_offset + reloc.offset_in_insn as u64,
                    symbol_index: reloc.symbol_index,
                    typ: reloc.typ,
                });
            }
        }
    } else {
        log::warn!(
            "Instruction count mismatch after encoding: {} vs {}",
            decoded_offsets.len(),
            all_insns.len()
        );
        for (i, ir_insn) in all_insns.iter().enumerate() {
            if let Some(ref reloc) = ir_insn.relocation {
                if i < decoded_offsets.len() {
                    let (bo, _) = decoded_offsets[i];
                    relocations.push(CoffRelocation {
                        offset: bo + reloc.offset_in_insn as u64,
                        symbol_index: reloc.symbol_index,
                        typ: reloc.typ,
                    });
                }
            }
        }
    }

    Ok(EncodedFunction { code, relocations })
}
