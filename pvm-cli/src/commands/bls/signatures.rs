use ark_bls12_381::G1Affine;
use bls12_381::{
    aggregate_signatures, derive_public_key, generate_batch_signature_testdata, sign_message,
    verify_batch_signatures, verify_signature,
};
use serde::{Deserialize, Serialize};

use super::{
    points::{print_g1_point, print_g2_point},
    shared::{
        exit_with_error, g1_to_output, g2_to_output, parse_g1_point_or_exit, parse_g1_point_struct,
        parse_g2_point_or_exit, parse_g2_point_struct, parse_secret_key_or_exit, G1PointInput,
        G2PointInput,
    },
};

#[derive(Deserialize)]
struct BatchAggregateInput {
    signatures: Vec<G1PointInput>,
}

#[derive(Deserialize)]
struct BatchVerifyInput {
    messages: Vec<String>,
    signatures: Vec<G1PointInput>,
    pubkeys: Vec<G2PointInput>,
    aggregated_signature: Option<G1PointInput>,
}

#[derive(Serialize)]
struct BatchSignOut {
    messages: Vec<String>,
    secret_keys: Vec<String>,
    signatures: Vec<super::shared::G1PointOut>,
    pubkeys: Vec<super::shared::G2PointOut>,
    aggregated_signature: super::shared::G1PointOut,
}

#[derive(Serialize)]
struct BatchSmokeOut {
    count: usize,
    aggregate_matches: bool,
    verification_valid: bool,
}

pub(super) fn cmd_sign(secret_key: String, message: String) {
    let sk = parse_secret_key_or_exit(&secret_key);
    let signature = sign_message(sk, message.as_bytes());
    let pubkey = derive_public_key(sk);

    println!("=== BLS Signature ===");
    println!("Message: {}", message);
    print_g1_point("Signature", &signature);
    print_g2_point("Public Key", &pubkey);
}

pub(super) fn cmd_verify(signature: String, pubkey: String, message: String) {
    let signature = parse_g1_point_or_exit(&signature, "signature");
    let pubkey = parse_g2_point_or_exit(&pubkey, "pubkey");
    let valid = verify_signature(&signature, message.as_bytes(), &pubkey);

    if valid {
        println!("BLS signature is VALID");
    } else {
        println!("BLS signature is INVALID");
        std::process::exit(1);
    }
}

pub(super) fn cmd_batch_sign_testdata(count: usize, output: String) {
    let (messages, secret_keys, signatures, pubkeys, aggregated_signature) =
        generate_batch_signature_testdata(count, [91u8; 32]).unwrap_or_else(|e| exit_with_error(&e));

    let out = BatchSignOut {
        messages: messages
            .iter()
            .map(|m| String::from_utf8_lossy(m).to_string())
            .collect(),
        secret_keys: secret_keys
            .iter()
            .map(|s| format!("0x{}", hex::encode(bls12_381::codecs::encode_scalar(*s))))
            .collect(),
        signatures: signatures.iter().map(|s| g1_to_output(*s)).collect(),
        pubkeys: pubkeys.iter().map(|p| g2_to_output(*p)).collect(),
        aggregated_signature: g1_to_output(aggregated_signature),
    };

    println!("=== BLS Batch Signature Test Data ===");
    println!("Count: {}", count);

    if output == "hex" || output == "both" {
        let sig_hex = signatures
            .iter()
            .map(|s| hex::encode(bls12_381::codecs::encode_g1(*s)))
            .collect::<Vec<_>>()
            .join("");
        println!("Signatures (concatenated hex): 0x{}", sig_hex);
        println!(
            "Aggregated signature (hex): 0x{}",
            hex::encode(bls12_381::codecs::encode_g1(aggregated_signature))
        );
    }

    if output == "json" || output == "both" {
        println!(
            "JSON: {}",
            serde_json::to_string_pretty(&out).expect("serialize batch sign output")
        );
    }
}

pub(super) fn cmd_batch_aggregate(signatures: String) {
    let signatures = parse_signatures_payload(&signatures)
        .unwrap_or_else(|e| exit_with_error(&format!("invalid signatures payload: {}", e)));
    let aggregated = aggregate_signatures(&signatures).unwrap_or_else(|e| exit_with_error(&e));

    println!("=== BLS Batch Aggregate ===");
    println!("Signatures: {}", signatures.len());
    print_g1_point("Aggregated Signature", &aggregated);
}

pub(super) fn cmd_batch_verify(data: String) {
    let parsed: BatchVerifyInput = serde_json::from_str(&data)
        .unwrap_or_else(|e| exit_with_error(&format!("invalid batch verify json: {}", e)));

    if parsed.messages.is_empty() {
        exit_with_error("messages cannot be empty");
    }

    let signatures = parsed
        .signatures
        .iter()
        .map(parse_g1_point_struct)
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|e| exit_with_error(&format!("invalid signature point: {}", e)));

    let pubkeys = parsed
        .pubkeys
        .iter()
        .map(parse_g2_point_struct)
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|e| exit_with_error(&format!("invalid pubkey point: {}", e)));

    let messages = parsed
        .messages
        .iter()
        .map(|m| m.as_bytes().to_vec())
        .collect::<Vec<_>>();

    let aggregated = parsed
        .aggregated_signature
        .as_ref()
        .map(parse_g1_point_struct)
        .transpose()
        .unwrap_or_else(|e| exit_with_error(&format!("invalid aggregated signature: {}", e)));

    let valid = verify_batch_signatures(&messages, &signatures, &pubkeys, aggregated.as_ref())
        .unwrap_or_else(|e| exit_with_error(&e));

    if valid {
        println!("BLS batch verification is VALID");
    } else {
        println!("BLS batch verification is INVALID");
        std::process::exit(1);
    }
}

