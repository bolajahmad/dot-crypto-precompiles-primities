# Solidity Precompile Libraries

This directory contains Solidity libraries that simplify interacting with the PVM cryptographic precompiles.

Instead of manually constructing low-level calls, developers can import these libraries and call strongly-typed functions.

The libraries handle:

- ABI encoding
- precompile address routing
- decoding results

Example usage:

```
import "pvm-precompiles/BLS.sol";

BLS.G1Point memory result = BLS.g1Add(a, b);
```

## Included Components

- `contracts/Precompiles.sol`: low-level precompile address helpers
- `contracts/modules/BLS.sol`: BLS operation wrappers
- `contracts/modules/Schnorr.sol`: Schnorr verification wrappers
- `contracts/types/*.sol`: strongly-typed structs shared across modules
- `contracts/examples/*.sol`: sample usage contracts

## Supported Flows

- G1/G2 point addition via BLS module helpers
- G1/G2 MSM payload execution paths
- map-to-curve operation inputs/outputs
- Schnorr verification integration
- BLS single and batch signature verification workflows

This structure improves developer experience when integrating advanced cryptographic primitives into contracts.

Example contracts demonstrating usage are included in this directory.
