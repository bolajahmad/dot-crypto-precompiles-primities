# PVM CLI

The PVM CLI is a developer utility for interacting with cryptographic precompiles.

It provides commands that generate valid inputs and expected outputs for various cryptographic operations.

## Installation

```bash
cd pvm-cli
cargo build --release
```

The binary will be available at `target/release/pvmcli`.

## Usage

### BLS12-381 Point Operations

#### Generate Random Points

```bash
# Generate a random G1 point
pvmcli bls random-g1

# Generate a random G2 point
pvmcli bls random-g2
```

Each command prints the point in three forms:
- full precompile hex encoding
- JSON shaped like the Solidity struct fields
- Solidity-friendly struct literal

#### Add G1 Points

```bash
# Using full hex point encodings
pvmcli bls g1-add \
  --point-a 0x<G1_POINT_A_128_BYTES> \
  --point-b 0x<G1_POINT_B_128_BYTES>

# Using Solidity-shaped JSON
pvmcli bls g1-add \
  --point-a '{"x":"0x<X_COORD>","y":"0x<Y_COORD>"}' \
  --point-b '{"x":"0x<X_COORD>","y":"0x<Y_COORD>"}'
```

#### Add G2 Points

```bash
# Using full hex point encodings
pvmcli bls g2-add \
  --point-a 0x<G2_POINT_A_256_BYTES> \
  --point-b 0x<G2_POINT_B_256_BYTES>

# Using Solidity-shaped JSON
pvmcli bls g2-add \
  --point-a '{"x":["0x<X0>","0x<X1>"],"y":["0x<Y0>","0x<Y1>"]}' \
  --point-b '{"x":["0x<X0>","0x<X1>"],"y":["0x<Y0>","0x<Y1>"]}'
```

The JSON input mirrors the Solidity library layout in [../solidity/contracts/types/BLS.sol](../solidity/contracts/types/BLS.sol).

#### G1/G2 MSM Testdata Generation

```bash
# Generate deterministic G1 MSM testdata with 3 pairs
pvmcli bls g1-msm-testdata --pairs 3 --output both

# Generate deterministic G2 MSM testdata with 2 pairs
pvmcli bls g2-msm-testdata --pairs 2 --output json
```

This returns MSM payloads in:
- full precompile hex encoding
- Solidity-shaped JSON (`points` + `scalars`)

#### G1/G2 MSM Execution

```bash
# Execute G1 MSM with hex-encoded input (k * 160 bytes)
pvmcli bls g1-msm --data 0x<MSM_INPUT_HEX>

# Execute G2 MSM with Solidity-shaped JSON
pvmcli bls g2-msm --data '{"points":[...],"scalars":["0x...","123"]}'
```

#### G1/G2 MSM Validation

```bash
# Validate G1 MSM payload before computation
pvmcli bls g1-msm-validate --data 0x<MSM_INPUT_HEX>

# Validate G2 MSM payload before computation
pvmcli bls g2-msm-validate --data '{"points":[...],"scalars":[...]}'
```

Invalid payloads are rejected with a descriptive error and non-zero exit code.

#### Map Fp -> G1 and Fp2 -> G2

```bash
# Random Fp input if --fp is omitted
pvmcli bls map-fp-to-g1

# Explicit Fp input (32 or 64-byte hex, or JSON)
pvmcli bls map-fp-to-g1 --fp 0x<FP_HEX>
pvmcli bls map-fp-to-g1 --fp '{"value":"0x<FP_HEX>"}'

# Random Fp2 input if --fp2 is omitted
pvmcli bls map-fp2-to-g2

# Explicit Fp2 input (128-byte hex, or JSON)
pvmcli bls map-fp2-to-g2 --fp2 0x<FP2_HEX>
pvmcli bls map-fp2-to-g2 --fp2 '{"value":["0x<C0>","0x<C1>"]}'
```

#### BLS Signature Generation and Verification

```bash
# Sign one message with a secret key
pvmcli bls sign --secret-key 42 --message "hello"

# Verify one signature using pairing logic
pvmcli bls verify \
  --signature '{"x":"0x<...>","y":"0x<...>"}' \
  --pubkey '{"x":["0x<...>","0x<...>"],"y":["0x<...>","0x<...>"]}' \
  --message "hello"
```

#### Batch Signature Workflows

