# BLS12-381 Precompiles (EIP-2537)

## Abstract

This document is the authoritative specification for the seven BLS12-381 precompiles implemented in the PVM runtime (`pallet-revive`). The implementation targets conformance with [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537) — the Ethereum standard for BLS12-381 curve operations — so that contracts written against Ethereum's BLS precompiles will work without modification on PVM.

The precompiles cover:

- G1 and G2 point addition
- G1 and G2 multi-scalar multiplication (MSM)
- Pairing-based product check
- Field-element-to-curve mapping (Fp → G1, Fp2 → G2)

## Motivation

### Why BLS12-381?

BLS12-381 is a pairing-friendly elliptic curve and is the cryptographic foundation for:

- Ethereum's consensus layer (Proof-of-Stake validator signatures)
- BLS signature aggregation (many signers, one signature, one verification)
- zkSNARK proof systems (Groth16, PLONK, and many others)
- KZG polynomial commitments (the basis for EIP-4844 blob data and Danksharding)

Making these operations available as native precompiles is far cheaper than reimplementing the curve arithmetic in Solidity/EVM bytecode, where a single pairing can cost millions of gas.

### Why does this matter for rollup developers?

A rollup building on top of a PVM-compatible chain inherits these precompiles directly. In practice that means:

- **ZK proof verification on-chain** — Groth16 and PLONK verifiers that run under Ethereum use exactly the same pairing-check precompile encoding. A verifier contract ported from Ethereum requires no encoding changes.
- **BLS aggregate signature verification** — EigenLayer-style AVS contracts, threshold signatures, and validator set attestations can be verified using the same pipeline as Ethereum mainnet.
- **KZG commitment verification** — Rollups using blob proofs can reuse their Ethereum verifier contracts as-is.
- **Cross-chain compatibility** — Because the wire encoding is identical to EIP-2537, test vectors from the Ethereum test suite can be run directly against these precompiles.

## Curve Parameters

```text
Curve family:   Barreto–Lynn–Scott, embedding degree 12
                (BLS12, order approximately 2^381)

Base field (Fq):
  p = 0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab

Scalar field (Fr):
  r = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001

G1 generator:
  Gx = 0x17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb
  Gy = 0x08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1

G2 generator (Fq2 coordinates, imaginary part first in encoding):
  Gx.c1 = 0x024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8
  Gx.c0 = 0x13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e
  Gy.c1 = 0x0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801
  Gy.c0 = 0x0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be

G1 cofactor: 1   (every point on the G1 curve is in the prime-order subgroup)
G2 cofactor: h2  (non-trivial; subgroup membership must be checked separately)
```

## Precompile Registry

| Precompile         | Address | EIP-2537 Address | Input (bytes) | Output (bytes) |
|--------------------|---------|------------------|---------------|----------------|
| `BLS12G1Add`       | `0x0b`  | `0x0b`           | 256           | 128            |
| `BLS12G1MSM`       | `0x0c`  | `0x0c`           | 160 × k       | 128            |
| `BLS12G2Add`       | `0x0d`  | `0x0d`           | 512           | 256            |
| `BLS12G2MSM`       | `0x0e`  | `0x0e`           | 288 × k       | 256            |
| `BLS12PairingCheck`| `0x0f`  | `0x0f`           | 384 × k       | 32             |
| `BLS12MapFpToG1`   | `0x10`  | `0x10`           | 64            | 128            |
| `BLS12MapFp2ToG2`  | `0x11`  | `0x11`           | 128           | 256            |

All seven addresses are identical to EIP-2537.

## Encoding Formats

### Field element Fp (64 bytes)

EIP-2537 encodes a base-field element in 64 bytes: 16 zero-padding bytes followed by 48 bytes of big-endian representation.

```
[00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00] [48 bytes, big-endian, value < p]
 ^^^^^^^^^^^^^^ 16 zero bytes ^^^^^^^^^^^^^^^^^^   ^^^^^^^^ actual element ^^^^^^^^
```

The padding bytes MUST be zero; non-zero padding causes the call to revert.

### Extension field element Fp2 (128 bytes)

An Fp2 element `a + b·u` (where `u² = -1`) is encoded as:

```
[64 bytes: imaginary part b (c1)] [64 bytes: real part a (c0)]
```

> The imaginary part comes **first**. This ordering follows EIP-2537 and is opposite to the arkworks internal field representation where `Fq2 { c0, c1 }` stores real first.

### Scalar (32 bytes)

Scalars for MSM operations are 32-byte big-endian unsigned integers. They are reduced modulo the group order `r` before use, so values ≥ r are silently normalised rather than rejected.

### G1 point (128 bytes)

```
[64 bytes: x-coordinate (Fp)] [64 bytes: y-coordinate (Fp)]
```

