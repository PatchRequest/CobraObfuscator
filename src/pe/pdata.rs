use anyhow::{bail, Result};

/// A single RUNTIME_FUNCTION entry from .pdata.
#[derive(Debug, Clone)]
pub struct RuntimeFunction {
    pub begin_address: u32,
    pub end_address: u32,
    pub unwind_info_address: u32,
}

/// Size of a RUNTIME_FUNCTION entry (3 × u32).
const RUNTIME_FUNCTION_SIZE: usize = 12;

/// Parse the .pdata section into RUNTIME_FUNCTION entries.
pub fn parse_pdata(data: &[u8]) -> Result<Vec<RuntimeFunction>> {
    if data.len() % RUNTIME_FUNCTION_SIZE != 0 {
        bail!(
            ".pdata size {} is not a multiple of {}",
            data.len(),
            RUNTIME_FUNCTION_SIZE
        );
    }

    let count = data.len() / RUNTIME_FUNCTION_SIZE;
    let mut entries = Vec::with_capacity(count);

    for i in 0..count {
        let base = i * RUNTIME_FUNCTION_SIZE;
        let begin = u32::from_le_bytes(data[base..base + 4].try_into().unwrap());
        let end = u32::from_le_bytes(data[base + 4..base + 8].try_into().unwrap());
        let unwind = u32::from_le_bytes(data[base + 8..base + 12].try_into().unwrap());

        // Skip null entries
        if begin == 0 && end == 0 {
            continue;
        }

        entries.push(RuntimeFunction {
            begin_address: begin,
            end_address: end,
            unwind_info_address: unwind,
        });
    }

    Ok(entries)
}
