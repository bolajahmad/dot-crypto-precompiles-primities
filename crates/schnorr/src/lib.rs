use secp256k1::{Parity, PublicKey, Scalar, Secp256k1, SecretKey, XOnlyPublicKey};

use crate::utils::{generate_challenge_hash, generate_nonce_key};

pub mod utils;

// Re-export for CLI usage
pub use secp256k1;

/// Ensure secret key produces even Y public key
pub fn normalize_secret_key(secret_key: SecretKey) -> SecretKey {
    let secp = Secp256k1::new();

    let mut sk = secret_key;
    let (_, parity) = PublicKey::from_secret_key(&secp, &sk).x_only_public_key();

    if parity == Parity::Odd {
        sk = sk.negate();
    }

    sk
}

/// Generate nonce key (r)
pub fn generate_nonce(
    aux: &[u8; 32],
    secret_key: &SecretKey,
    msg: &[u8; 32],
) -> SecretKey {
    generate_nonce_key(aux, secret_key, msg)
}

/// Generate challenge hash e = H(R || P || m)
pub fn generate_challenge(
    rx: &[u8; 32],
    pubkey_x: &[u8; 32],
    msg: &[u8; 32],
) -> secp256k1::Scalar {
    generate_challenge_hash(rx, pubkey_x, msg)
}

/// Generate Schnorr signature
pub fn generate_signature(
    secret_key: SecretKey,
    msg: &[u8; 32],
    aux: [u8; 32],
) -> ([u8; 32], [u8; 32], [u8; 32]) {

    let secp = Secp256k1::new();

    let sk = normalize_secret_key(secret_key);

    let nonce_secret = generate_nonce(&aux, &sk, msg);

    let nonce_pubkey = PublicKey::from_secret_key(&secp, &nonce_secret);

    let (signer_xonly, _) =
        PublicKey::from_secret_key(&secp, &sk).x_only_public_key();

    let (nonce_xonly, _) = nonce_pubkey.x_only_public_key();

    let pubkey_x = signer_xonly.serialize();
    let rx = nonce_xonly.serialize();

    let challenge = generate_challenge(&rx, &pubkey_x, msg);

    let ed = sk.mul_tweak(&challenge).expect("valid tweak");

    let ed_scalar =
        Scalar::from_be_bytes(ed.secret_bytes()).unwrap();

    let s = nonce_secret
        .add_tweak(&ed_scalar)
        .expect("valid tweak")
        .secret_bytes();

    (pubkey_x, rx, s)
}

/// Encode precompile input
pub fn encode_precompile_input(
    pubkey_x: &[u8; 32],
    rx: &[u8; 32],
    s: &[u8; 32],
    msg: &[u8],
) -> Vec<u8> {

    let mut input = Vec::with_capacity(128);

    input.extend_from_slice(pubkey_x);
    input.extend_from_slice(rx);
    input.extend_from_slice(s);
    input.extend_from_slice(msg);

    input
}

/// Verify a Schnorr signature
/// Returns true if the signature is valid, false otherwise.
/// 
/// Verification algorithm:
/// 1. Parse pubkey_x as x-coordinate of P (derive even y)
/// 2. Parse rx as x-coordinate of R (derive even y)
/// 3. Compute challenge e = H("PIP/challenge" || rx || pubkey_x || msg) mod n
/// 4. Verify: s·G == R + e·P
pub fn verify_signature(
    pubkey_x: &[u8; 32],
    rx: &[u8; 32],
    s: &[u8; 32],
    msg: &[u8; 32],
) -> bool {
    let secp = Secp256k1::new();

    // Parse public key (x-only, assume even y)
    let pubkey = match XOnlyPublicKey::from_slice(pubkey_x) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    // Parse nonce point R (x-only, assume even y)
    let nonce_point = match XOnlyPublicKey::from_slice(rx) {
        Ok(np) => np,
        Err(_) => return false,
    };

    // Parse s as a scalar - must be less than curve order
    let s_scalar = match Scalar::from_be_bytes(*s) {
        Ok(scalar) => scalar,
        Err(_) => return false,
    };

    // Compute challenge e = H(rx || pubkey_x || msg) mod n
    let challenge = generate_challenge(rx, pubkey_x, msg);

    // Convert x-only pubkeys to full pubkeys with even y parity
    let pubkey_full = match PublicKey::from_x_only_public_key(pubkey, Parity::Even) {
        pk => pk,
    };
    let nonce_full = match PublicKey::from_x_only_public_key(nonce_point, Parity::Even) {
        pk => pk,
    };

    // Compute e·P
    let ep = match pubkey_full.mul_tweak(&secp, &challenge) {
        Ok(point) => point,
        Err(_) => return false,
    };

    // Compute R + e·P
    let r_plus_ep = match nonce_full.combine(&ep) {
        Ok(point) => point,
        Err(_) => return false,
    };

    // Compute s·G (using generator point)
    // We create a secret key from s and get its public key
    let s_bytes = s_scalar.to_be_bytes();
    let s_as_sk = match SecretKey::from_slice(&s_bytes) {
        Ok(sk) => sk,
        Err(_) => return false,
    };
    let s_g = PublicKey::from_secret_key(&secp, &s_as_sk);

    // Verify s·G == R + e·P
    s_g == r_plus_ep
}