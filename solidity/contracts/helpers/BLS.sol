// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "../types/BLS.sol";

library BLSTypes {
    error InvalidLength();

    function G1_GENERATOR() internal pure returns (G1Point memory) {
        return
            G1Point({
                x: bytes(
                    hex"17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb"
                ),
                y: bytes(
                    hex"08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1"
                )
            });
    }

    function G2_GENERATOR() internal pure returns (G2Point memory) {
        return
            G2Point({
                x: [
                    bytes(
                        hex"024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8"
                    ),
                    bytes(
                        hex"13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e"
                    )
                ],
                y: [
                    bytes(
                        hex"0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801"
                    ),
                    bytes(
                        hex"0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be"
                    )
                ]
            });
    }

    function encodeG1(G1Point memory p) internal pure returns (bytes memory) {
        if (p.x.length != 64 || p.y.length != 64) revert InvalidLength();
        return abi.encodePacked(p.x, p.y);
    }

    function decodeG1(
        bytes memory data
    ) internal pure returns (G1Point memory p) {
        if (data.length != 128) revert InvalidLength();

        bytes memory x = new bytes(64);
        bytes memory y = new bytes(64);

        for (uint i; i < 64; i++) {
            x[i] = data[i];
            y[i] = data[i + 64];
        }
        p = G1Point(x, y);
    }

    function encodeG2(G2Point memory p) internal pure returns (bytes memory) {
        if (
            p.x[0].length != 64 ||
            p.x[1].length != 64 ||
            p.y[0].length != 64 ||
            p.y[1].length != 64
        ) revert InvalidLength();
        return abi.encodePacked(p.x[0], p.x[1], p.y[0], p.y[1]);
    }

    function decodeG2(bytes memory p) internal pure returns (G2Point memory) {
        if (p.length != 128) revert InvalidLength();

        bytes memory x0 = new bytes(64);
        bytes memory x1 = new bytes(64);
        bytes memory y0 = new bytes(64);
        bytes memory y1 = new bytes(64);

        for (uint i; i < 64; i++) {
            x0[i] = p[i];
            x1[i] = p[i + 64];
            y0[i] = p[i + 128];
            y1[i] = p[i + 192];
        }
        return G2Point([x0, x1], [y0, y1]);
    }

    function encodeFp(FP memory fp) internal pure returns (bytes memory) {
        return fp.value;
    }

    function decodeFp(bytes memory data) internal pure returns (FP memory fp) {
        if (data.length != 32) revert InvalidLength();
        fp = FP(data);
    }

    function encodeFp2(FP2 memory fp2) internal pure returns (bytes memory) {
        return abi.encodePacked(fp2.value[0], fp2.value[1]);
    }

    function decodeFp2(
        bytes memory data
    ) internal pure returns (FP2 memory fp2) {
        if (data.length != 128) revert InvalidLength();

        bytes memory r = new bytes(64);
        bytes memory im = new bytes(64);

        for (uint i; i < 64; i++) {
            r[i] = data[i];
            im[i] = data[i + 64];
        }
        fp2 = FP2([r, im]);
    }
}
