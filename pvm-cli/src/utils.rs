/// Shared utility functions for the PVM CLI.

/// Parse a hex string into a 32-byte array.
/// Accepts input with or without "0x" prefix.
pub fn parse_hex_32(hex_str: &str, name: &str) -> Result<[u8; 32], String> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);

    let bytes = hex::decode(hex_str)
        .map_err(|e| format!("{} is not valid hex: {}", name, e))?;

    if bytes.len() != 32 {
        return Err(format!(
            "{} must be 32 bytes (64 hex chars), got {} bytes",
            name,
            bytes.len()
        ));
    }

    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_32_valid_input() {
        let hex_str = "0101010101010101010101010101010101010101010101010101010101010101";
        let result = parse_hex_32(hex_str, "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [1u8; 32]);
    }

    #[test]
    fn test_parse_hex_32_with_0x_prefix() {
        let hex_str = "0x0101010101010101010101010101010101010101010101010101010101010101";
        let result = parse_hex_32(hex_str, "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [1u8; 32]);
    }

    #[test]
    fn test_parse_hex_32_invalid_hex() {
        let hex_str = "GGGG010101010101010101010101010101010101010101010101010101010101";
        let result = parse_hex_32(hex_str, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not valid hex"));
    }

    #[test]
    fn test_parse_hex_32_wrong_length_short() {
        let hex_str = "0101010101"; // Only 5 bytes
        let result = parse_hex_32(hex_str, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be 32 bytes"));
    }

    #[test]
    fn test_parse_hex_32_wrong_length_long() {
        // 34 bytes
        let hex_str = "01010101010101010101010101010101010101010101010101010101010101010101";
        let result = parse_hex_32(hex_str, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be 32 bytes"));
    }

    #[test]
    fn test_parse_hex_32_empty() {
        let result = parse_hex_32("", "test");
        assert!(result.is_err());
    }
}
