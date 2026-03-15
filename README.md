# PVM Precompiles Developer Toolkit

This repository contains developer tooling, Solidity helper libraries, and test infrastructure for interacting with cryptographic precompiles implemented for the PVM runtime.

The goal is to make advanced cryptographic functionality accessible to smart contracts running on parachains that integrate `pallet-revive`.

## Runtime Precompile Implementations

The core runtime precompile logic lives in a Polkadot SDK fork under `pallet-revive`:

- Schnorr Signature Verification (BIP-340-inspired)
	- https://github.com/bolajahmad/polkadot-sdk/blob/2fdb7206d942fc4a7a677261131ba4fa30c4b54f/substrate/frame/revive/src/precompiles/builtin/schnorr.rs
- BLS12-381 Operations (EIP-2537-inspired)
	- https://github.com/bolajahmad/polkadot-sdk/blob/2fdb7206d942fc4a7a677261131ba4fa30c4b54f/substrate/frame/revive/src/precompiles/builtin/bls12.rs

Implemented runtime-facing primitives include:

- BLS12-381 G1 and G2 point operations
- G1/G2 multi-scalar multiplication (MSM)
- Mapping `Fp -> G1` and `Fp2 -> G2`
- Pairing-based signature checks
- Schnorr signature verification

These operations are exposed as native precompiles, so contracts can execute them with far lower gas/weight than equivalent Solidity implementations.

## Purpose of This Repository

This repository is the developer-facing layer around those runtime precompiles. It provides:

- [Solidity helper libraries](solidity/README.md) for contract integration
- [CLI tooling](pvm-cli/README.md) to generate/validate payloads and run local cryptographic flows
- [Rust crates](crates/README.md) for deterministic test vectors and core operation utilities
- Solidity example contracts in `solidity/contracts/examples`

## Repository Structure

```
root
├─ crates/
│  └─ Rust crates for Schnorr and BLS test vectors/operations
├─ pvm-cli/
│  └─ Command line tooling for Schnorr and BLS workflows
├─ solidity/
│  └─ Solidity libraries, interfaces, modules, examples, and scripts
└─ docs/
	 └─ Architecture, integration, and roadmap documentation
```

## Current Feature Coverage

### Schnorr

- Sign and verify flows in `pvmcli schnorr`
- Deterministic test-data generation for Solidity
- Optional `secret_key` and `nonce` overrides for `test-data`

### BLS12-381

- Random G1/G2 generation and G1/G2 addition
- G1/G2 MSM testdata generation, execution, and payload validation
- Mapping `Fp -> G1` and `Fp2 -> G2`
- Single-signature sign/verify flows
- Batch-signature testdata generation, aggregation, verification, and one-command smoke flow

## Documentation Index

- [How precompiles work](docs/how-precompiles-work.md)
- [Building with precompiles](docs/building-with-precompiles.md)
- [Project roadmap](docs/roadmap.md)
- [PVM CLI guide](pvm-cli/README.md)
- [Solidity integration guide](solidity/README.md)
- [Rust crates guide](crates/README.md)

## Architecture Overview

```
Solidity Contract
		-> Solidity Helper Library
				-> Precompile Address Call
						-> PVM Runtime Dispatcher (pallet-revive)
								-> Rust Cryptographic Implementation
										-> Encoded Result Back to Contract
```

## Status

The toolkit currently provides production-focused development support for Schnorr and BLS precompiles, including deterministic test vectors, strict payload validation, and Solidity-friendly data shapes.

## License

MIT