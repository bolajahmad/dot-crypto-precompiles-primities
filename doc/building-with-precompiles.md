# Building With PVM Precompiles

This guide explains how developers can integrate the cryptographic precompiles into their smart contracts.

Instead of manually interacting with precompile addresses and encoding raw calldata, this repository provides Solidity helper libraries that make integration straightforward.

---

## Using the Solidity Library

Import the helper library:

`import "pvm-precompiles/BLS.sol";`


The library wraps low-level precompile calls and exposes typed helper functions.

Example:
```
BLS.G1Point memory result = BLS.g1Add(p1, p2);
```

The library handles:

- ABI encoding
- precompile address routing
- return value decoding

---

## Example Contract

```
contract Example {
    function addPoints(
    BLS.G1Point memory a,
    BLS.G1Point memory b
    ) public returns (BLS.G1Point memory) {
        return BLS.g1Add(a, b);
    }
}
```

---

## CLI Tool

`pvmcli` can generate valid payloads, run local cryptographic operations, and validate payload correctness before contract testing.

Example:
`pvmcli bls g1-add`

Useful command groups:

- Point ops: `random-g1`, `random-g2`, `g1-add`, `g2-add`
- MSM: `g1-msm-testdata`, `g2-msm-testdata`, `g1-msm`, `g2-msm`, `g1-msm-validate`, `g2-msm-validate`
- Mapping: `map-fp-to-g1`, `map-fp2-to-g2`
- Signatures: `sign`, `verify`, `batch-sign-testdata`, `batch-aggregate`, `batch-verify`, `batch-smoke`
- Schnorr: `schnorr sign`, `schnorr verify`, `schnorr test-data`

This generates encoded inputs and expected outputs for testing.

---

## Test Vector Generation

The Rust crates and CLI generate deterministic cryptographic test vectors.

These vectors are useful for:

- validating precompile correctness
- writing contract integration tests
- debugging encoding issues

---

## Recommended Workflow

1. Generate vectors with CLI (hex and JSON outputs).
2. Write/update Solidity contract using helper modules.
3. Call helper library functions in tests/contracts.
4. Compare Solidity results with CLI-expected values.
5. Use CLI validation commands in CI for payload sanity checks.

Example:

```bash
# Generate G1 MSM payload and expected result
pvmcli bls g1-msm-testdata --pairs 4 --output both

# Validate payload shape (non-zero exit on invalid data)
pvmcli bls g1-msm-validate --data 0x<MSM_HEX>

# Compute expected MSM output locally
pvmcli bls g1-msm --data 0x<MSM_HEX>
```

---

## Example Use Cases

Developers can use these precompiles to build:

- aggregated signature verification
- zk proof verification
- decentralized identity systems
- rollup verification contracts

For batch BLS scenarios, `pvmcli bls batch-smoke --count <N>` provides a one-command end-to-end flow (generate, aggregate, verify) suitable for local correctness checks.

These primitives unlock powerful cryptographic functionality that would otherwise be too expensive in Solidity.