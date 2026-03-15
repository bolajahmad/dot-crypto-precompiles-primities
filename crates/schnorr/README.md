# Schnorr Signature Verification Precompile (PIP)

## Abstract

This document is the authoritative specification for the Schnorr signature verification precompile in the PVM runtime (`pallet-revive`). It is **not** a BIP-340-compatible protocol — knowledge of BIP-340 does not transfer directly. Read this document carefully and check the noted differences before building integrations.

The scheme uses:

- x-only public keys (even-Y normalisation)
- Domain-separated, single-pass Keccak256 tagged hashing
- Deterministic nonce derivation that binds the nonce to the public key and message

## Motivation

`ecrecover` in Solidity supports only ECDSA, which lacks:

- Secure signature aggregation
- Efficient batch verification
- Key aggregation (MuSig2, FROST)

Schnorr signatures address all three and also:

- Eliminate signature malleability
- Allow smaller key representations (x-only)
- Have constant, deterministic verification cost

Exposing verification as a native precompile makes these benefits accessible to smart contracts without re-implementing expensive elliptic-curve arithmetic in Solidity.

## Curve Parameters

```text
Curve:         secp256k1
Equation:      y^2 = x^3 + 7 (mod p)
Field prime p: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
Generator Gx:  79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798
Generator Gy:  483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8
Group order n: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
Address:       0x0000000000000000000000000000000000000905
```

## Precompile Interface

```solidity
interface ISchnorr {
    function verify(bytes calldata input) external view returns (bool valid);
}
```

Input that is not exactly 128 bytes causes the call to revert with `"Invalid input len"`. All other error conditions return `false` (ABI-encoded as a 32-byte bool).

## Verification Input Format

The 128-byte input is laid out as four 32-byte fields:

| Offset | Field      | Interpretation                              |
|--------|------------|---------------------------------------------|
| 0      | `pubkey_x` | x-coordinate of signer public key `P`       |
| 32     | `r_x`      | x-coordinate of nonce point `R`             |
| 64     | `s`        | signature scalar                            |
| 96     | `message`  | 32-byte message hash                        |

Point lift rules (apply to both `pubkey_x` and `r_x`):

- Interpret the 32 bytes as a big-endian x-coordinate.
- The corresponding y is chosen to be **even** (x-only / even-Y normalisation).
- If no valid even-y point exists on the curve for that x, return `false`.

## Tagged Hash Construction

All hashing in this scheme uses a single-pass Keccak256 construction:

```
pip_hash(tag, data...) = keccak256(tag || data[0] || data[1] || ...)
```

where `tag` is a UTF-8 string literal and `data` components are concatenated in the order listed.

> **Important:** This is NOT the BIP-340 double-SHA256 tagged hash
> `SHA256(SHA256(tag) || SHA256(tag) || data)`.
> Generating data with the BIP-340 convention will not verify here.

Tags used by this scheme:

| Tag             | Purpose                                      |
|-----------------|----------------------------------------------|
| `"PIP/aux"`     | Domain separation for auxiliary randomness   |
| `"PIP/nonce"`   | Domain separation for nonce derivation       |
| `"PIP/challenge"` | Domain separation for challenge scalar     |

## Verification Algorithm

```
Input:  pubkey_x (32 bytes), r_x (32 bytes), s (32 bytes), msg (32 bytes)
Output: bool (ABI-encoded, 32 bytes)
```

1. **Length check** — if input ≠ 128 bytes, revert.
2. **Lift P** — derive curve point `P` with even y from `pubkey_x`.  
   Return `false` if lift fails.
3. **Lift R** — derive curve point `R` with even y from `r_x`.  
   Return `false` if lift fails.
4. **Parse s** — interpret `s` as a big-endian scalar.  
   Return `false` if `s >= n` (out of range).
5. **Compute challenge** — compute scalar `e`:
   ```
   e = keccak256("PIP/challenge" || r_x || pubkey_x || msg) mod n
   ```