The point at infinity is represented by 128 zero bytes.

### G2 point (256 bytes)

```
[128 bytes: x-coordinate (Fp2)] [128 bytes: y-coordinate (Fp2)]
```

Within each Fp2 coordinate, the imaginary part comes first (as specified above). The point at infinity is 256 zero bytes.

### G1 MSM pair (160 bytes)

```
[128 bytes: G1 point] [32 bytes: scalar]
```

### G2 MSM pair (288 bytes)

```
[256 bytes: G2 point] [32 bytes: scalar]
```

### Pairing pair (384 bytes)

```
[128 bytes: G1 point] [256 bytes: G2 point]
```

## Precompile Specifications

### BLS12G1Add — `0x0b`

Adds two G1 points using projective coordinates and returns the result in affine form.

**Input:** Exactly 256 bytes = two 128-byte G1 points.  
**Output:** 128 bytes — the affine sum.  
**Revert conditions:** Input length ≠ 256, or either point fails the on-curve check.

**Algorithm:**
1. Decode P1 from bytes 0–127.
2. Decode P2 from bytes 128–255.
3. Validate both points are on the G1 curve (or the identity).
4. Compute P1 + P2 in G1Projective and convert to affine.
5. Encode and return.

Point at infinity in: if both coordinates of a point are zero, it is treated as the identity element and participates in addition normally.

### BLS12G1MSM — `0x0c`

Multi-scalar multiplication over G1: computes `Σ sᵢ·Pᵢ` for k pairs using the arkworks `VariableBaseMSM` algorithm.

**Input:** `160 × k` bytes, k ≥ 1.  
**Output:** 128 bytes — the affine MSM result.  
**Revert conditions:** Empty input, input length not a multiple of 160, or any point fails validation.

**Algorithm:**
1. Reject if `input.len() == 0` or `input.len() % 160 != 0`.
2. For each 160-byte chunk: decode the 128-byte G1 point and the 32-byte scalar (reduced mod r).
3. Execute `G1Projective::msm(&points, &scalars)`.
4. Convert result to affine, encode, and return.

**Note on scalar reduction:** A scalar `s ≥ r` is reduced modulo `r` silently. This matches EIP-2537 behaviour.

### BLS12G2Add — `0x0d`

Adds two G2 points. Structurally identical to G1Add but operating over the Fp2-based G2 curve.

**Input:** Exactly 512 bytes = two 256-byte G2 points.  
**Output:** 256 bytes — the affine sum.  
**Revert conditions:** Input length ≠ 512, or either point fails the on-curve check.

### BLS12G2MSM — `0x0e`

Multi-scalar multiplication over G2: computes `Σ sᵢ·Qᵢ` for k pairs.

**Input:** `288 × k` bytes, k ≥ 1.  
**Output:** 256 bytes — the affine MSM result.  
**Revert conditions:** Empty input, input length not a multiple of 288, or any point fails validation.

### BLS12PairingCheck — `0x0f`

Checks whether the product of k Miller-loop pairings equals the identity in GT (the target field). This is the core primitive for signature aggregation, SNARK proof verification, and KZG commitment verification.

**Input:** `384 × k` bytes, k ≥ 1 — pairs of (G1 point, G2 point).  
**Output:** 32 bytes — ABI-encoded `bool`:
- `0x0000...0001` if `∏ e(G1ᵢ, G2ᵢ) = 1_GT`
- `0x0000...0000` otherwise

**Revert conditions:** Empty input, input length not a multiple of 384, or any point fails the on-curve check.

**Algorithm:**
1. Reject if `input.len() == 0` or `input.len() % 384 != 0`.
2. For each 384-byte chunk: decode a G1 point and a G2 point.
3. Execute `Bls12_381::multi_pairing(&g1_points, &g2_points)`.
4. Return `true` if result is the multiplicative identity in GT (`result.0.is_one()`), `false` otherwise.

**Canonical pairing identity check for BLS signatures:**

To verify `e(σ, G2) == e(H(msg), PK)` using the precompile, negate one side and batch:

```
pairing_check( [σ, -H(msg)], [G2_generator, PK] ) == true
```

This avoids computing two separate pairings.

### BLS12MapFpToG1 — `0x10`

Maps an Fp field element to a G1 point using the Wahby–Boneh (WB) map-to-curve method, then clears the cofactor.

**Input:** Exactly 64 bytes — one Fp element.  
**Output:** 128 bytes — a G1 affine point.  
**Revert conditions:** Input length ≠ 64, or the Fp value is malformed (non-zero padding).

**Algorithm:**
1. Decode the 64-byte Fp element.
2. Apply `WBMap::<G1Config>::map_to_curve(fp)`.
3. Call `.clear_cofactor()` (G1 cofactor = 1, so this is a no-op but included for correctness).
4. Encode and return.

