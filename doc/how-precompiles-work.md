# How PVM Precompiles Work

This document explains the design and internal mechanics of the cryptographic precompiles implemented for the pallet-revive runtime.

These precompiles allow smart contracts to access high-performance cryptographic primitives that would otherwise be impractical to implement directly in Solidity.

The runtime implementations live inside the `pallet-revive` module in the Polkadot SDK fork. Some precompiles are external to `pallet-revive` (for example ERC20 and XCM) because they need to live close to their state and business logic.

Key implementations:

Schnorr Verification  
https://github.com/bolajahmad/polkadot-sdk/blob/2fdb7206d942fc4a7a677261131ba4fa30c4b54f/substrate/frame/revive/src/precompiles/builtin/schnorr.rs

BLS12-381 Operations  
https://github.com/bolajahmad/polkadot-sdk/blob/2fdb7206d942fc4a7a677261131ba4fa30c4b54f/substrate/frame/revive/src/precompiles/builtin/bls12.rs

---

# What Is a Precompile?

A precompile is a native runtime function exposed to smart contracts via a fixed address. EVM contracts are constrained by opcode-level execution, so expensive cryptographic operations are not practical to re-implement in Solidity.

Precompiles bridge that gap by accepting encoded bytes from contracts, executing optimized logic in Rust, and returning encoded outputs.

Instead of executing expensive computations inside Solidity bytecode, the EVM forwards the call to optimized runtime code written in Rust.

This approach provides:

- significantly lower gas/weight cost for complex cryptographic logic
- deterministic execution
- access to advanced cryptographic primitives

The smart contract simply performs a `staticcall` to a known (and documented) address.

---

# Execution Flow

```
Solidity Contract
│
▼
Precompile Call (staticcall)
│
▼
PVM Runtime Precompile Dispatcher
│
▼
Rust Implementation
│
▼
Cryptographic Primitive
```

---

# Input Encoding

Precompiles receive raw byte arrays as inputs.

The runtime implementation is responsible for:

1. validating input size
2. decoding input bytes
3. Charge a weight, (based on benchmarking)
4. reconstructing needed data for computation
5. performing the relevant cryptographic operation
6. encoding the result

In this repository, `pvmcli` and the Rust crates expose these byte-level payloads directly and also provide Solidity-struct-shaped JSON forms to make integration and testing easier.

---

# Schnorr Signature Precompile

The Schnorr precompile verifies signatures produced over a supported elliptic curve.

Verification involves:

1. parsing the signature
2. computing challenge hash
3. verifying the elliptic curve equation

Read the [specifications here](https://scarlet-zinnia-086.notion.site/Schnorr-Signatures-PIP-30dc5f3533eb80f39767dbffe41ebdaf).

Developer tooling currently available in this repository:

- `pvmcli schnorr sign`
- `pvmcli schnorr verify`
- `pvmcli schnorr test-data` with optional `--secret-key` and `--nonce` overrides

---

# BLS12-381 Precompiles

The BLS implementation exposes several elliptic curve operations, based on the specifications of the [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537).

Examples include:

- G1 point addition
- G2 point addition
- G1/G2 MSM
- map-to-curve (`Fp -> G1`, `Fp2 -> G2`)
- pairing verification
- signature aggregation flows

These primitives are fundamental to many modern cryptographic protocols including:

- zkSNARK verification
- signature aggregation
- polynomial commitments

Developer tooling currently available in this repository:

- random point generation (`random-g1`, `random-g2`)
- point addition (`g1-add`, `g2-add`)
- MSM testdata, execution, and validation commands
- mapping commands (`map-fp-to-g1`, `map-fp2-to-g2`)
- single-signature sign/verify commands
- batch testdata, aggregate, verify, and smoke-flow commands

---

# Error Handling

The precompile implementation validates several conditions before executing operations.

Examples include:

- invalid input length
- malformed field elements
- invalid curve points
- points not belonging to correct subgroup

If validation fails the precompile returns an error.

The CLI mirrors this behavior and exits non-zero on invalid payloads, making it safe to use in CI/test scripts.

---

# Gas Cost

Precompile gas costs are determined based on the complexity of the operation.

More expensive operations such as multi-scalar multiplication scale with the number of inputs.

Benchmarking logic is implemented inside the runtime to ensure predictable gas usage.

---

# Why This Matters

Without precompiles, advanced cryptography in smart contracts is extremely expensive.

Precompiles allow:

- scalable signature verification
- efficient zero-knowledge proof verification
- advanced cryptographic protocols on-chain

This significantly expands the capabilities of smart contract systems built on PVM.