6. **Verify equation**:
   ```
   s·G == R + e·P
   ```
7. Return `true` if equality holds, `false` otherwise.

## Signing Algorithm

The precompile only verifies. Signing is defined here so that test vectors, tooling, and Solidity integration tests produce consistent results across the codebase.

### Step 1 — Normalise the secret key

```
d' = d         if public_key(d).y is even
d' = -d (mod n) if public_key(d).y is odd
```

This is the even-Y normalisation. All signing operations use `d'`.

### Step 2 — Derive the nonce key

```
aux_hash   = keccak256("PIP/aux"   || aux)
t          = aux_hash XOR d'_bytes
nonce_hash = keccak256("PIP/nonce" || t || pubkey_x || msg)
r          = nonce_hash (as secret key; rehash if out of range)
r'         = r           if public_key(r).y is even
r'         = -r (mod n)  if public_key(r).y is odd
```

`aux` is 32 bytes of auxiliary randomness (any source; deterministic inputs are accepted for test vectors).

> The nonce hash commits to both `pubkey_x` and `msg`, binding the nonce to the specific signer and message. This prevents cross-key and cross-message nonce reuse.

### Step 3 — Compute challenge and signature scalar

```
R_x = x-coordinate of public_key(r')
e   = keccak256("PIP/challenge" || R_x || pubkey_x || msg) mod n
s   = (r' + e·d') mod n
```

Signature output is `(R_x, s)`. The full precompile input is `pubkey_x || R_x || s || msg`.

## Return Value Encoding

The precompile returns a 32-byte ABI-encoded `bool`:

- Valid signature → `0x0000...0001` (32 bytes)
- Invalid signature → `0x0000...0000` (32 bytes)
- Invalid input length → reverts with `"Invalid input len"`

## Differences from BIP-340

| Aspect                    | BIP-340                                      | This scheme                                   |
|---------------------------|----------------------------------------------|-----------------------------------------------|
| Hash function             | SHA256                                       | Keccak256                                     |
| Tagged hash construction  | `SHA256(SHA256(tag) \|\| SHA256(tag) \|\| data)` | `keccak256(tag_string \|\| data)`           |
| Nonce hash inputs         | `t \|\| xonly_pk \|\| msg`                   | `t \|\| pubkey_x \|\| msg`                    |
| s out-of-range            | Verification fails (step 2 check)            | Returns `false` via scalar parse failure      |
| Return encoding           | n/a (native)                                 | 32-byte ABI-encoded bool                     |
| Input format              | Native binary API                            | 128-byte ABI call via `ISchnorr.verify`       |

## Security Considerations

- **No BIP-340 interoperability** — do not use off-the-shelf BIP-340 signer libraries; the tagged hash construction differs and will produce signatures that do not verify.
- **Nonce commit to key and message** — the nonce hash includes both `pubkey_x` and `msg`, preventing cross-key nonce reuse.
- **Even-Y enforcement** — both P and R are normalised to even-Y before use, preventing the Y-ambiguity class of forgeries.
- **Aggregation** — because the precompile accepts an arbitrary 32-byte public key and nonce point, it natively supports multi-party or aggregated Schnorr schemes (MuSig2, FROST) as long as the aggregated public key and nonce are provided in the same layout.

## Tooling in This Repository

- `crates/schnorr` — core signing, verification, nonce derivation, and precompile input encoder.
- `pvmcli schnorr sign` — sign a message and print the precompile-ready payload.
- `pvmcli schnorr verify` — verify a signature locally.
- `pvmcli schnorr test-data` — generate Solidity-ready constants; accepts optional `--secret-key` and `--nonce` overrides for deterministic test vectors.
- `solidity/contracts/interfaces/ISchnorr.sol` — Solidity interface.
- `solidity/contracts/modules/Schnorr.sol` — library wrapper around the precompile.
