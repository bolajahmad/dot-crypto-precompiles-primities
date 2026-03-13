// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

struct G1Point {
    bytes x;
    bytes y;
}

struct G2Point {
    bytes[2] x;   // [x_real, x_imag]
    bytes[2] y;   // [y_real, y_imag]
}

struct FP { bytes value; }

struct FP2 { bytes[2] value; }  // [real, imag]

struct G1MSM {
    G1Point[] points;
    uint256[] scalars;
}

struct G2MSM {
    G2Point[] points;
    uint256[] scalars;
}