The output is always a valid G1 point in the prime-order subgroup. This is suitable for hash-to-curve constructions.

### BLS12MapFp2ToG2 — `0x11`

Maps an Fp2 field element to a G2 point using the WB map-to-curve method, then clears the G2 cofactor.

**Input:** Exactly 128 bytes — one Fp2 element (imaginary part first, per encoding rules).  
**Output:** 256 bytes — a G2 affine point.  
**Revert conditions:** Input length ≠ 128, or either Fp component is malformed.

**Algorithm:**
1. Decode the 128-byte Fp2 element.
2. Apply `WBMap::<G2Config>::map_to_curve(fp2)`.
3. Call `.clear_cofactor()` — this IS a meaningful reduction for G2.
4. Encode and return.

## Divergences from EIP-2537

The implementation is faithful to EIP-2537, with one intentional difference.

### Difference — Gas replaced by PVM weight tokens

EIP-2537 defines specific Ethereum gas costs per operation. In the PVM runtime, these are replaced by `RuntimeCosts::Bls12381*` weight tokens metered through `env.frame_meter_mut().charge_weight_token(...)`. The economic model differs but the functional behaviour is the same: the runtime will return an out-of-gas/out-of-weight error if the caller cannot cover the operation cost.

### Conformance table

| Aspect                            | EIP-2537                          | This implementation                          |
|-----------------------------------|-----------------------------------|----------------------------------------------|
| G1 point encoding                 | 128 bytes (16 pad + 48 big-endian per coord) | Identical                           |
| G2 point encoding                 | 256 bytes (Fp2 imaginary-first)   | Identical                                    |
| Fp2 component order               | Imaginary (c1) first              | Identical                                    |
| Scalar encoding                   | 32 bytes big-endian, mod r        | Identical                                    |
| G1Add input size                  | Exactly 256 bytes                 | Identical                                    |
| G1MSM input size                  | Multiple of 160 bytes, non-zero   | Identical                                    |
| G2Add input size                  | Exactly 512 bytes                 | Identical                                    |
| G2MSM input size                  | Multiple of 288 bytes, non-zero   | Identical                                    |
| Pairing input size                | Multiple of 384 bytes, non-zero   | Identical                                    |
| MapFpToG1 input size              | Exactly 64 bytes                  | Identical                                    |
| MapFp2ToG2 input size             | Exactly 128 bytes                 | Identical                                    |
| PairingCheck return               | 32-byte ABI bool                  | Identical                                    |
| G1 on-curve validation            | Required                          | Implemented                                  |
| G2 on-curve validation            | Required                          | Implemented                                  |
| G2 subgroup membership validation | Required                          | Implemented                                  |
| Map-to-curve algorithm            | WB (Wahby–Boneh)                  | Identical (arkworks WBMap)                   |
| Precompile addresses              | 0x0b – 0x11                       | Identical                                    |
| Gas model                         | Ethereum gas schedule             | PVM weight tokens                            |

## BLS Signature Scheme

The precompiles are building blocks. The conventional BLS signature scheme (as used in Ethereum PoS) is assembled from them as follows. This scheme is implemented in `crates/bls12_381/src/lib.rs` and used by the CLI tooling.

### Keys

```
secret key: sk  ∈ Fr  (random scalar)
public key: PK  = sk · G2  ∈ G2     (G2 scalar multiplication)
```

### Hash to G1

Messages are mapped to G1 points via a simplified hash-to-curve:

```
H(msg) = SHA256(msg) interpreted as a scalar, then multiplied by the G1 generator
```

