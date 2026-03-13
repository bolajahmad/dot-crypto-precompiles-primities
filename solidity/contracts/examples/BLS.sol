// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "../Precompiles.sol";

/// @title BLS Precompile Example Contract
/// @notice Demonstrates how to interact with BLS12-381 precompiles using the library wrappers.
/// @dev This contract showcases common BLS operations such as point addition,
/// multi-scalar multiplication (MSM), signature aggregation, and mapping field
/// elements to elliptic curve points.
contract BLSContract {

    /// @notice Adds two G1 points using the BLS precompile.
    /// @param a First G1 point.
    /// @param b Second G1 point.
    /// @return result The resulting G1 point after addition.
    function addG1Points(
        G1Point memory a,
        G1Point memory b
    ) public view returns (G1Point memory result) {
        return BLS.g1AddPoint(a, b);
    }

    /// @notice Adds two G2 points using the BLS precompile.
    /// @param a First G2 point.
    /// @param b Second G2 point.
    /// @return result The resulting G2 point after addition.
    function addG2Points(
        G2Point memory a,
        G2Point memory b
    ) public view returns (G2Point memory result) {
        return BLS.g2AddPoint(a, b);
    }

    /// @notice Performs a multi-scalar multiplication (MSM) on G1 points.
    /// @dev Each point is multiplied by its corresponding scalar and all
    /// results are summed together using the precompile.
    /// @param msm Struct containing G1 points and their associated scalars.
    /// @return result The resulting G1 point after MSM computation.
    function performG1Msm(
        G1MSM memory msm
    ) public view returns (G1Point memory result) {
        return BLS.g1MSM(msm);
    }

    /// @notice Performs a multi-scalar multiplication (MSM) on G2 points.
    /// @param msm Struct containing G2 points and their associated scalars.
    /// @return result The resulting G2 point after MSM computation.
    function performG2Msm(
        G2MSM memory msm
    ) public view returns (G2Point memory result) {
        return BLS.g2MSM(msm);
    }

    /// @notice Aggregates multiple BLS signatures into a single signature.
    /// @dev Aggregation is performed via repeated G1 additions.
    /// @param signatures Array of individual BLS signatures represented as G1 points.
    /// @return aggregatedSignature The aggregated signature.
    function aggregate(
        G1Point[] memory signatures
    ) public view returns (G1Point memory aggregatedSignature) {
        return BLS.aggregateSignatures(signatures);
    }

    /// @notice Maps a field element (Fp) to a G1 point.
    /// @dev Uses the BLS12-381 `mapFpToG1` precompile defined in the spec.
    /// @param fp Field element in Fp.
    /// @return point Corresponding point on the G1 curve.
    function mapFpToG1(FP memory fp) public view returns (G1Point memory point) {
        return BLS.mapFpToG1Point(fp);
    }

    /// @notice Maps a quadratic field element (Fp2) to a G2 point.
    /// @dev Uses the BLS12-381 `mapFp2ToG2` precompile.
    /// @param fp2 Field element in Fp².
    /// @return point Corresponding point on the G2 curve.
    function mapFp2ToG2(FP2 memory fp2) public view returns (G2Point memory point) {
        return BLS.mapFp2ToG2Point(fp2);
    }
}