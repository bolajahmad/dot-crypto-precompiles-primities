use schnorr::{
    encode_precompile_input,
    generate_signature,
};
use schnorr::utils::message;

use secp256k1::SecretKey;

fn main() {

    let secret_key =
        SecretKey::from_slice(&[1u8; 32]).unwrap();

    let aux = [2u8; 32];

    let msg = message(Some("Hello, world!"));

    let (pubkey_x, rx, s) =
        generate_signature(secret_key, &msg, aux);

    let input =
        encode_precompile_input(&pubkey_x, &rx, &s, &msg);

    println!("Input Length: {}", input.len());
    println!("Input: 0x{}", hex::encode(input));
}