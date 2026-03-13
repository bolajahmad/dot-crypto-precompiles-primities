# Schnorr signatures as a precompile for PVM

## Abstract

This specification introduces a precompile contract, on the pallet-revive runtime, that verifies Schnorr signatures over the secp256k1 curve (used predominantly by Ethereum and Bitcoin). This specification takes huge inspirations from the BIP-340 proposal. 

> [!NOTE]
> While this spec is based on BIP-340, it's not exactly the same and it's crucial to understand the differences.

## Motivation

Solidity smart contracts currently support verifying ECDSA signatures via the `ecrecover` and these sorts of signatures are not very scalable as verifications cannot be batched, neither can signatures be aggregated. This is why the Schnorr signatures are gaining popularity as they are way more efficient ways to handle Signature verification on-chain due to their speed, security and interactivity. Multiple signatures, from a multisig for example, can be aggregated into one signature and verifying this, proves that all the signatures are valid.There's lso a layer of security as even though the signature is an aggregation of multiple signatures, it appears as only one signature.

## Specification

The precompile operates over `secp256k1` and the following parameters are defined

```
Field prime, p = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
Curve Equation: $y^2 = x^3 + 7 mod p$
$G_x$ = 79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798,
$G_y$ = 483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8
Group Order, n = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
Precompile Address: 0x905
```

This specification is foxued only on verification of signatures, but it also defines how the signature is expected to be generated. It is critical to generate the signature appropriately, or verification will ALWAYS fail. The important algorithm is the calcilation of the challenge hash, which is a core feature of the Schnorr signature.

### Single Signature Verification

This works for a single wallet signature, or an aggregated signature. The precompile MUST accept inputs of exactly 128 bytes, encoded as follows:

Offset	FIeld
0	pubkey_x
32	r_x
64	s
96	message

#### Interpretations

**Public Key**: `pubkey_x` is interpreted as x-coordinate of a curve point, P. The corresponding y-coordinate is derived such that,
    - P lies on the curve
    - y is ALWAYS even, if y is Odd then verification fails.

**Nonce Point**: `r_x` is interpreted as the x-coordinate of the Nonce Point, R. The corresponding y-coordinate is derived such that:
    - R lies on the curve
    - y is ALWAYS even, if y is Odd then verification fails.

**Signature**: The siagnature is 64 byte which is the 32-byte `r_x` and a scalar, s, defined as:
` s = r + ed`

```
r is the secret key that generated the Nonce Point, R
e is a scalar referred to as the challenge
d is the secret key of the signer
```

#### Verification Algorithm

Given a 128-byte input:

**Steps**

1. Decode the input into the `pubkey_x`, `r_x`, `s`, `msg`
2. Fails if s ≥ n
3. Compute the challenge hash, e
   `e = int(tagged\_hash(PIP/challenge, r\_x || pubkey\_x || m)) mod n`
4. Compute 
   `s.G ?= R + e.P`
5. If equality holds, return 1, else return 0. The return values are parsed to 32 bytes, little-endian.


#### Signature Algorithm

Even though the precompile focuses on verification only, it is still very critical to ensure the signing algorithm is well defined, to ensure that the challenge hash especially can be gotten deterministically.
Given a secp256k1 SecretKey, d and a message hash:

**Steps**

1. Generate the public key. If the public key has Odd y-coordinate, then the SecretKey must be negated.
2. Generate the nonce. The steps to generate the nonce are as follows:
   1. Choose a random integer, `aux`. 
   2. Hash `aux` with tag string `"PIP/aux"` to get a 32 `aux_hash`
   3. Do a XOR of the `aux_hash` with the SecretKey to get 32-bytes t
   4. Hash `t` with tag string `"PIP/nonce"`. This is the nonce secret key.
   5. Ensure this secret key will generate a Public key with an Odd y-coordinate. If yes, return the nonce, else return the negated secret key.
3. Generate the challenge hash. This same approach is used to generate the challenge hash during verification also. Hash the concatenation of the `tag_hash`, `r-x`, `pubkey_x`, `msg`. 
4. Calculate s, as described above. The signature is (R, s)


## Tagged Hash ImplementationTh

This is something that is taken from the BIP-340 logic and avoids confusion when hashing the same data for different purposes. For instance,
- Hashing the `aux` uses `"PIP/aux"`
- Hashing the `nonce` uses `"PIP/nonce"`


## Differences from BIP-340

1. This hashes the tagged hash only once, where BIP-340 hashes twice
2. All hashing is done using Keccak256, instead of SHA256
