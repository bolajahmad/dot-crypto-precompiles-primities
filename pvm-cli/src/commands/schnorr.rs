/// Schnorr signature command handlers.
///
/// This module provides CLI commands for:
/// - Generating Schnorr signatures
/// - Verifying Schnorr signatures
/// - Generating test data for Solidity integration

use alloy_primitives::Keccak256;
use clap::Subcommand;
use schnorr::{
    encode_precompile_input, generate_signature, verify_signature,
    secp256k1::SecretKey,
    utils::message,
};

use crate::utils::parse_hex_32;

/// Schnorr signature subcommands.
#[derive(Subcommand)]
pub enum SchnorrCommands {
    /// Generate a Schnorr signature
    Sign {
        /// Secret key as hex string (32 bytes / 64 hex chars)
        #[arg(short, long)]
        secret_key: String,

        /// Message to sign (will be hashed)
        #[arg(short, long)]
        message: String,

        /// Random auxiliary data for nonce generation (hex, 32 bytes).
        /// If not provided, a deterministic value will be generated.
        #[arg(short, long)]
        aux: Option<String>,

        /// Output format: 'json' or 'hex' (default: hex)
        #[arg(short, long, default_value = "hex")]
        output: String,
    },

    /// Verify a Schnorr signature
    Verify {
        /// Public key x-coordinate (hex, 32 bytes)
        #[arg(short, long)]
        pubkey: String,

        /// Nonce point R x-coordinate (hex, 32 bytes)
        #[arg(short, long)]
        nonce: String,

        /// Signature scalar s (hex, 32 bytes)
        #[arg(short, long)]
        signature: String,

        /// Message that was signed (will be hashed)
        #[arg(short, long)]
        message: String,
    },

    /// Generate test data for Solidity integration
    TestData {
        /// Message to sign (will be hashed)
        #[arg(short, long, default_value = "Hello, world!")]
        message: String,

        /// Secret key as hex string (32 bytes / 64 hex chars).
        /// If not provided, a deterministic default will be used.
        #[arg(short, long)]
        secret_key: Option<String>,

        /// Auxiliary nonce input as hex string (32 bytes / 64 hex chars).
        /// If not provided, a deterministic default will be used.
        #[arg(short = 'n', long, alias = "aux")]
        nonce: Option<String>,
    },
}

/// Handle Schnorr subcommands.
pub fn handle(action: SchnorrCommands) {
    match action {
        SchnorrCommands::Sign {
            secret_key,
            message: msg_text,
            aux,
            output,
        } => handle_sign(secret_key, msg_text, aux, output),

        SchnorrCommands::Verify {
            pubkey,
            nonce,
            signature,
            message: msg_text,
        } => handle_verify(pubkey, nonce, signature, msg_text),

        SchnorrCommands::TestData {
            message: msg_text,
            secret_key,
            nonce,
        } => handle_test_data(msg_text, secret_key, nonce),
    }
}

/// Generate a Schnorr signature.
fn handle_sign(secret_key: String, msg_text: String, aux: Option<String>, output: String) {
    let sk_bytes = parse_required_hex_32(&secret_key, "secret_key");
    let secret_key = parse_secret_key_from_bytes(sk_bytes);
    let aux_bytes = parse_optional_hex_32(aux, "aux")
        .unwrap_or_else(|| generate_deterministic_aux(&msg_text, &sk_bytes));

    // Hash the message
    let msg = message(Some(&msg_text));

    // Generate signature
    let (pubkey_x, rx, s) = generate_signature(secret_key, &msg, aux_bytes);

    // Generate precompile input
    let precompile_input = encode_precompile_input(&pubkey_x, &rx, &s, &msg);

    // Output result
    if output == "json" {
        print_signature_json(&pubkey_x, &rx, &s, &msg, &precompile_input);
    } else {
        print_signature_hex(&pubkey_x, &rx, &s, &msg, &precompile_input);
    }
}

