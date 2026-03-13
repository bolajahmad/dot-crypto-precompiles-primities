// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "../constants/Addresses.sol";
import "../helpers/Schnorr.sol";
import "../types/Schnorr.sol";

library Schnorr {
    using SchnorrTypes for SchnorrSignature;

    error SchnorrCallFailed();

    function verifySignature(bytes memory input) internal view returns (bool) {
        (bool success, bytes memory result) = PrecompilesAddresses
            .SCHNORR_SIGNATURE
            .staticcall(input);
        if (!success) revert SchnorrCallFailed();
        return abi.decode(result, (bool));
    }

    function verify(SchnorrSignature memory sig) internal view returns (bool) {
        bytes memory input = sig.encode();
        return verifySignature(input);
    }
}
