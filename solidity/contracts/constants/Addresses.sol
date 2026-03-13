// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

library PrecompilesAddresses {
    // BLS precompile addresses
    address constant BLS_G1_ADD = address(0x0b);
    address constant BLS_G2_ADD = address(0x0d);
    address constant BLS_G1_MSM = address(0x0c);
    address constant BLS_G2_MSM = address(0x0e);
    address constant BLS_FIELD_PAIRING = address(0x0f);
    address constant BLS_FP_TO_G1 = address(0x10);
    address constant BLS_FP2_TO_G2 = address(0x11);

    // Schnorr precompile address
    address constant SCHNORR_SIGNATURE = address(0x905);
}