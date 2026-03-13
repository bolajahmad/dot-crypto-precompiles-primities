
use ark_bls12_381::{Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ff::{PrimeField, Zero};
use ark_ec::{AffineRepr, CurveGroup, Group};
use ark_std::UniformRand;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use sha2::{Sha256, Digest};

pub fn seeded_rng(seed: [u8; 32]) -> ChaCha20Rng {
    ChaCha20Rng::from_seed(seed)
}

pub struct Signature {
    pub messages: Vec<Vec<u8>>,
    pub signatures: Vec<G1Affine>,
    pub pubkeys: Vec<G2Affine>,
}

pub fn hash_to_g1(msg: &[u8]) -> G1Projective {
    let hash = Sha256::digest(msg);
    let scalar = Fr::from_be_bytes_mod_order(&hash);

    G1Projective::generator() * scalar
}

pub fn generate_signatures(k: usize, seed: [u8; 32]) -> Signature {
    let mut rng = seeded_rng(seed);

    let mut messages = Vec::new();
    let mut signatures = Vec::new();
    let mut pubkeys = Vec::new();

    for i in 0..k {
        let sk = Fr::rand(&mut rng);
        let pk = (G2Projective::generator() * sk).into_affine();
        let msg = format!("Message {}", i).into_bytes();
        let msg_hash = hash_to_g1(&msg);
        let sigma = (msg_hash * sk).into_affine();

        messages.push(msg);
        signatures.push(sigma);
        pubkeys.push(pk);
    }

    Signature {
        messages,
        signatures,
        pubkeys,
    }
}

pub fn generate_individual_pairing_inputs(data: &Signature) -> Vec<(G1Affine, G2Affine)> {
    let mut pairs = Vec::new();
    let g2 = G2Affine::generator();

    for i in 0..data.signatures.len() {
        let msg = &data.messages[i];
        let msg_hash = hash_to_g1(msg).into_affine();
        let neg_msg_hash = (-msg_hash).into();

        pairs.push((data.signatures[i], g2));
        pairs.push((neg_msg_hash, data.pubkeys[i]));
    }

    pairs
}

pub fn generate_aggregated_pairing_inputs(data: &Signature) -> Vec<(G1Affine, G2Affine)> {
    let mut pairs = Vec::new();
    let g2 = G2Affine::generator();

    let mut agg_signature = G1Projective::zero();

    for i in 0..data.signatures.len() {
        agg_signature += data.signatures[i];
    }

    pairs.push((agg_signature.into_affine(), g2));

    for i in 0..data.signatures.len() {
        let msg_hash = hash_to_g1(&data.messages[i]).into_affine();
        let neg_msg_hash = (-msg_hash).into();

        pairs.push((neg_msg_hash, data.pubkeys[i]));
    }

    pairs
}
