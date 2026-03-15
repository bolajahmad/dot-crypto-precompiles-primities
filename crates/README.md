# Cryptographic Test Vector Crates

This directory contains Rust crates used to generate deterministic test vectors for the PVM cryptographic precompiles.

Each crate demonstrates how the underlying cryptographic primitives work and produces valid inputs and outputs for testing.

These tools help developers:

- understand how precompile inputs are constructed
- generate reproducible cryptographic data
- validate runtime implementations

## Crates

### `schnorr`

Provides Schnorr signing/verification primitives and payload encoders compatible with the Schnorr precompile format.

### `bls12_381`

Provides BLS12-381 helpers used by `pvmcli`, including:

- point generation and addition
- G1/G2 MSM encoding/decoding and execution
- map-to-curve helpers (`Fp -> G1`, `Fp2 -> G2`)
- signature generation, verification, and batch aggregation helpers

## Typical Uses

- deterministic test-vector generation for Solidity and runtime tests
- local verification of cryptographic operation behavior
- payload structure validation before integration with contracts