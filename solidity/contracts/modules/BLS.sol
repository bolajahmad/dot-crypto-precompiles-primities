// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "../constants/Addresses.sol";
import "../types/BLS.sol";
import "../helpers/BLS.sol";

library BLS {
    using BLSTypes for G1Point;
    using BLSTypes for G2Point;
    using BLSTypes for FP;
    using BLSTypes for FP2;

    error BLSCallFailed();
    error MismatchedInputs();

    function g1Add(bytes memory input) internal view returns (bytes memory) {
        return _callPrecompile(PrecompilesAddresses.BLS_G1_ADD, input);
    }

    function g2Add(bytes memory input) internal view returns (bytes memory) {
        return _callPrecompile(PrecompilesAddresses.BLS_G2_ADD, input);
    }

    function g1Mul(bytes memory input) internal view returns (bytes memory) {
        return _callPrecompile(PrecompilesAddresses.BLS_G1_MSM, input);
    }

    function g2Mul(bytes memory input) internal view returns (bytes memory) {
        return _callPrecompile(PrecompilesAddresses.BLS_G2_MSM, input);
    }

    function pairingCheck(bytes memory input) internal view returns (bool) {
        bytes memory out = _callPrecompile(
            PrecompilesAddresses.BLS_FIELD_PAIRING,
            input
        );
        return abi.decode(out, (bool));
    }

    function mapFpToG1(
        bytes memory input
    ) internal view returns (bytes memory) {
        return _callPrecompile(PrecompilesAddresses.BLS_FP_TO_G1, input);
    }

    function mapFp2ToG2(
        bytes memory input
    ) internal view returns (bytes memory) {
        return _callPrecompile(PrecompilesAddresses.BLS_FP2_TO_G2, input);
    }

    function g1AddPoint(
        G1Point memory a,
        G1Point memory b
    ) internal view returns (G1Point memory) {
        bytes memory input = abi.encodePacked(a.encodeG1(), b.encodeG1());
        bytes memory out = g1Add(input);

        return BLSTypes.decodeG1(out);
    }

    function g2AddPoint(
        G2Point memory a,
        G2Point memory b
    ) internal view returns (G2Point memory) {
        bytes memory input = abi.encodePacked(a.encodeG2(), b.encodeG2());
        bytes memory out = g2Add(input);

        return BLSTypes.decodeG2(out);
    }

    function mapFpToG1Point(
        FP memory fp
    ) internal view returns (G1Point memory) {
        bytes memory input = fp.encodeFp();
        bytes memory out = mapFpToG1(input);
        return BLSTypes.decodeG1(out);
    }

    function mapFp2ToG2Point(
        FP2 memory fp2
    ) internal view returns (G2Point memory) {
        bytes memory input = fp2.encodeFp2();
        bytes memory out = mapFp2ToG2(input);
        return BLSTypes.decodeG2(out);
    }

    function pairingCheckPoints(
        G1Point[] memory g1Points,
        G2Point[] memory g2Points
    ) internal view returns (bool) {
        if (g1Points.length != g2Points.length) revert MismatchedInputs();

        bytes memory input;
        for (uint256 i = 0; i < g1Points.length; i++) {
            input = abi.encodePacked(
                input,
                g1Points[i].encodeG1(),
                g2Points[i].encodeG2()
            );
        }

        return pairingCheck(input);
    }

    function g1MSM(G1MSM memory msm) internal view returns (G1Point memory) {
        if (msm.points.length != msm.scalars.length) revert MismatchedInputs();

        bytes memory input;
        for (uint i = 0; i < msm.points.length; i++) {
            input = abi.encodePacked(
                input,
                msm.points[i].encodeG1(),
                msm.scalars[i]
            );
        }
        bytes memory out = g1Mul(input);
        return BLSTypes.decodeG1(out);
    }

    function g2MSM(G2MSM memory msm) internal view returns (G2Point memory) {
        if (msm.points.length != msm.scalars.length) revert MismatchedInputs();

        bytes memory input;
        for (uint i = 0; i < msm.points.length; i++) {
            input = abi.encodePacked(
                input,
                msm.points[i].encodeG2(),
                msm.scalars[i]
            );
        }
        bytes memory out = g2Mul(input);
        return BLSTypes.decodeG2(out);
    }

    function aggregateSignatures(
        G1Point[] memory sigs
    ) internal view returns (G1Point memory) {
        if (sigs.length == 0) revert MismatchedInputs();
        G1Point memory agg = sigs[0];
        for (uint256 i = 1; i < sigs.length; i++) {
            agg = g1AddPoint(agg, sigs[i]);
        }

        return agg;
    }

    function verifyAggregatedSignatures(
        G1Point memory aggSig,
        G2Point[] memory pubkeys,
        G1Point[] memory msgs
    ) internal view returns (bool) {
        if (pubkeys.length != msgs.length) revert MismatchedInputs();

        G1Point[] memory g1Points = new G1Point[](pubkeys.length + 1);
        G2Point[] memory g2Points = new G2Point[](pubkeys.length + 1);

        // Add the aggregated signature and the generator point
        g1Points[0] = aggSig;
        g2Points[0] = BLSTypes.G2_GENERATOR();

        for (uint256 i = 0; i < pubkeys.length; i++) {
            g1Points[i + 1] = msgs[i];
            g2Points[i + 1] = pubkeys[i];
        }

        return pairingCheckPoints(g1Points, g2Points);
    }

    function hashToG1(
        bytes memory message
    ) internal view returns (G1Point memory) {
        bytes memory h = abi.encodePacked(keccak256(message));
        FP memory fp = FP(h);
        return mapFpToG1Point(fp);
    }

    function hashToG2(
        bytes memory message
    ) internal view returns (G2Point memory) {
        bytes memory h = abi.encodePacked(keccak256(message));
        FP2 memory fp2 = FP2([h, bytes("")]);
        return mapFp2ToG2Point(fp2);
    }

    function isValidG1(G1Point memory p) internal pure returns (bool) {
        return !(p.x.length == 0 && p.y.length == 0);
    }

    function isValidG2(G2Point memory p) internal pure returns (bool) {
        return !(p.x[0].length == 0 && p.x[1].length == 0 && p.y[0].length == 0 && p.y[1].length == 0);
    }

    // --- Central dispatcher ---
    function _callPrecompile(
        address target,
        bytes memory input
    ) private view returns (bytes memory) {
        (bool success, bytes memory out) = target.staticcall(input);
        if (!success) revert BLSCallFailed();
        return out;
    }
}