pub(super) fn cmd_batch_smoke(count: usize, output: String) {
    let smoke = run_batch_smoke(count, [77u8; 32]).unwrap_or_else(|e| exit_with_error(&e));

    if output == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&smoke).expect("serialize batch smoke output")
        );
        return;
    }

    println!("=== BLS Batch Smoke ===");
    println!("Count: {}", smoke.count);
    println!("Aggregate matches generated signatures: {}", smoke.aggregate_matches);
    println!("Batch verification valid: {}", smoke.verification_valid);

    if smoke.aggregate_matches && smoke.verification_valid {
        println!("Batch smoke flow PASSED");
    } else {
        println!("Batch smoke flow FAILED");
        std::process::exit(1);
    }
}

fn parse_signatures_payload(value: &str) -> Result<Vec<G1Affine>, String> {
    if super::shared::looks_like_json(value) {
        let input: BatchAggregateInput = serde_json::from_str(value)
            .map_err(|e| format!("invalid json payload: {}", e))?;

        if input.signatures.is_empty() {
            return Err("signatures cannot be empty".to_string());
        }

        return input
            .signatures
            .iter()
            .map(parse_g1_point_struct)
            .collect::<Result<Vec<_>, _>>();
    }

    let raw = super::shared::parse_hex_variable(value, "signatures")?;
    if raw.is_empty() || raw.len() % 128 != 0 {
        return Err(format!(
            "signatures hex payload must be a non-empty multiple of 128 bytes, got {}",
            raw.len()
        ));
    }

    let mut signatures = Vec::with_capacity(raw.len() / 128);
    for chunk in raw.chunks_exact(128) {
        let mut bytes = [0u8; 128];
        bytes.copy_from_slice(chunk);
        signatures.push(bls12_381::decode_g1(&bytes)?);
    }

    Ok(signatures)
}

fn run_batch_smoke(count: usize, seed: [u8; 32]) -> Result<BatchSmokeOut, String> {
    let (messages, _secret_keys, signatures, pubkeys, generated_aggregate) =
        generate_batch_signature_testdata(count, seed)?;

    let computed_aggregate = aggregate_signatures(&signatures)?;
    let aggregate_matches = generated_aggregate == computed_aggregate;

    let verification_valid = verify_batch_signatures(
        &messages,
        &signatures,
        &pubkeys,
        Some(&computed_aggregate),
    )?;

    Ok(BatchSmokeOut {
        count,
        aggregate_matches,
        verification_valid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bls12_381::{
        derive_public_key, generate_batch_signature_testdata, sign_message, verify_batch_signatures,
    };

    #[test]
    fn test_parse_signatures_payload_json() {
        let input = r#"{"signatures":[{"x":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","y":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}]}"#;
        let parsed = parse_signatures_payload(input).expect("should parse signature payload");
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let secret_key = parse_secret_key_or_exit("42");
        let message = "hello";
        let signature = sign_message(secret_key, message.as_bytes());
        let pubkey = derive_public_key(secret_key);

        assert!(verify_signature(&signature, message.as_bytes(), &pubkey));
    }

    #[test]
    fn test_verify_batch_payload_rejects_mismatched_lengths() {
        let input = r#"{"messages":["m1"],"signatures":[],"pubkeys":[]}"#;
        let parsed: BatchVerifyInput = serde_json::from_str(input).expect("json should parse");

        let signatures = parsed
            .signatures
            .iter()
            .map(parse_g1_point_struct)
            .collect::<Result<Vec<_>, _>>()
            .expect("signature parsing should succeed");
        let pubkeys = parsed
            .pubkeys
            .iter()
            .map(parse_g2_point_struct)
            .collect::<Result<Vec<_>, _>>()
            .expect("pubkey parsing should succeed");
        let messages = parsed
            .messages
            .iter()
            .map(|m| m.as_bytes().to_vec())
            .collect::<Vec<_>>();

        let result = verify_batch_signatures(&messages, &signatures, &pubkeys, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_batch_signatures_valid_path() {
        let (messages, _sks, signatures, pubkeys, aggregated) =
            generate_batch_signature_testdata(3, [5u8; 32]).expect("test data should generate");

        let valid = verify_batch_signatures(&messages, &signatures, &pubkeys, Some(&aggregated))
            .expect("verification should run");
        assert!(valid);
    }

    #[test]
    fn test_batch_smoke_happy_path() {
        let smoke = run_batch_smoke(4, [8u8; 32]).expect("smoke should run");
        assert!(smoke.aggregate_matches);
        assert!(smoke.verification_valid);
    }

    #[test]
    fn test_batch_smoke_rejects_zero_count() {
        let smoke = run_batch_smoke(0, [8u8; 32]);
        assert!(smoke.is_err());
    }
}
