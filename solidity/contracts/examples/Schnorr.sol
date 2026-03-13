// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "../Precompiles.sol";

/// @title Schnorr Signature Verification Example
/// @notice Demonstrates how to verify Schnorr signatures using the Schnorr precompile wrapper.
/// @dev The Schnorr signature verification precompile expects a 128-byte input
/// containing the nonce, public key, message, and signature scalar.
contract SchnorrContract {

    /// @notice Verifies a Schnorr signature.
    /// @param sig The Schnorr signature struct containing nonce, public key,
    /// message hash, and signature scalar.
    /// @return valid True if the signature is valid according to the precompile.
    function verify(
        SchnorrSignature memory sig
    ) public view returns (bool valid) {
        return Schnorr.verify(sig);
    }
}