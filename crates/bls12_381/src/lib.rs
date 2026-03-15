pub mod utils;
pub mod codecs;

use ark_bls12_381::{Bls12_381, Fq, Fq2, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup, Group};
use ark_std::UniformRand;
use ark_ff::{Zero, PrimeField};
use ark_ec::hashing::{curve_maps::wb::WBMap, map_to_curve_hasher::MapToCurve};
use rand::thread_rng;
use utils::seeded_rng;
use codecs::{encode_g1, encode_g2, encode_scalar};
use ark_bls12_381::g1::Config as G1Config;
use ark_bls12_381::g2::Config as G2Config;

use crate::{codecs::{encode_fp, encode_fp2}, utils::{generate_aggregated_pairing_inputs, generate_individual_pairing_inputs, generate_signatures}};

pub fn generate_random_g1_point() -> G1Affine {
    G1Projective::rand(&mut thread_rng()).into_affine()
}

pub fn generate_random_g2_point() -> G2Affine {
    G2Projective::rand(&mut thread_rng()).into_affine()
}

pub fn add_g1_points(point_a: G1Affine, point_b: G1Affine) -> G1Affine {
    (point_a + point_b).into_affine()
}

pub fn add_g2_points(point_a: G2Affine, point_b: G2Affine) -> G2Affine {
    (point_a + point_b).into_affine()
}

pub fn derive_public_key(secret_key: Fr) -> G2Affine {
    (G2Projective::generator() * secret_key).into_affine()
}

pub fn sign_message(secret_key: Fr, message: &[u8]) -> G1Affine {
    let hash = utils::hash_to_g1(message);
    (hash * secret_key).into_affine()
}

pub fn verify_signature(signature: &G1Affine, message: &[u8], pubkey: &G2Affine) -> bool {
    let hash = utils::hash_to_g1(message).into_affine();
    let left = Bls12_381::pairing(*signature, G2Affine::generator());
    let right = Bls12_381::pairing(hash, *pubkey);

    left == right
}

pub fn aggregate_signatures(signatures: &[G1Affine]) -> Result<G1Affine, String> {
    if signatures.is_empty() {
        return Err("at least one signature is required for aggregation".to_string());
    }

    let mut acc = G1Projective::zero();
    for signature in signatures {
        acc += *signature;
    }

    Ok(acc.into_affine())
}

