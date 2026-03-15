use alloy_primitives::U256;
use ark_bls12_381::{Fr, G1Affine, G2Affine};
use bls12_381::{
    decode_fp2_input, decode_fp_input, decode_g1, decode_g1_msm_input, decode_g2,
    decode_g2_msm_input, decode_scalar,
    codecs::{encode_g1, encode_g2},
};
use serde::{Deserialize, Serialize};
use std::{fs, str::FromStr};

/// Test vector in the Ethereum EIP-2537 format.
#[derive(Serialize)]
pub(crate) struct TestVector {
    #[serde(rename = "Input")]
    pub input: String,    // 0x-prefixed hex
    #[serde(rename = "Expected")]
    pub expected: String, // hex without 0x prefix (matches Ethereum test suite convention)
    #[serde(rename = "Name")]
    pub name: String,
}

/// Write test vectors to `output_file` as pretty JSON, or print hex + decoded structs to stdout.
pub(crate) fn write_or_print_vectors(vectors: &[TestVector], output_file: Option<&str>) {
    match output_file {
        Some(path) => {
            let json = serde_json::to_string_pretty(vectors).expect("serialize test vectors");
            fs::write(path, &json).unwrap_or_else(|e| {
                eprintln!("error writing to {}: {}", path, e);
                std::process::exit(1);
            });
            eprintln!("wrote {} test vector(s) to {}", vectors.len(), path);
        }
        None => {
            for vector in vectors {
                println!("=== {} ===", vector.name);
                println!("Input:    {}", vector.input);
                println!("Expected: 0x{}", vector.expected);
                print_decoded_expected(&vector.expected);
                println!();
            }
        }
    }
}

/// Decode the expected hex output into struct fields based on its length.
fn print_decoded_expected(expected_hex: &str) {
    let hex_str = expected_hex.strip_prefix("0x").unwrap_or(expected_hex);
    let byte_len = hex_str.len() / 2;

    match byte_len {
        // G1 point: x(64 bytes) || y(64 bytes) = 128 bytes
        128 => {
            println!("  Decoded G1 Point:");
            println!("    x: 0x{}", &hex_str[..128]);
            println!("    y: 0x{}", &hex_str[128..]);
        }
        // G2 point: x[0](64) || x[1](64) || y[0](64) || y[1](64) = 256 bytes
        256 => {
            println!("  Decoded G2 Point:");
            println!("    x[0]: 0x{}", &hex_str[..128]);
            println!("    x[1]: 0x{}", &hex_str[128..256]);
            println!("    y[0]: 0x{}", &hex_str[256..384]);
            println!("    y[1]: 0x{}", &hex_str[384..]);
        }
        _ => {}
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct G1PointInput {
    pub x: String,
    pub y: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct G2PointInput {
    pub x: [String; 2],
    pub y: [String; 2],
}

#[derive(Deserialize)]
pub(crate) struct FpInput {
    pub value: String,
}

#[derive(Deserialize)]
pub(crate) struct Fp2Input {
    pub value: [String; 2],
}

#[derive(Deserialize)]
pub(crate) struct G1MsmInput {
    pub points: Vec<G1PointInput>,
    pub scalars: Vec<ScalarInput>,
}

#[derive(Deserialize)]
pub(crate) struct G2MsmInput {
    pub points: Vec<G2PointInput>,
    pub scalars: Vec<ScalarInput>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum ScalarInput {
    String(String),
    Number(serde_json::Number),
}

#[derive(Serialize)]
pub(crate) struct G1PointOut {
    pub x: String,
    pub y: String,
}

#[derive(Serialize)]
pub(crate) struct G2PointOut {
    pub x: [String; 2],
    pub y: [String; 2],
}

pub(crate) fn exit_with_error(message: &str) -> ! {
    eprintln!("Error: {}", message);
    std::process::exit(1);
}

pub(crate) fn looks_like_json(value: &str) -> bool {
    value.trim_start().starts_with('{')
}

pub(crate) fn parse_hex_variable(value: &str, name: &str) -> Result<Vec<u8>, String> {
    let trimmed = value.trim();
    let stripped = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);

    hex::decode(stripped).map_err(|error| format!("{} is not valid hex: {}", name, error))
}

pub(crate) fn parse_hex_fixed<const N: usize>(value: &str, name: &str) -> Result<[u8; N], String> {
    let bytes = parse_hex_variable(value, name)?;

    if bytes.len() != N {
        return Err(format!(
            "{} must be {} bytes ({} hex chars), got {} bytes",
            name,
            N,
            N * 2,
            bytes.len()
        ));
    }

    let mut fixed = [0u8; N];
    fixed.copy_from_slice(&bytes);
    Ok(fixed)
}

pub(crate) fn parse_scalar_32(value: &ScalarInput) -> Result<[u8; 32], String> {
    match value {
        ScalarInput::String(s) => parse_scalar_str(s),
        ScalarInput::Number(n) => parse_scalar_str(&n.to_string()),
    }
}

pub(crate) fn parse_scalar_str(value: &str) -> Result<[u8; 32], String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("scalar cannot be empty".to_string());
    }

    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        let raw = parse_hex_variable(trimmed, "scalar")?;
        if raw.len() > 32 {
            return Err(format!("scalar cannot exceed 32 bytes, got {}", raw.len()));
        }
        let mut out = [0u8; 32];
        out[32 - raw.len()..].copy_from_slice(&raw);
        return Ok(out);
    }

    let parsed = U256::from_str(trimmed).map_err(|_| "invalid decimal scalar".to_string())?;
    Ok(parsed.to_be_bytes())
}

