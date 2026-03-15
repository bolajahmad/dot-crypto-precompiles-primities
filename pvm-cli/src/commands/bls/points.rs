use ark_bls12_381::{G1Affine, G2Affine};
use bls12_381::{add_g1_points, add_g2_points, generate_random_g1_point, generate_random_g2_point,
    generate_g1_add_testdata, generate_g2_add_testdata, codecs::{encode_g1, encode_g2}};

use super::shared::{parse_g1_point_or_exit, parse_g2_point_or_exit, TestVector, write_or_print_vectors};

pub(super) fn cmd_random_g1() {
    print_g1_point("Random BLS G1 Point", &generate_random_g1_point());
}

pub(super) fn cmd_random_g2() {
    print_g2_point("Random BLS G2 Point", &generate_random_g2_point());
}

pub(super) fn cmd_g1_add(point_a: String, point_b: String) {
    let point_a = parse_g1_point_or_exit(&point_a, "point-a");
    let point_b = parse_g1_point_or_exit(&point_b, "point-b");
    let result = add_g1_points(point_a, point_b);

    println!("=== BLS G1 Addition ===");
    print_g1_point("Point A", &point_a);
    print_g1_point("Point B", &point_b);
    print_g1_point("Result", &result);
}

pub(super) fn cmd_g2_add(point_a: String, point_b: String) {
    let point_a = parse_g2_point_or_exit(&point_a, "point-a");
    let point_b = parse_g2_point_or_exit(&point_b, "point-b");
    let result = add_g2_points(point_a, point_b);

    println!("=== BLS G2 Addition ===");
    print_g2_point("Point A", &point_a);
    print_g2_point("Point B", &point_b);
    print_g2_point("Result", &result);
}

pub(super) fn cmd_g1_add_testdata(count: usize, output_file: Option<String>) {
    let vectors: Vec<TestVector> = generate_g1_add_testdata(count)
        .into_iter()
        .enumerate()
        .map(|(i, (input, expected))| TestVector {
            input: format!("0x{}", hex::encode(input)),
            expected: hex::encode(expected),
            name: format!("bls12381_g1_add {}", i + 1),
        })
        .collect();
    write_or_print_vectors(&vectors, output_file.as_deref());
}

pub(super) fn cmd_g2_add_testdata(count: usize, output_file: Option<String>) {
    let vectors: Vec<TestVector> = generate_g2_add_testdata(count)
        .into_iter()
        .enumerate()
        .map(|(i, (input, expected))| TestVector {
            input: format!("0x{}", hex::encode(input)),
            expected: hex::encode(expected),
            name: format!("bls12381_g2_add {}", i + 1),
        })
        .collect();
    write_or_print_vectors(&vectors, output_file.as_deref());
}

pub(crate) fn print_g1_point(label: &str, point: &G1Affine) {
    let encoded = encode_g1(*point);
    let x = &encoded[..64];
    let y = &encoded[64..];

    println!("{}:", label);
    println!("  Hex: 0x{}", hex::encode(encoded));
    println!("  JSON: {{\"x\":\"0x{}\",\"y\":\"0x{}\"}}", hex::encode(x), hex::encode(y));
    println!(
        "  Solidity: G1Point({{ x: hex\"{}\", y: hex\"{}\" }})",
        hex::encode(x),
        hex::encode(y)
    );
    println!();
}

pub(crate) fn print_g2_point(label: &str, point: &G2Affine) {
    let encoded = encode_g2(point);
    let x0 = &encoded[..64];
    let x1 = &encoded[64..128];
    let y0 = &encoded[128..192];
    let y1 = &encoded[192..];

    println!("{}:", label);
    println!("  Hex: 0x{}", hex::encode(encoded));
    println!(
        "  JSON: {{\"x\":[\"0x{}\",\"0x{}\"],\"y\":[\"0x{}\",\"0x{}\"]}}",
        hex::encode(x0),
        hex::encode(x1),
        hex::encode(y0),
        hex::encode(y1)
    );
    println!(
        "  Solidity: G2Point({{ x: [hex\"{}\", hex\"{}\"], y: [hex\"{}\", hex\"{}\"] }})",
        hex::encode(x0),
        hex::encode(x1),
        hex::encode(y0),
        hex::encode(y1)
    );
    println!();
}

#[cfg(test)]
mod tests {
    use bls12_381::{generate_random_g1_point, generate_random_g2_point, codecs::{encode_g1, encode_g2}};

    #[test]
    fn test_parse_g1_hex_input() {
        let point = generate_random_g1_point();
        let encoded = encode_g1(point);
        let parsed = super::super::shared::parse_g1_point(&format!("0x{}", hex::encode(encoded)))
            .expect("g1 hex parse should succeed");
        assert_eq!(encode_g1(parsed), encoded);
    }

    #[test]
    fn test_parse_g2_json_input() {
        let point = generate_random_g2_point();
        let encoded = encode_g2(&point);
        let input = format!(
            r#"{{"x":["0x{}","0x{}"],"y":["0x{}","0x{}"]}}"#,
            hex::encode(&encoded[..64]),
            hex::encode(&encoded[64..128]),
            hex::encode(&encoded[128..192]),
            hex::encode(&encoded[192..])
        );

        let parsed = super::super::shared::parse_g2_point(&input).expect("g2 json parse should succeed");
        assert_eq!(encode_g2(&parsed), encoded);
    }
}
