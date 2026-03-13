// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "../types/Schnorr.sol";

library SchnorrTypes {
    error InvalidLength();

    function encode(
        SchnorrSignature memory sig
    ) internal pure returns (bytes memory) {
        if (
            sig.nonce.length != 32 ||
            sig.pubkey.length != 32 ||
            sig.message.length != 32 ||
            sig.s.length != 32
        ) revert InvalidLength();

        return abi.encodePacked(sig.pubkey, sig.nonce, sig.s, sig.message);
    }

    function decode(
        bytes memory data
    ) internal pure returns (SchnorrSignature memory sig) {
        if (data.length != 128) revert InvalidLength();

        for (uint i; i < 32; i++) {
            sig.pubkey = data[i];
            sig.nonce = data[32 + i];
            sig.message = data[64 + i];
            sig.s = data[96 + i];
        }
    }

    function fromParts(bytes memory pubkey, bytes memory nonce, bytes memory s, bytes memory message) internal pure returns (SchnorrSignature memory sig) {
        if (
            pubkey.length != 32 ||
            nonce.length != 32 ||
            s.length != 32 ||
            message.length != 32
        ) revert InvalidLength();

        sig.pubkey = bytes32(pubkey);
        sig.nonce = bytes32(nonce);
        sig.s = bytes32(s);
        sig.message = bytes32(message);
    }
}