pub(crate) fn parse_g1_point(value: &str) -> Result<G1Affine, String> {
    if looks_like_json(value) {
        let input: G1PointInput = serde_json::from_str(value)
            .map_err(|error| format!("invalid G1 JSON input: {}", error))?;
        return parse_g1_point_struct(&input);
    }

    let bytes = parse_hex_fixed::<128>(value, "g1 point")?;
    decode_g1(&bytes)
}

pub(crate) fn parse_g2_point(value: &str) -> Result<G2Affine, String> {
    if looks_like_json(value) {
        let input: G2PointInput = serde_json::from_str(value)
            .map_err(|error| format!("invalid G2 JSON input: {}", error))?;
        return parse_g2_point_struct(&input);
    }

    let bytes = parse_hex_fixed::<256>(value, "g2 point")?;
    decode_g2(&bytes)
}

pub(crate) fn parse_g1_point_struct(point: &G1PointInput) -> Result<G1Affine, String> {
    let mut bytes = [0u8; 128];
    bytes[..64].copy_from_slice(&parse_hex_fixed::<64>(&point.x, "x")?);
    bytes[64..].copy_from_slice(&parse_hex_fixed::<64>(&point.y, "y")?);
    decode_g1(&bytes)
}

pub(crate) fn parse_g2_point_struct(point: &G2PointInput) -> Result<G2Affine, String> {
    let mut bytes = [0u8; 256];
    bytes[..64].copy_from_slice(&parse_hex_fixed::<64>(&point.x[0], "x[0]")?);
    bytes[64..128].copy_from_slice(&parse_hex_fixed::<64>(&point.x[1], "x[1]")?);
    bytes[128..192].copy_from_slice(&parse_hex_fixed::<64>(&point.y[0], "y[0]")?);
    bytes[192..].copy_from_slice(&parse_hex_fixed::<64>(&point.y[1], "y[1]")?);
    decode_g2(&bytes)
}

pub(crate) fn parse_g1_msm_pairs(value: &str) -> Result<Vec<(G1Affine, Fr)>, String> {
    if looks_like_json(value) {
        let msm: G1MsmInput = serde_json::from_str(value)
            .map_err(|error| format!("invalid g1 msm json input: {}", error))?;

        if msm.points.is_empty() {
            return Err("points cannot be empty".to_string());
        }

        if msm.points.len() != msm.scalars.len() {
            return Err(format!(
                "mismatched points/scalars length: {} points, {} scalars",
                msm.points.len(),
                msm.scalars.len()
            ));
        }

        let mut pairs = Vec::with_capacity(msm.points.len());
        for (point, scalar) in msm.points.iter().zip(msm.scalars.iter()) {
            let point = parse_g1_point_struct(point)?;
            let scalar = decode_scalar(&parse_scalar_32(scalar)?)?;
            pairs.push((point, scalar));
        }
        return Ok(pairs);
    }

    let raw = parse_hex_variable(value, "g1 msm data")?;
    decode_g1_msm_input(&raw)
}

