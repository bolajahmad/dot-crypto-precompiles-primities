use bls12_381::{
    compute_g1_msm, compute_g2_msm,
    generate_g1_msm_testdata_n, generate_g2_msm_testdata_n,
};

use super::{
    points::{print_g1_point, print_g2_point},
    shared::{
        exit_with_error, parse_g1_msm_pairs, parse_g2_msm_pairs,
        TestVector, write_or_print_vectors,
    },
};


pub(super) fn cmd_g1_msm_testdata(pairs: usize, count: usize, output_file: Option<String>) {
    let vectors = generate_g1_msm_testdata_n(count, pairs)
        .unwrap_or_else(|e| exit_with_error(&e));
    let tv: Vec<TestVector> = vectors
        .into_iter()
        .enumerate()
        .map(|(i, (input, expected))| TestVector {
            input: format!("0x{}", hex::encode(&input)),
            expected: hex::encode(expected),
            name: format!("bls12381_g1_msm {}", i + 1),
        })
        .collect();
    write_or_print_vectors(&tv, output_file.as_deref());
}

pub(super) fn cmd_g2_msm_testdata(pairs: usize, count: usize, output_file: Option<String>) {
    let vectors = generate_g2_msm_testdata_n(count, pairs)
        .unwrap_or_else(|e| exit_with_error(&e));
    let tv: Vec<TestVector> = vectors
        .into_iter()
        .enumerate()
        .map(|(i, (input, expected))| TestVector {
            input: format!("0x{}", hex::encode(&input)),
            expected: hex::encode(expected),
            name: format!("bls12381_g2_msm {}", i + 1),
        })
        .collect();
    write_or_print_vectors(&tv, output_file.as_deref());
}

pub(super) fn cmd_g1_msm(data: String) {
    let pairs_data = parse_g1_msm_pairs(&data)
        .unwrap_or_else(|e| exit_with_error(&format!("invalid g1 msm data: {}", e)));
    let result = compute_g1_msm(&pairs_data).unwrap_or_else(|e| exit_with_error(&e));

    println!("=== BLS G1 MSM ===");
    println!("Input pairs validated: {}", pairs_data.len());
    print_g1_point("Result", &result);
}

pub(super) fn cmd_g2_msm(data: String) {
    let pairs_data = parse_g2_msm_pairs(&data)
        .unwrap_or_else(|e| exit_with_error(&format!("invalid g2 msm data: {}", e)));
    let result = compute_g2_msm(&pairs_data).unwrap_or_else(|e| exit_with_error(&e));

    println!("=== BLS G2 MSM ===");
    println!("Input pairs validated: {}", pairs_data.len());
    print_g2_point("Result", &result);
}

pub(super) fn cmd_g1_msm_validate(data: String) {
    let pairs_data = parse_g1_msm_pairs(&data)
        .unwrap_or_else(|e| exit_with_error(&format!("invalid g1 msm data: {}", e)));
    println!("VALID G1 MSM input with {} point-scalar pairs", pairs_data.len());
}

pub(super) fn cmd_g2_msm_validate(data: String) {
    let pairs_data = parse_g2_msm_pairs(&data)
        .unwrap_or_else(|e| exit_with_error(&format!("invalid g2 msm data: {}", e)));
    println!("VALID G2 MSM input with {} point-scalar pairs", pairs_data.len());
}
