use std::collections::BTreeMap;

use crate::types::XcmInstruction;

/// Format a u128 amount with thousands separators (e.g. `1,000,000`).
pub fn format_amount(amount: u128) -> String {
    amount
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap_or_default()
        .join(",")
}

/// Truncate a hex string for display while preserving total byte length.
pub fn summarize_hex(bytes: &[u8], prefix_chars: usize) -> String {
    let hex = format!("0x{}", hex::encode(bytes));
    if hex.len() > prefix_chars {
        format!(
            "{}.. ({} bytes total)",
            &hex[..prefix_chars.min(hex.len())],
            bytes.len()
        )
    } else {
        hex
    }
}

/// Best-effort instruction name from a Debug dump (`Foo(…)` → `Foo`).
pub fn instruction_name_from_debug(value: &impl std::fmt::Debug) -> String {
    format!("{value:?}")
        .split('(')
        .next()
        .unwrap_or("Unknown")
        .to_string()
}

/// Fallback instruction when no structured formatter is available.
pub fn raw_instruction(value: &impl std::fmt::Debug) -> XcmInstruction {
    let mut params = BTreeMap::new();
    params.insert("Raw".to_string(), vec![format!("{value:#?}")]);
    XcmInstruction::new(instruction_name_from_debug(value), params)
}

/// Short AccountId32 / AccountKey20 preview.
pub fn short_hex(bytes: &[u8]) -> String {
    let hex = hex::encode(bytes);
    if hex.len() > 6 {
        format!("0x{}...", &hex[..6])
    } else {
        format!("0x{hex}")
    }
}