pub(crate) fn parse_g2_msm_pairs(value: &str) -> Result<Vec<(G2Affine, Fr)>, String> {
    if looks_like_json(value) {
        let msm: G2MsmInput = serde_json::from_str(value)
            .map_err(|error| format!("invalid g2 msm json input: {}", error))?;

        if msm.points.is_empty() {
            return Err("points cannot be empty".to_string());
        }

        if msm.points.len() != msm.scalars.len() {
            return Err(format!(
                "mismatched points/scalars length: {} points, {} scalars",
                msm.points.len(),
                msm.scalars.len()
            ));
        }

        let mut pairs = Vec::with_capacity(msm.points.len());
        for (point, scalar) in msm.points.iter().zip(msm.scalars.iter()) {
            let point = parse_g2_point_struct(point)?;
            let scalar = decode_scalar(&parse_scalar_32(scalar)?)?;
            pairs.push((point, scalar));
        }
        return Ok(pairs);
    }

    let raw = parse_hex_variable(value, "g2 msm data")?;
    decode_g2_msm_input(&raw)
}

pub(crate) fn parse_fp_input(value: Option<String>, random: [u8; 64]) -> Result<[u8; 64], String> {
    let Some(value) = value else {
        return Ok(random);
    };

    if looks_like_json(&value) {
        let input: FpInput = serde_json::from_str(&value)
            .map_err(|error| format!("invalid fp json input: {}", error))?;
        return parse_fp_hex_64(&input.value);
    }

    parse_fp_hex_64(&value)
}

pub(crate) fn parse_fp2_input(value: Option<String>, random: [u8; 128]) -> Result<[u8; 128], String> {
    let Some(value) = value else {
        return Ok(random);
    };

    if looks_like_json(&value) {
        let input: Fp2Input = serde_json::from_str(&value)
            .map_err(|error| format!("invalid fp2 json input: {}", error))?;
        let c0 = parse_fp_hex_64(&input.value[0])?;
        let c1 = parse_fp_hex_64(&input.value[1])?;
        let mut output = [0u8; 128];
        output[..64].copy_from_slice(&c0);
        output[64..].copy_from_slice(&c1);
        return Ok(output);
    }

    parse_hex_fixed::<128>(&value, "fp2 input")
}

pub(crate) fn parse_fp_hex_64(value: &str) -> Result<[u8; 64], String> {
    let raw = parse_hex_variable(value, "fp value")?;
    if raw.len() == 64 {
        let mut out = [0u8; 64];
        out.copy_from_slice(&raw);
        return Ok(out);
    }

    if raw.len() == 32 {
        let mut out = [0u8; 64];
        out[32..].copy_from_slice(&raw);
        return Ok(out);
    }

    Err(format!(
        "fp value must be 32 or 64 bytes, got {} bytes",
        raw.len()
    ))
}

pub(crate) fn g1_to_output(point: G1Affine) -> G1PointOut {
    let encoded = encode_g1(point);
    G1PointOut {
        x: format!("0x{}", hex::encode(&encoded[..64])),
        y: format!("0x{}", hex::encode(&encoded[64..])),
    }
}

pub(crate) fn g2_to_output(point: G2Affine) -> G2PointOut {
    let encoded = encode_g2(&point);
    G2PointOut {
        x: [
            format!("0x{}", hex::encode(&encoded[..64])),
            format!("0x{}", hex::encode(&encoded[64..128])),
        ],
        y: [
            format!("0x{}", hex::encode(&encoded[128..192])),
            format!("0x{}", hex::encode(&encoded[192..])),
        ],
    }
}

pub(crate) fn parse_secret_key_or_exit(secret_key: &str) -> Fr {
    let bytes = parse_scalar_str(secret_key)
        .unwrap_or_else(|e| exit_with_error(&format!("invalid secret_key: {}", e)));
    decode_scalar(&bytes)
        .unwrap_or_else(|e| exit_with_error(&format!("invalid secret_key: {}", e)))
}

pub(crate) fn parse_g1_point_or_exit(value: &str, name: &str) -> G1Affine {
    parse_g1_point(value).unwrap_or_else(|e| exit_with_error(&format!("invalid {}: {}", name, e)))
}

pub(crate) fn parse_g2_point_or_exit(value: &str, name: &str) -> G2Affine {
    parse_g2_point(value).unwrap_or_else(|e| exit_with_error(&format!("invalid {}: {}", name, e)))
}

pub(crate) fn decode_fp_input_or_exit(bytes: &[u8; 64]) -> ark_bls12_381::Fq {
    decode_fp_input(bytes).unwrap_or_else(|e| exit_with_error(&e))
}

pub(crate) fn decode_fp2_input_or_exit(bytes: &[u8; 128]) -> ark_bls12_381::Fq2 {
    decode_fp2_input(bytes).unwrap_or_else(|e| exit_with_error(&e))
}