/// Verify a Schnorr signature.
fn handle_verify(pubkey: String, nonce: String, signature: String, msg_text: String) {
    // Parse inputs
    let pubkey_x = match parse_hex_32(&pubkey, "pubkey") {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let rx = match parse_hex_32(&nonce, "nonce") {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let s = match parse_hex_32(&signature, "signature") {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Hash the message
    let msg = message(Some(&msg_text));

    // Verify
    let valid = verify_signature(&pubkey_x, &rx, &s, &msg);

    if valid {
        println!("Signature is VALID");
    } else {
        println!("Signature is INVALID");
        std::process::exit(1);
    }
}

/// Generate test data for Solidity integration.
fn handle_test_data(msg_text: String, secret_key: Option<String>, nonce: Option<String>) {
    let (sk_bytes, aux) = resolve_test_data_inputs(&msg_text, secret_key, nonce);
    let secret_key = parse_secret_key_from_bytes(sk_bytes);
    let msg = message(Some(&msg_text));

    let (pubkey_x, rx, s) = generate_signature(secret_key, &msg, aux);
    let precompile_input = encode_precompile_input(&pubkey_x, &rx, &s, &msg);

    println!("=== Schnorr Test Data for Solidity ===");
    println!("Message: \"{}\"", msg_text);
    println!();
    println!("=== Solidity test data ===");
    println!("bytes32 constant PUBKEY_X = 0x{};", hex::encode(pubkey_x));
    println!("bytes32 constant NONCE_RX = 0x{};", hex::encode(rx));
    println!("bytes32 constant SIGNATURE_S = 0x{};", hex::encode(s));
    println!("bytes32 constant MESSAGE_HASH = 0x{};", hex::encode(msg));
    println!();
    println!("=== Full precompile input (128 bytes) ===");
    println!(
        "bytes constant PRECOMPILE_INPUT = hex\"{}\";",
        hex::encode(&precompile_input)
    );
    println!();
    println!("=== For SchnorrSignature struct ===");
    println!("SchnorrSignature memory sig = SchnorrSignature({{");
    println!("    pubkey: 0x{},", hex::encode(pubkey_x));
    println!("    nonce: 0x{},", hex::encode(rx));
    println!("    s: 0x{},", hex::encode(s));
    println!("    message: 0x{}", hex::encode(msg));
    println!("}});");
}

// -- Helper functions --

fn parse_required_hex_32(hex_str: &str, name: &str) -> [u8; 32] {
    match parse_hex_32(hex_str, name) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn parse_optional_hex_32(hex_str: Option<String>, name: &str) -> Option<[u8; 32]> {
    hex_str.map(|value| parse_required_hex_32(&value, name))
}

fn parse_secret_key_from_bytes(sk_bytes: [u8; 32]) -> SecretKey {
    match SecretKey::from_slice(&sk_bytes) {
        Ok(sk) => sk,
        Err(e) => {
            eprintln!("Error: Invalid secret key: {}", e);
            std::process::exit(1);
        }
    }
}

fn resolve_test_data_inputs(
    _msg_text: &str,
    secret_key: Option<String>,
    nonce: Option<String>,
) -> ([u8; 32], [u8; 32]) {
    let sk_bytes = parse_optional_hex_32(secret_key, "secret_key").unwrap_or([1u8; 32]);
    let aux = parse_optional_hex_32(nonce, "nonce").unwrap_or([2u8; 32]);

    (sk_bytes, aux)
}

/// Generate deterministic aux bytes from message and secret key.
/// This ensures reproducible signatures when aux is not provided.
fn generate_deterministic_aux(msg_text: &str, sk_bytes: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update("PIP/cli-aux");
    hasher.update(msg_text.as_bytes());
    hasher.update(sk_bytes);
    let hash = hasher.finalize();
    let mut aux = [0u8; 32];
    aux.copy_from_slice(hash.as_slice());
    aux
}

/// Print signature in JSON format.
fn print_signature_json(
    pubkey_x: &[u8; 32],
    rx: &[u8; 32],
    s: &[u8; 32],
    msg: &[u8; 32],
    precompile_input: &[u8],
) {
    println!("{{");
    println!("  \"pubkey_x\": \"0x{}\",", hex::encode(pubkey_x));
    println!("  \"nonce_rx\": \"0x{}\",", hex::encode(rx));
    println!("  \"signature_s\": \"0x{}\",", hex::encode(s));
    println!("  \"message_hash\": \"0x{}\",", hex::encode(msg));
    println!(
        "  \"precompile_input\": \"0x{}\"",
        hex::encode(precompile_input)
    );
    println!("}}");
}

/// Print signature in hex format (human-readable).
fn print_signature_hex(
    pubkey_x: &[u8; 32],
    rx: &[u8; 32],
    s: &[u8; 32],
    msg: &[u8; 32],
    precompile_input: &[u8],
) {
    println!("=== Schnorr Signature Generated ===");
    println!("Public Key (x):     0x{}", hex::encode(pubkey_x));
    println!("Nonce (R_x):        0x{}", hex::encode(rx));
    println!("Signature (s):      0x{}", hex::encode(s));
    println!("Message Hash:       0x{}", hex::encode(msg));
    println!("---");
    println!("Precompile Input (128 bytes):");
    println!("0x{}", hex::encode(precompile_input));
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use schnorr::{
        encode_precompile_input, generate_signature, verify_signature,
        secp256k1::SecretKey,
        utils::message,
    };

    // -- Signature generation tests --

    #[test]
    fn test_generate_signature_produces_valid_components() {
        let sk_bytes = [1u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let aux = [2u8; 32];
        let msg = message(Some("Test message"));

        let (pubkey_x, rx, s) = generate_signature(secret_key, &msg, aux);

        // All components should be 32 bytes
        assert_eq!(pubkey_x.len(), 32);
        assert_eq!(rx.len(), 32);
        assert_eq!(s.len(), 32);

        // Components should not be all zeros
        assert_ne!(pubkey_x, [0u8; 32]);
        assert_ne!(rx, [0u8; 32]);
        assert_ne!(s, [0u8; 32]);
    }

    #[test]
    fn test_generate_signature_deterministic() {
        // Same inputs should produce same outputs
        let sk_bytes = [1u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let aux = [2u8; 32];
        let msg = message(Some("Test message"));

        let (pubkey_x_1, rx_1, s_1) = generate_signature(secret_key, &msg, aux);
        let (pubkey_x_2, rx_2, s_2) = generate_signature(secret_key, &msg, aux);

        assert_eq!(pubkey_x_1, pubkey_x_2);
        assert_eq!(rx_1, rx_2);
        assert_eq!(s_1, s_2);
    }

    #[test]
    fn test_generate_signature_different_messages_produce_different_signatures() {
        let sk_bytes = [1u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let aux = [2u8; 32];

        let msg_1 = message(Some("Message A"));
        let msg_2 = message(Some("Message B"));

        let (_, rx_1, s_1) = generate_signature(secret_key, &msg_1, aux);
        let (_, rx_2, s_2) = generate_signature(secret_key, &msg_2, aux);

        // Different messages should produce different nonces and signatures
        assert_ne!(rx_1, rx_2);
        assert_ne!(s_1, s_2);
    }

    #[test]
    fn test_generate_signature_different_aux_produce_different_nonces() {
        let sk_bytes = [1u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let msg = message(Some("Test message"));

        let aux_1 = [1u8; 32];
        let aux_2 = [2u8; 32];

        let (_, rx_1, _) = generate_signature(secret_key, &msg, aux_1);
        let (_, rx_2, _) = generate_signature(secret_key, &msg, aux_2);

        // Different aux should produce different nonces
        assert_ne!(rx_1, rx_2);
    }

    // -- Signature verification tests --

    #[test]
    fn test_verify_signature_valid() {
        let sk_bytes = [1u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let aux = [2u8; 32];
        let msg = message(Some("Hello, world!"));

        let (pubkey_x, rx, s) = generate_signature(secret_key, &msg, aux);

        let valid = verify_signature(&pubkey_x, &rx, &s, &msg);
        assert!(valid, "Generated signature should verify successfully");
    }

    #[test]
    fn test_verify_signature_wrong_message_fails() {
        let sk_bytes = [1u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let aux = [2u8; 32];
        let msg = message(Some("Original message"));

        let (pubkey_x, rx, s) = generate_signature(secret_key, &msg, aux);

        // Try to verify with a different message
        let wrong_msg = message(Some("Wrong message"));
        let valid = verify_signature(&pubkey_x, &rx, &s, &wrong_msg);
        assert!(!valid, "Signature should not verify with wrong message");
    }

    #[test]
    fn test_verify_signature_wrong_pubkey_fails() {
        let sk_bytes = [1u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let aux = [2u8; 32];
        let msg = message(Some("Test message"));

        let (_, rx, s) = generate_signature(secret_key, &msg, aux);

        // Use a different public key (from a different secret key)
        let other_sk_bytes = [3u8; 32];
        let other_secret_key = SecretKey::from_slice(&other_sk_bytes).unwrap();
        let (other_pubkey_x, _, _) = generate_signature(other_secret_key, &msg, aux);

        let valid = verify_signature(&other_pubkey_x, &rx, &s, &msg);
        assert!(!valid, "Signature should not verify with wrong public key");
    }

    #[test]
    fn test_verify_signature_tampered_s_fails() {
        let sk_bytes = [1u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let aux = [2u8; 32];
        let msg = message(Some("Test message"));

        let (pubkey_x, rx, mut s) = generate_signature(secret_key, &msg, aux);

        // Tamper with the signature scalar
        s[0] = s[0].wrapping_add(1);

        let valid = verify_signature(&pubkey_x, &rx, &s, &msg);
        assert!(!valid, "Signature should not verify with tampered s value");
    }

    #[test]
    fn test_verify_signature_tampered_nonce_fails() {
        let sk_bytes = [1u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let aux = [2u8; 32];
        let msg = message(Some("Test message"));

        let (pubkey_x, mut rx, s) = generate_signature(secret_key, &msg, aux);

        // Tamper with the nonce
        rx[0] = rx[0].wrapping_add(1);

        let valid = verify_signature(&pubkey_x, &rx, &s, &msg);
        assert!(!valid, "Signature should not verify with tampered nonce");
    }

    // -- Precompile input encoding tests --

    #[test]
    fn test_precompile_input_length() {
        let sk_bytes = [1u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let aux = [2u8; 32];
        let msg = message(Some("Test"));

        let (pubkey_x, rx, s) = generate_signature(secret_key, &msg, aux);
        let input = encode_precompile_input(&pubkey_x, &rx, &s, &msg);

        // Precompile input should be exactly 128 bytes
        // pubkey_x (32) + rx (32) + s (32) + msg (32) = 128
        assert_eq!(input.len(), 128, "Precompile input must be 128 bytes");
    }

    #[test]
    fn test_precompile_input_encoding_order() {
        let pubkey_x = [1u8; 32];
        let rx = [2u8; 32];
        let s = [3u8; 32];
        let msg = [4u8; 32];

        let input = encode_precompile_input(&pubkey_x, &rx, &s, &msg);

        // Verify encoding order: pubkey_x || rx || s || msg
        assert_eq!(&input[0..32], &pubkey_x, "First 32 bytes should be pubkey_x");
        assert_eq!(&input[32..64], &rx, "Bytes 32-64 should be rx");
        assert_eq!(&input[64..96], &s, "Bytes 64-96 should be s");
        assert_eq!(&input[96..128], &msg, "Bytes 96-128 should be msg");
    }

    // -- Integration tests: sign then verify flow --

    #[test]
    fn test_sign_verify_roundtrip_multiple_messages() {
        let sk_bytes = [42u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let aux = [99u8; 32];

        let test_messages = vec![
            "Hello",
            "Test message with spaces",
            "1234567890",
            "",
            "A very long message that contains many characters and should still work correctly",
        ];

        for msg_text in test_messages {
            let msg = message(Some(msg_text));
            let (pubkey_x, rx, s) = generate_signature(secret_key, &msg, aux);
            let valid = verify_signature(&pubkey_x, &rx, &s, &msg);
            assert!(valid, "Roundtrip failed for message: '{}'", msg_text);
        }
    }

    #[test]
    fn test_sign_verify_with_different_secret_keys() {
        let aux = [1u8; 32];
        let msg = message(Some("Common message"));

        // Test with several different secret keys
        for i in 1u8..=5 {
            let sk_bytes = [i; 32];
            let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();

            let (pubkey_x, rx, s) = generate_signature(secret_key, &msg, aux);
            let valid = verify_signature(&pubkey_x, &rx, &s, &msg);
            assert!(valid, "Roundtrip failed for secret key index: {}", i);
        }
    }

    // -- Known test vector tests --

    #[test]
    fn test_known_test_vector() {
        // This test uses the same parameters as test-data command
        // to ensure consistency
        let sk_bytes = [1u8; 32];
        let secret_key = SecretKey::from_slice(&sk_bytes).unwrap();
        let aux = [2u8; 32];
        let msg = message(Some("Hello, world!"));

        let (pubkey_x, rx, s) = generate_signature(secret_key, &msg, aux);

        // Verify the signature is valid
        let valid = verify_signature(&pubkey_x, &rx, &s, &msg);
        assert!(valid);

        // Verify precompile input is correct length
        let input = encode_precompile_input(&pubkey_x, &rx, &s, &msg);
        assert_eq!(input.len(), 128);

        // The public key should be deterministic for this secret key
        let expected_pubkey_hex = "1b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f";
        assert_eq!(
            hex::encode(pubkey_x),
            expected_pubkey_hex,
            "Public key should match expected value for sk=[1u8; 32]"
        );
    }

    // -- Helper function tests --

    #[test]
    fn test_generate_deterministic_aux() {
        let sk_bytes = [1u8; 32];
        let msg = "Test message";

        let aux1 = generate_deterministic_aux(msg, &sk_bytes);
        let aux2 = generate_deterministic_aux(msg, &sk_bytes);

        // Same inputs should produce same aux
        assert_eq!(aux1, aux2);
    }

    #[test]
    fn test_generate_deterministic_aux_different_inputs() {
        let sk_bytes = [1u8; 32];

        let aux1 = generate_deterministic_aux("Message A", &sk_bytes);
        let aux2 = generate_deterministic_aux("Message B", &sk_bytes);

        // Different messages should produce different aux
        assert_ne!(aux1, aux2);
    }

    #[test]
    fn test_resolve_test_data_inputs_defaults() {
        let (sk_bytes, aux) = resolve_test_data_inputs("Hello, world!", None, None);

        assert_eq!(sk_bytes, [1u8; 32]);
        assert_eq!(aux, [2u8; 32]);
    }

    #[test]
    fn test_resolve_test_data_inputs_custom_secret_key_keeps_default_nonce_seed() {
        let custom_sk = Some(
            "0303030303030303030303030303030303030303030303030303030303030303"
                .to_string(),
        );
        let (sk_bytes, aux) = resolve_test_data_inputs("Custom message", custom_sk, None);

        assert_eq!(sk_bytes, [3u8; 32]);
        assert_eq!(aux, [2u8; 32]);
    }

    #[test]
    fn test_resolve_test_data_inputs_custom_nonce_overrides_default() {
        let custom_nonce = Some(
            "0404040404040404040404040404040404040404040404040404040404040404"
                .to_string(),
        );
        let (_, aux) = resolve_test_data_inputs("Hello, world!", None, custom_nonce);

        assert_eq!(aux, [4u8; 32]);
    }

    // -- Edge case tests --

    #[test]
    fn test_message_hash_is_deterministic() {
        let msg1 = message(Some("Test"));
        let msg2 = message(Some("Test"));
        assert_eq!(msg1, msg2, "Same message should produce same hash");
    }

    #[test]
    fn test_different_messages_produce_different_hashes() {
        let msg1 = message(Some("Test1"));
        let msg2 = message(Some("Test2"));
        assert_ne!(msg1, msg2, "Different messages should produce different hashes");
    }
}
