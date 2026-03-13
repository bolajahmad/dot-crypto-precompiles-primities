use alloy_primitives::Keccak256;
use secp256k1::{Parity, PublicKey, Scalar, Secp256k1, SecretKey};

const SECP256K1_ORDER: [u8; 32] = [
    0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
    0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFE,
    0xBA,0xAE,0xDC,0xE6,0xAF,0x48,0xA0,0x3B,
    0xBF,0xD2,0x5E,0x8C,0xD0,0x36,0x41,0x41
];

pub fn generate_nonce_key(aux: &[u8; 32], priv_key: &SecretKey, msg: &[u8; 32]) -> SecretKey {
    let secp = Secp256k1::new();

    // H(aux)
    let mut hasher = Keccak256::new();
    hasher.update("PIP/aux");
    hasher.update(aux);
    let aux_hash = hasher.finalize();

    // t = H(aux) XOR sk
    let mut t = [0u8; 32];
    let sk_bytes = priv_key.secret_bytes();

    for i in 0..32 {
        t[i] = aux_hash[i] ^ sk_bytes[i];
    }

    // include pubkey
    let pubkey = PublicKey::from_secret_key(&secp, priv_key);
    let (xonly, _) = pubkey.x_only_public_key();

    let mut hasher = Keccak256::new();
    hasher.update("PIP/nonce");
    hasher.update(&t);
    hasher.update(&xonly.serialize());
    hasher.update(msg);

    let mut nonce_hash = hasher.finalize();
    let mut nonce_sk = loop {
        let nonce_bytes = *nonce_hash;
        if let Ok(sk) = SecretKey::from_slice(&nonce_bytes) {
            break sk;
        }

        // rehash if overflow
        let mut retry = Keccak256::new();
        retry.update(&nonce_bytes);
        nonce_hash = retry.finalize();
    };

    // ensure even R
    let (_, parity) = PublicKey::from_secret_key(&secp, &nonce_sk).x_only_public_key();

    if parity == Parity::Odd {
        nonce_sk = nonce_sk.negate();
    }

    nonce_sk
}

pub fn message(text: Option<&str>) -> [u8; 32] {
    let value = text.unwrap_or("Hello, world!");
    let mut hasher = Keccak256::new();
    hasher.update(value);
    hasher.finalize().into()
}

pub fn generate_challenge_hash(rx: &[u8; 32], pubkey_x: &[u8; 32], msg: &[u8; 32]) -> Scalar {
    let mut hasher = Keccak256::new();
    hasher.update("PIP/challenge");
    hasher.update(rx);
    hasher.update(pubkey_x);
    hasher.update(msg);
    let digest = hasher.finalize();
    let data = *digest;

    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&data);
    let reduced = mod_n(bytes);

    Scalar::from_be_bytes(reduced).unwrap()
}

fn mod_n(mut x: [u8; 32]) -> [u8; 32] {

    if x >= SECP256K1_ORDER {
        let mut borrow = 0u16;

        for i in (0..32).rev() {
            let xi = x[i] as i16;
            let ni = SECP256K1_ORDER[i] as i16;

            let val = xi - ni - borrow as i16;

            if val < 0 {
                x[i] = (val + 256) as u8;
                borrow = 1;
            } else {
                x[i] = val as u8;
                borrow = 0;
            }
        }
    }

    x
}
