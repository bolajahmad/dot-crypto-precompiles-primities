// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

struct SchnorrSignature {
    bytes32 pubkey;
    bytes32 nonce;
    bytes32 s;
    bytes32 message;
}