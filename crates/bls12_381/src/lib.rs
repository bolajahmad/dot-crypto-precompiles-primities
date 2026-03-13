mod utils;
mod codecs;

use ark_bls12_381::{Fq, Fq2, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{AffineRepr, CurveGroup};
use ark_std::UniformRand;
use ark_ff::{Fp, Zero, PrimeField};
use ark_ec::hashing::{curve_maps::wb::WBMap, map_to_curve_hasher::MapToCurve};
use utils::seeded_rng;
use codecs::{encode_g1, encode_g2, encode_scalar};
use ark_bls12_381::g1::Config as G1Config;
use ark_bls12_381::g2::Config as G2Config;

use crate::{codecs::{encode_fp, encode_fp2}, utils::{generate_aggregated_pairing_inputs, generate_individual_pairing_inputs, generate_signatures}};

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
    let mut rng = seeded_rng([56_u8; 32]);

    let mut points = Vec::with_capacity(k);
    let mut scalars = Vec::with_capacity(k);
    let mut inputs = Vec::with_capacity(k * 160_usize);

    for _ in 0..k {
        let p: G1Affine = G1Projective::rand(&mut rng).into_affine();
        let s = Fp::rand(&mut rng);

        inputs.extend_from_slice(&encode_g1(p));
        inputs.extend_from_slice(&encode_scalar(s));

        points.push(p);
        scalars.push(s);
    }
    println!("MSM input point for index: 0x{}", hex::encode(&inputs));

    let mut acc = G1Projective::zero();
    for (p, s) in points.iter().zip(scalars.iter()) {
        acc += p.mul_bigint(s.into_bigint());
    }

    let result = acc.into_affine();
    result
}

pub fn generate_g2_msm_params(k: usize) -> G2Affine {
    let mut rng = seeded_rng([65_u8; 32]);

    let mut points = Vec::with_capacity(k);
    let mut scalars = Vec::with_capacity(k);
    let mut inputs = Vec::with_capacity(k * 288);

    for _ in 0..k {
        let p = G2Projective::rand(&mut rng).into_affine();
        let s = Fp::rand(&mut rng);
        points.push(p);
        scalars.push(s);

        inputs.extend_from_slice(&encode_g2(&p));
        inputs.extend_from_slice(&encode_scalar(s));
    }

    let mut acc = G2Projective::zero();
    for (p, s) in points.iter().zip(scalars.iter()) {
        acc += p.mul_bigint(s.into_bigint());
    }

    let result = acc.into_affine();
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
    for (i, (p, q)) in pairs.iter().enumerate() {
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