pub fn verify_batch_signatures(
    messages: &[Vec<u8>],
    signatures: &[G1Affine],
    pubkeys: &[G2Affine],
    aggregated_signature: Option<&G1Affine>,
) -> Result<bool, String> {
    if messages.is_empty() {
        return Err("messages cannot be empty".to_string());
    }

    if messages.len() != signatures.len() || messages.len() != pubkeys.len() {
        return Err(format!(
            "mismatched lengths: messages={}, signatures={}, pubkeys={}",
            messages.len(),
            signatures.len(),
            pubkeys.len()
        ));
    }

    for i in 0..messages.len() {
        if !verify_signature(&signatures[i], &messages[i], &pubkeys[i]) {
            return Ok(false);
        }
    }

    if let Some(expected_aggregated) = aggregated_signature {
        let computed = aggregate_signatures(signatures)?;
        if computed != *expected_aggregated {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn generate_batch_signature_testdata(
    k: usize,
    seed: [u8; 32],
) -> Result<(Vec<Vec<u8>>, Vec<Fr>, Vec<G1Affine>, Vec<G2Affine>, G1Affine), String> {
    if k == 0 {
        return Err("count must be greater than zero".to_string());
    }

    let mut rng = seeded_rng(seed);
    let mut messages = Vec::with_capacity(k);
    let mut secret_keys = Vec::with_capacity(k);
    let mut signatures = Vec::with_capacity(k);
    let mut pubkeys = Vec::with_capacity(k);

    for i in 0..k {
        let secret_key = Fr::rand(&mut rng);
        let message = format!("BLS batch message {}", i).into_bytes();
        let signature = sign_message(secret_key, &message);
        let pubkey = derive_public_key(secret_key);

        secret_keys.push(secret_key);
        messages.push(message);
        signatures.push(signature);
        pubkeys.push(pubkey);
    }

    let aggregated = aggregate_signatures(&signatures)?;

    Ok((messages, secret_keys, signatures, pubkeys, aggregated))
}

pub fn decode_scalar(bytes: &[u8; 32]) -> Result<Fr, String> {
    let scalar = Fr::from_be_bytes_mod_order(bytes);

    if encode_scalar(scalar) != *bytes {
        return Err("invalid Fr scalar encoding".to_string());
    }

    Ok(scalar)
}

pub fn decode_g1(bytes: &[u8; 128]) -> Result<G1Affine, String> {
    if bytes.iter().all(|byte| *byte == 0) {
        return Ok(G1Affine::zero());
    }

    let mut x_bytes = [0u8; 64];
    let mut y_bytes = [0u8; 64];
    x_bytes.copy_from_slice(&bytes[..64]);
    y_bytes.copy_from_slice(&bytes[64..]);

    let point = G1Affine::new_unchecked(decode_fq(&x_bytes)?, decode_fq(&y_bytes)?);

    if !point.is_on_curve() {
        return Err("point is not on the G1 curve".to_string());
    }

    if !point.is_in_correct_subgroup_assuming_on_curve() {
        return Err("point is not in the G1 prime-order subgroup".to_string());
    }

    Ok(point)
}

pub fn decode_g2(bytes: &[u8; 256]) -> Result<G2Affine, String> {
    if bytes.iter().all(|byte| *byte == 0) {
        return Ok(G2Affine::zero());
    }

    let mut x_bytes = [0u8; 128];
    let mut y_bytes = [0u8; 128];
    x_bytes.copy_from_slice(&bytes[..128]);
    y_bytes.copy_from_slice(&bytes[128..]);

    let point = G2Affine::new_unchecked(decode_fp2_value(&x_bytes)?, decode_fp2_value(&y_bytes)?);

    if !point.is_on_curve() {
        return Err("point is not on the G2 curve".to_string());
    }

    if !point.is_in_correct_subgroup_assuming_on_curve() {
        return Err("point is not in the G2 prime-order subgroup".to_string());
    }

    Ok(point)
}

fn decode_fq(bytes: &[u8; 64]) -> Result<Fq, String> {
    let value = Fq::from_be_bytes_mod_order(bytes);

    if codecs::encode_fq(value) != *bytes {
        return Err("invalid Fq encoding".to_string());
    }

    Ok(value)
}

fn decode_fp(bytes: &[u8; 64]) -> Result<Fq, String> {
    let value = Fq::from_be_bytes_mod_order(&bytes[16..]);

    if encode_fp(&value) != *bytes {
        return Err("invalid Fp encoding".to_string());
    }

    Ok(value)
}

pub fn decode_fp_input(bytes: &[u8; 64]) -> Result<Fq, String> {
    decode_fp(bytes)
}

fn decode_fp2_value(bytes: &[u8; 128]) -> Result<Fq2, String> {
    let mut c1_bytes = [0u8; 64];
    let mut c0_bytes = [0u8; 64];
    c1_bytes.copy_from_slice(&bytes[..64]);
    c0_bytes.copy_from_slice(&bytes[64..]);

    Ok(Fq2 {
        c0: decode_fp(&c0_bytes)?,
        c1: decode_fp(&c1_bytes)?,
    })
}

pub fn decode_fp2_input(bytes: &[u8; 128]) -> Result<Fq2, String> {
    decode_fp2_value(bytes)
}

pub fn map_fp_to_g1(fp: Fq) -> Result<G1Affine, String> {
    let wb_map = WBMap::<G1Config>::new().map_err(|e| format!("wb map init failed: {}", e))?;
    let mapped = wb_map
        .map_to_curve(fp)
        .map_err(|e| format!("map_to_curve failed: {}", e))?;

    Ok(mapped.clear_cofactor())
}

pub fn map_fp2_to_g2(fp2: Fq2) -> Result<G2Affine, String> {
    let wb_map = WBMap::<G2Config>::new().map_err(|e| format!("wb map init failed: {}", e))?;
    let mapped = wb_map
        .map_to_curve(fp2)
        .map_err(|e| format!("map_to_curve failed: {}", e))?;

    Ok(mapped.clear_cofactor())
}

pub fn generate_random_fp_input() -> [u8; 64] {
    let value = Fq::rand(&mut thread_rng());
    encode_fp(&value)
}

pub fn generate_random_fp2_input() -> [u8; 128] {
    let value = Fq2 {
        c0: Fq::rand(&mut thread_rng()),
        c1: Fq::rand(&mut thread_rng()),
    };
    encode_fp2(&value)
}

pub fn encode_g1_msm_input(pairs: &[(G1Affine, Fr)]) -> Vec<u8> {
    let mut input = Vec::with_capacity(pairs.len() * 160);

    for (point, scalar) in pairs {
        input.extend_from_slice(&encode_g1(*point));
        input.extend_from_slice(&encode_scalar(*scalar));
    }

    input
}

pub fn encode_g2_msm_input(pairs: &[(G2Affine, Fr)]) -> Vec<u8> {
    let mut input = Vec::with_capacity(pairs.len() * 288);

    for (point, scalar) in pairs {
        input.extend_from_slice(&encode_g2(point));
        input.extend_from_slice(&encode_scalar(*scalar));
    }

    input
}

pub fn decode_g1_msm_input(input: &[u8]) -> Result<Vec<(G1Affine, Fr)>, String> {
    if input.is_empty() {
        return Err("g1 msm input cannot be empty".to_string());
    }

    if input.len() % 160 != 0 {
        return Err(format!(
            "invalid g1 msm input length: expected multiple of 160 bytes, got {}",
            input.len()
        ));
    }

    let mut pairs = Vec::with_capacity(input.len() / 160);

    for chunk in input.chunks_exact(160) {
        let mut point_bytes = [0u8; 128];
        let mut scalar_bytes = [0u8; 32];
        point_bytes.copy_from_slice(&chunk[..128]);
        scalar_bytes.copy_from_slice(&chunk[128..]);

        pairs.push((decode_g1(&point_bytes)?, decode_scalar(&scalar_bytes)?));
    }

    Ok(pairs)
}

pub fn decode_g2_msm_input(input: &[u8]) -> Result<Vec<(G2Affine, Fr)>, String> {
    if input.is_empty() {
        return Err("g2 msm input cannot be empty".to_string());
    }

    if input.len() % 288 != 0 {
        return Err(format!(
            "invalid g2 msm input length: expected multiple of 288 bytes, got {}",
            input.len()
        ));
    }

    let mut pairs = Vec::with_capacity(input.len() / 288);

    for chunk in input.chunks_exact(288) {
        let mut point_bytes = [0u8; 256];
        let mut scalar_bytes = [0u8; 32];
        point_bytes.copy_from_slice(&chunk[..256]);
        scalar_bytes.copy_from_slice(&chunk[256..]);

        pairs.push((decode_g2(&point_bytes)?, decode_scalar(&scalar_bytes)?));
    }

    Ok(pairs)
}

pub fn compute_g1_msm(pairs: &[(G1Affine, Fr)]) -> Result<G1Affine, String> {
    if pairs.is_empty() {
        return Err("g1 msm requires at least one point-scalar pair".to_string());
    }

    let mut acc = G1Projective::zero();
    for (point, scalar) in pairs {
        acc += point.mul_bigint(scalar.into_bigint());
    }

    Ok(acc.into_affine())
}

pub fn compute_g2_msm(pairs: &[(G2Affine, Fr)]) -> Result<G2Affine, String> {
    if pairs.is_empty() {
        return Err("g2 msm requires at least one point-scalar pair".to_string());
    }

    let mut acc = G2Projective::zero();
    for (point, scalar) in pairs {
        acc += point.mul_bigint(scalar.into_bigint());
    }

    Ok(acc.into_affine())
}

pub fn generate_g1_msm_testdata(k: usize) -> Result<(Vec<(G1Affine, Fr)>, G1Affine), String> {
    if k == 0 {
        return Err("pairs must be greater than zero".to_string());
    }

    let mut rng = seeded_rng([56_u8; 32]);
    let mut pairs = Vec::with_capacity(k);

    for _ in 0..k {
        pairs.push((
            G1Projective::rand(&mut rng).into_affine(),
            Fr::rand(&mut rng),
        ));
    }

    let result = compute_g1_msm(&pairs)?;
    Ok((pairs, result))
}

pub fn generate_g2_msm_testdata(k: usize) -> Result<(Vec<(G2Affine, Fr)>, G2Affine), String> {
    if k == 0 {
        return Err("pairs must be greater than zero".to_string());
    }

    let mut rng = seeded_rng([65_u8; 32]);
    let mut pairs = Vec::with_capacity(k);

    for _ in 0..k {
        pairs.push((
            G2Projective::rand(&mut rng).into_affine(),
            Fr::rand(&mut rng),
        ));
    }

    let result = compute_g2_msm(&pairs)?;
    Ok((pairs, result))
}

pub fn generate_g1_add_params() -> [u8; 128] {
    let mut rng = seeded_rng([62u8; 32]);

    let point_a = G1Projective::rand(&mut rng);
    let point_b = G1Projective::rand(&mut rng);
    let result = (point_a + point_b).into_affine();

    let mut input = Vec::new();
    input.extend_from_slice(&encode_g1(point_a.into_affine()));
    input.extend_from_slice(&encode_g1(point_b.into_affine()));

    let output = encode_g1(result);
    output
}

pub fn generate_g2_add_params() -> [u8; 256] {
    let mut rng = seeded_rng([45_u8; 32]);

    let p = G2Projective::rand(&mut rng).into_affine();
    let q = G2Projective::rand(&mut rng).into_affine();

    let result = (p + q).into_affine();

    let mut input = Vec::new();

    input.extend_from_slice(&encode_g2(&p));
    input.extend_from_slice(&encode_g2(&q));

    let output = encode_g2(&result);
    output
}

pub fn generate_g1_msm_params(k: usize) -> G1Affine {
    let (pairs, result) = generate_g1_msm_testdata(k).expect("valid g1 msm test data");
    let inputs = encode_g1_msm_input(&pairs);
    println!("MSM input point for index: 0x{}", hex::encode(&inputs));

    result
}

pub fn generate_g2_msm_params(k: usize) -> G2Affine {
    let (pairs, result) = generate_g2_msm_testdata(k).expect("valid g2 msm test data");
    let _inputs = encode_g2_msm_input(&pairs);

    result
}


pub fn generate_pairing_pairs(k: usize, aggregate: bool) -> Vec<u8> {
    let data = generate_signatures(k, [45_u8; 32]);
    println!(
        "Generated {} signatures, total sigs {}",
        k,
        data.signatures.len()
    );

    let pairs = if aggregate {
        generate_aggregated_pairing_inputs(&data)
    } else {
        generate_individual_pairing_inputs(&data)
    };
    
    let mut inputs = Vec::new();
    for (p, q) in pairs.iter() {
        inputs.extend_from_slice(&encode_g1(*p));
        inputs.extend_from_slice(&encode_g2(q));
    }

    inputs
}

pub fn generate_mapped_g1_to_fp(n: usize) -> Vec<([u8; 64], [u8; 128])> {
    let mut vectors = Vec::with_capacity(n);

    for _ in 0..n {
        let fp = Fq::rand(&mut seeded_rng([35u8; 32]));
        let input = encode_fp(&fp);
        let p = {
            let wb_map = WBMap::<G1Config>::new().expect("WB map initialization should succeed");
            let mapped = wb_map.map_to_curve(fp).expect("Mapping should succeed");

            mapped.clear_cofactor()
        };
        let output = encode_g1(p);

        vectors.push((input, output));
    }

    vectors
}

pub fn generate_mapped_g2_to_fp2(n: usize) -> Vec<([u8; 128], [u8; 256])> {
    let mut vectors = Vec::with_capacity(n);

    for _ in 0..n {
        let fp2 = Fq2 {
            c0: Fq::rand(&mut seeded_rng([63_u8; 32])),
            c1: Fq::rand(&mut seeded_rng([64_u8; 32])),
        };
        let input = encode_fp2(&fp2);
        let p = {
            let wb_map = WBMap::<G2Config>::new().expect("WB map initialization should succeed");
            let mapped = wb_map.map_to_curve(fp2).expect("Mapping should succeed");

            mapped.clear_cofactor()
        };
        let output = encode_g2(&p);

        print!("MapToCurve input: 0x{} \n", hex::encode(input));
        print!(" MapToCurve output: 0x{} \n", hex::encode(output));

        vectors.push((input, output));
    }

    vectors
}

/// Generate `count` deterministic G1-add test vectors.
/// Returns `(input_bytes [256], expected_bytes [128])` per vector.
pub fn generate_g1_add_testdata(count: usize) -> Vec<([u8; 256], [u8; 128])> {
    (0..count)
        .map(|i| {
            let mut seed = [0xA1u8; 32];
            seed[0] = (i & 0xFF) as u8;
            seed[1] = ((i >> 8) & 0xFF) as u8;
            let mut rng = seeded_rng(seed);
            let a = G1Projective::rand(&mut rng).into_affine();
            let b = G1Projective::rand(&mut rng).into_affine();
            let result = add_g1_points(a, b);
            let mut input = [0u8; 256];
            input[..128].copy_from_slice(&encode_g1(a));
            input[128..].copy_from_slice(&encode_g1(b));
            (input, encode_g1(result))
        })
        .collect()
}

/// Generate `count` deterministic G2-add test vectors.
/// Returns `(input_bytes [512], expected_bytes [256])` per vector.
pub fn generate_g2_add_testdata(count: usize) -> Vec<([u8; 512], [u8; 256])> {
    (0..count)
        .map(|i| {
            let mut seed = [0xB2u8; 32];
            seed[0] = (i & 0xFF) as u8;
            seed[1] = ((i >> 8) & 0xFF) as u8;
            let mut rng = seeded_rng(seed);
            let a = G2Projective::rand(&mut rng).into_affine();
            let b = G2Projective::rand(&mut rng).into_affine();
            let result = add_g2_points(a, b);
            let mut input = [0u8; 512];
            input[..256].copy_from_slice(&encode_g2(&a));
            input[256..].copy_from_slice(&encode_g2(&b));
            (input, encode_g2(&result))
        })
        .collect()
}

/// Generate `count` deterministic G1-MSM test vectors each with `pairs` point-scalar pairs.
/// Returns `(input_bytes, expected_bytes [128])` per vector.
pub fn generate_g1_msm_testdata_n(
    count: usize,
    pairs: usize,
) -> Result<Vec<(Vec<u8>, [u8; 128])>, String> {
    if pairs == 0 {
        return Err("pairs must be greater than zero".to_string());
    }
    (0..count)
        .map(|i| {
            let mut seed = [0x56u8; 32];
            seed[0] = (i & 0xFF) as u8;
            seed[1] = ((i >> 8) & 0xFF) as u8;
            let mut rng = seeded_rng(seed);
            let pairs_data: Vec<(G1Affine, Fr)> = (0..pairs)
                .map(|_| (G1Projective::rand(&mut rng).into_affine(), Fr::rand(&mut rng)))
                .collect();
            let input = encode_g1_msm_input(&pairs_data);
            let result = compute_g1_msm(&pairs_data)?;
            Ok((input, encode_g1(result)))
        })
        .collect()
}

/// Generate `count` deterministic G2-MSM test vectors each with `pairs` point-scalar pairs.
/// Returns `(input_bytes, expected_bytes [256])` per vector.
pub fn generate_g2_msm_testdata_n(
    count: usize,
    pairs: usize,
) -> Result<Vec<(Vec<u8>, [u8; 256])>, String> {
    if pairs == 0 {
        return Err("pairs must be greater than zero".to_string());
    }
    (0..count)
        .map(|i| {
            let mut seed = [0x65u8; 32];
            seed[0] = (i & 0xFF) as u8;
            seed[1] = ((i >> 8) & 0xFF) as u8;
            let mut rng = seeded_rng(seed);
            let pairs_data: Vec<(G2Affine, Fr)> = (0..pairs)
                .map(|_| (G2Projective::rand(&mut rng).into_affine(), Fr::rand(&mut rng)))
                .collect();
            let input = encode_g2_msm_input(&pairs_data);
            let result = compute_g2_msm(&pairs_data)?;
            Ok((input, encode_g2(&result)))
        })
        .collect()
}

/// Generate `count` deterministic MapFp→G1 test vectors.
/// Returns `(input_bytes [64], expected_bytes [128])` per vector.
pub fn generate_map_fp_testdata(count: usize) -> Result<Vec<([u8; 64], [u8; 128])>, String> {
    (0..count)
        .map(|i| {
            let mut seed = [0x35u8; 32];
            seed[0] = (i & 0xFF) as u8;
            seed[1] = ((i >> 8) & 0xFF) as u8;
            let fp = Fq::rand(&mut seeded_rng(seed));
            let input = encode_fp(&fp);
            let mapped = map_fp_to_g1(fp)?;
            Ok((input, encode_g1(mapped)))
        })
        .collect()
}

/// Generate `count` deterministic MapFp2→G2 test vectors.
/// Returns `(input_bytes [128], expected_bytes [256])` per vector.
pub fn generate_map_fp2_testdata(count: usize) -> Result<Vec<([u8; 128], [u8; 256])>, String> {
    (0..count)
        .map(|i| {
            let mut seed_c0 = [0x63u8; 32];
            let mut seed_c1 = [0x64u8; 32];
            seed_c0[0] = (i & 0xFF) as u8;
            seed_c1[0] = (i & 0xFF) as u8;
            let fp2 = Fq2 {
                c0: Fq::rand(&mut seeded_rng(seed_c0)),
                c1: Fq::rand(&mut seeded_rng(seed_c1)),
            };
            let input = encode_fp2(&fp2);
            let mapped = map_fp2_to_g2(fp2)?;
            Ok((input, encode_g2(&mapped)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_g1_encode_decode_roundtrip() {
        let point = generate_random_g1_point();
        let encoded = encode_g1(point);

        let decoded = decode_g1(&encoded).expect("g1 decode should succeed");

        assert_eq!(encode_g1(decoded), encoded);
    }

    #[test]
    fn test_g2_encode_decode_roundtrip() {
        let point = generate_random_g2_point();
        let encoded = encode_g2(&point);

        let decoded = decode_g2(&encoded).expect("g2 decode should succeed");

        assert_eq!(encode_g2(&decoded), encoded);
    }

    #[test]
    fn test_add_g1_points_matches_projective_addition() {
        let point_a = generate_random_g1_point();
        let point_b = generate_random_g1_point();

        let result = add_g1_points(point_a, point_b);

        assert_eq!(result, (point_a + point_b).into_affine());
    }

    #[test]
    fn test_add_g2_points_matches_projective_addition() {
        let point_a = generate_random_g2_point();
        let point_b = generate_random_g2_point();

        let result = add_g2_points(point_a, point_b);

        assert_eq!(result, (point_a + point_b).into_affine());
    }

    #[test]
    fn test_g1_msm_encode_decode_roundtrip() {
        let (pairs, _) = generate_g1_msm_testdata(3).expect("testdata should build");
        let encoded = encode_g1_msm_input(&pairs);
        let decoded = decode_g1_msm_input(&encoded).expect("decode should succeed");

        assert_eq!(decoded.len(), pairs.len());
        assert_eq!(compute_g1_msm(&decoded).unwrap(), compute_g1_msm(&pairs).unwrap());
    }

    #[test]
    fn test_g2_msm_encode_decode_roundtrip() {
        let (pairs, _) = generate_g2_msm_testdata(3).expect("testdata should build");
        let encoded = encode_g2_msm_input(&pairs);
        let decoded = decode_g2_msm_input(&encoded).expect("decode should succeed");

        assert_eq!(decoded.len(), pairs.len());
        assert_eq!(compute_g2_msm(&decoded).unwrap(), compute_g2_msm(&pairs).unwrap());
    }

    #[test]
    fn test_map_fp_to_g1_returns_valid_point() {
        let fp = decode_fp_input(&generate_random_fp_input()).expect("fp input should decode");
        let point = map_fp_to_g1(fp).expect("map should succeed");

        assert!(point.is_on_curve());
        assert!(point.is_in_correct_subgroup_assuming_on_curve());
    }

    #[test]
    fn test_map_fp2_to_g2_returns_valid_point() {
        let fp2 = decode_fp2_input(&generate_random_fp2_input()).expect("fp2 input should decode");
        let point = map_fp2_to_g2(fp2).expect("map should succeed");

        assert!(point.is_on_curve());
        assert!(point.is_in_correct_subgroup_assuming_on_curve());
    }

    #[test]
    fn test_bls_sign_and_verify_roundtrip() {
        let sk = Fr::from(42u64);
        let msg = b"hello bls";
        let sig = sign_message(sk, msg);
        let pk = derive_public_key(sk);

        assert!(verify_signature(&sig, msg, &pk));
    }

    #[test]
    fn test_bls_verify_fails_for_wrong_message() {
        let sk = Fr::from(7u64);
        let sig = sign_message(sk, b"correct");
        let pk = derive_public_key(sk);

        assert!(!verify_signature(&sig, b"wrong", &pk));
    }

    #[test]
    fn test_batch_verify_with_aggregate() {
        let (messages, _sks, signatures, pubkeys, aggregated) =
            generate_batch_signature_testdata(4, [21u8; 32]).expect("batch generation should work");

        let valid = verify_batch_signatures(&messages, &signatures, &pubkeys, Some(&aggregated))
            .expect("batch verify should run");
        assert!(valid);
    }

    #[test]
    fn test_batch_verify_rejects_bad_aggregate() {
        let (messages, _sks, signatures, pubkeys, mut aggregated) =
            generate_batch_signature_testdata(3, [11u8; 32]).expect("batch generation should work");

        // Corrupt aggregate with another valid point.
        aggregated = add_g1_points(aggregated, generate_random_g1_point());

        let valid = verify_batch_signatures(&messages, &signatures, &pubkeys, Some(&aggregated))
            .expect("batch verify should run");
        assert!(!valid);
    }
}