```bash
# Generate deterministic batch-signature test data
pvmcli bls batch-sign-testdata --count 4 --output both

# Aggregate multiple signatures (hex or JSON payload)
pvmcli bls batch-aggregate --signatures '{"signatures":[...G1 points...]}'

# Verify batch signatures with optional aggregated signature check
pvmcli bls batch-verify --data '{"messages":[...],"signatures":[...],"pubkeys":[...],"aggregated_signature":{...}}'

# End-to-end smoke flow (generate -> aggregate -> verify)
pvmcli bls batch-smoke --count 4 --output summary
pvmcli bls batch-smoke --count 4 --output json
```

`batch-verify` checks each signature against its message/public key using pairing logic and also checks aggregate consistency when an `aggregated_signature` is provided.

### Schnorr Signature Operations

#### Generate a Signature

```bash
# Basic usage
pvmcli schnorr sign --secret-key <HEX> --message "Hello, world!"

# With custom aux (nonce randomness)
pvmcli schnorr sign \
  --secret-key 0101010101010101010101010101010101010101010101010101010101010101 \
  --message "Hello, world!" \
  --aux 0202020202020202020202020202020202020202020202020202020202020202

# JSON output
pvmcli schnorr sign --secret-key <HEX> --message "Test" --output json
```

**Options:**
- `-s, --secret-key` — Secret key as hex (32 bytes / 64 hex chars)
- `-m, --message` — Message to sign (will be hashed with keccak256)
- `-a, --aux` — Optional auxiliary randomness for nonce (32 bytes hex)
- `-o, --output` — Output format: `hex` (default) or `json`

#### Verify a Signature

```bash
pvmcli schnorr verify \
  --pubkey <PUBKEY_X_HEX> \
  --nonce <R_X_HEX> \
  --signature <S_HEX> \
  --message "Hello, world!"
```

**Options:**
- `-p, --pubkey` — Public key x-coordinate (32 bytes hex)
- `-n, --nonce` — Nonce point R x-coordinate (32 bytes hex)
- `-s, --signature` — Signature scalar s (32 bytes hex)
- `-m, --message` — Original message (will be hashed)

#### Generate Test Data for Solidity

```bash
pvmcli schnorr test-data --message "Hello, world!"

# Optional deterministic test vector overrides
pvmcli schnorr test-data \
  --message "Hello, world!" \
  --secret-key 0101010101010101010101010101010101010101010101010101010101010101 \
  --nonce 0202020202020202020202020202020202020202020202020202020202020202
```

This generates ready-to-use Solidity constants and struct initialization code for testing.

**Options:**
- `-m, --message` — Message to sign (will be hashed with keccak256)
- `-s, --secret-key` — Optional secret key override (32 bytes / 64 hex chars)
- `-n, --nonce` — Optional nonce seed override (32 bytes / 64 hex chars)

## Examples

### Sign and Verify Flow

```bash
# Generate signature
$ pvmcli schnorr sign \
  --secret-key 0101010101010101010101010101010101010101010101010101010101010101 \
  --message "Test message"

=== Schnorr Signature Generated ===
Public Key (x):     0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798
Nonce (R_x):        0x...
Signature (s):      0x...
Message Hash:       0x...
---
Precompile Input (128 bytes):
0x...

# Verify the signature
$ pvmcli schnorr verify \
  --pubkey 79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798 \
  --nonce <NONCE_FROM_ABOVE> \
  --signature <SIGNATURE_FROM_ABOVE> \
  --message "Test message"

✓ Signature is VALID
```

### Generate Solidity Test Data

```bash
$ pvmcli schnorr test-data --message "Hello"

=== Schnorr Test Data for Solidity ===
Message: "Hello"

// Solidity test data
bytes32 constant PUBKEY_X = 0x...;
bytes32 constant NONCE_RX = 0x...;
bytes32 constant SIGNATURE_S = 0x...;
bytes32 constant MESSAGE_HASH = 0x...;

// Full precompile input (128 bytes)
bytes constant PRECOMPILE_INPUT = hex"...";

// For SchnorrSignature struct
SchnorrSignature memory sig = SchnorrSignature({
    pubkey: 0x...,
    nonce: 0x...,
    s: 0x...,
    message: 0x...
});
```

## Notes

- Invalid BLS/Schnorr payloads return a descriptive error and a non-zero exit code.
- Most BLS commands support either full precompile hex input or Solidity-shaped JSON input.
- `batch-smoke` is the fastest end-to-end local correctness check for batch signature flows.