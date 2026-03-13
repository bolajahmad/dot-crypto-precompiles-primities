use ark_bls12_381::{Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_ff::BigInteger;
use ark_ff::{PrimeField, Zero};


pub fn encode_fq(f: Fq) -> [u8; 64] {
    let mut out = [0u8; 64];

    let be = f.into_bigint().to_bytes_be();
    let start = 64 - be.len();

    out[start..].copy_from_slice(&be);
    out
}

pub fn encode_g1(p: G1Affine) -> [u8; 128] {
    let mut out = [0u8; 128];

    out[..64].copy_from_slice(&encode_fq(p.x));
    out[64..].copy_from_slice(&encode_fq(p.y));

    out
}

pub fn encode_g2(p: &G2Affine) -> [u8; 256] {
    if p.x.is_zero() && p.y.is_zero() {
        return [0u8; 256];
    }

    let mut out = [0u8; 256];

    let x = encode_fp2(&p.x);
    let y = encode_fp2(&p.y);

    out[0..128].copy_from_slice(&x);
    out[128..256].copy_from_slice(&y);

    out
}

pub fn encode_scalar(f: Fr) -> [u8; 32] {
    let mut out = [0u8; 32];

    let be = f.into_bigint().to_bytes_be();
    let start = 32 - be.len();

    out[start..32].copy_from_slice(&be);

    out
}

pub fn encode_fp(f: &Fq) -> [u8; 64] {
    let mut out = [0u8; 64];

    let mut bytes = f.into_bigint().to_bytes_be();
    if bytes.len() < 48 {
        let mut padded = vec![0u8; 48 - bytes.len()];
        padded.extend(bytes);
        bytes = padded;
    }

    out[16..].copy_from_slice(&bytes);
    out
}

pub fn encode_fp2(f: &Fq2) -> [u8; 128] {
    let mut out = [0u8; 128];

    let c1 = encode_fp(&f.c1);
    let c0 = encode_fp(&f.c0);

    out[0..64].copy_from_slice(&c1);
    out[64..128].copy_from_slice(&c0);

    out
}