> This is a simplified scheme for tooling and testing. Production BLS signature schemes use a proper hash-to-curve per [RFC 9380](https://www.rfc-editor.org/rfc/rfc9380), which involves `MapFpToG1` or `MapFp2ToG2`. The `BLS12MapFpToG1` precompile is available if you need a standard-compliant hash-to-curve.

### Sign

```
σ = sk · H(msg)  ∈ G1
```

### Verify (single)

Check that `e(σ, G2_generator) == e(H(msg), PK)`:

```
pairing_check( [σ, -H(msg)], [G2_generator, PK] ) == true
```

This is a single call to `BLS12PairingCheck` (0x0f) with k = 2 pairs.

### Aggregate and batch-verify

To aggregate n signatures:

```
σ_agg = σ₁ + σ₂ + ... + σₙ   (G1 point addition, n-1 calls to BLS12G1Add)
```

Batch verification: instead of n individual pairing checks (2n Miller loops), use one multi-pairing:

```
pairing_check(
  [σ_agg,     -H(msg₁), -H(msg₂), ..., -H(msgₙ)],
  [G2_gen,    PK₁,       PK₂,       ..., PKₙ     ]
) == true
```

This is one call to `BLS12PairingCheck` with k = n + 1 pairs — constant-time in the number of pairings regardless of how many signers contributed.

## Solidity Integration

### Types

```solidity
struct G1Point { bytes x; bytes y; }         // 64 bytes each
struct G2Point { bytes[2] x; bytes[2] y; }   // [imaginary, real], 64 bytes each
struct FP  { bytes value; }                  // 64 bytes (16 zero pad + 48 value)
struct FP2 { bytes[2] value; }               // [imaginary, real]
struct G1MSM { G1Point[] points; uint256[] scalars; }
struct G2MSM { G2Point[] points; uint256[] scalars; }
```

### Library interface

```solidity
import "./modules/BLS.sol";

// G1 point addition
G1Point memory sum = BLS.g1AddPoint(pointA, pointB);

// G2 point addition
G2Point memory sum = BLS.g2AddPoint(pointA, pointB);

// Pairing check (returns bool)
bool valid = BLS.pairingCheck(encodedPairs);

// Map field element to G1 point
G1Point memory p = BLS.mapFpToG1Point(fp);
```

### Raw staticcall (zero-copy)

All precompiles accept raw bytes via `staticcall`. This avoids Solidity ABI overhead for performance-critical paths:

```solidity
(bool ok, bytes memory out) = address(0x0b).staticcall(abi.encodePacked(p1x, p1y, p2x, p2y));
require(ok, "G1Add failed");
```

### Addresses

```solidity
address constant BLS_G1_ADD       = address(0x0b);
address constant BLS_G1_MSM       = address(0x0c);
address constant BLS_G2_ADD       = address(0x0d);
address constant BLS_G2_MSM       = address(0x0e);
address constant BLS_FIELD_PAIRING = address(0x0f);
address constant BLS_FP_TO_G1     = address(0x10);
address constant BLS_FP2_TO_G2    = address(0x11);
```

## Tooling in This Repository

The `pvm-cli` provides commands covering all seven precompile operations, using two input formats:

- **Hex:** raw concatenated hex encoding matching the precompile wire format.
- **JSON:** Solidity-struct-shaped JSON for readability.

### Point generation and addition

```bash
# Random G1 / G2 points
pvm-cli bls random-g1
pvm-cli bls random-g2

# G1 point addition (hex or JSON input)
pvm-cli bls g1-add --point-a <hex|json> --point-b <hex|json>
pvm-cli bls g2-add --point-a <hex|json> --point-b <hex|json>
```

### MSM

```bash
# Generate deterministic test data (k pairs)
pvm-cli bls g1-msm-testdata -k 4 --output both
pvm-cli bls g2-msm-testdata -k 4 --output both

# Execute MSM from hex or JSON input
pvm-cli bls g1-msm --data <hex|json>
pvm-cli bls g2-msm --data <hex|json>
```

### Map-to-curve

```bash
# Map a random Fp element to a G1 point
pvm-cli bls map-fp-to-g1

# Map a random Fp2 element to a G2 point
pvm-cli bls map-fp2-to-g2
```

### Signatures

```bash
# Generate a BLS keypair and sign a message
pvm-cli bls sign --message <hex>

# Verify a BLS signature
pvm-cli bls verify --signature <hex> --message <hex> --pubkey <hex>

# Generate batch testdata (k signers)
pvm-cli bls batch-testdata -k 4

# Aggregate k signatures into one
pvm-cli bls aggregate --signatures <hex>,<hex>,...

# Batch verify with aggregation and pairing
pvm-cli bls batch-smoke -k 4
```

## Security Considerations

- **G2 subgroup safety** — the runtime rejects G2 points that are on the curve but outside the prime-order subgroup. If you need to produce a valid G2 subgroup point from an arbitrary byte input, use `BLS12MapFp2ToG2` (0x11), which applies WB map-to-curve and cofactor clearing deterministically.
- **Scalar reduction** — scalars ≥ r are reduced silently. This is standard behaviour but callers should be aware that extremely large scalar values produce the same output as their reduction mod r.
- **BLS signature uniqueness** — this implementation uses a simplified SHA256-based hash-to-G1. For interoperability with external BLS libraries using RFC 9380 hash-to-curve, use the `MapFpToG1` precompile with a proper hash-and-map construction.
- **Rogue key attacks** — BLS signature aggregation is vulnerable to rogue public-key attacks when public keys are not accompanied by proof-of-possession. Do not blindly aggregate public keys from untrusted parties without a proof-of-possession or distinct message domains.
- **Pairing malleability** — any pair `(σ, -H(msg), G2, PK)` that satisfies the pairing identity is accepted. Ensure application-level message binding prevents replay or substitution attacks.
