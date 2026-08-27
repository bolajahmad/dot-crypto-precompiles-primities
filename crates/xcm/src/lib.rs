//! XCM decode and inspect helpers for PVM precompile tooling.
//!
//! Version-specific parse/format logic lives in [`v3`], [`v4`], and [`v5`].
//! Shared types and utilities sit outside those modules and do not depend on
//! a particular XCM version.

mod types;
mod util;
mod v3;
mod v4;
mod v5;

pub use types::{CError, XcmInstruction, XcmMessage, XcmParser};

use parity_scale_codec::{Decode, Encode};
use staging_xcm::VersionedXcm;

/// Decode a hex-encoded XCM message (with or without version prefix handling)
/// into a version-agnostic [`XcmMessage`].
pub fn decode_xcm_message(message: String) -> Result<XcmMessage, CError> {
    let raw_bytes = hex::decode(message).map_err(|e| CError {
        error: e.to_string(),
        message: "Invalid character suspected!".to_string(),
    })?;

    let (size, version, xcm) = match VersionedXcm::<()>::decode(&mut &raw_bytes[..]) {
        Ok(vxcm) => {
            let (xcm_len, xcm_version) = match &vxcm {
                VersionedXcm::V3(xcm) => (xcm.len(), 3_u32),
                VersionedXcm::V4(xcm) => (xcm.len(), 4_u32),
                VersionedXcm::V5(xcm) => (xcm.len(), 5_u32),
            };
            (xcm_len, xcm_version, vxcm)
        }
        Err(_) => {
            let vxcm = staging_xcm::v5::Xcm::<()>::decode(&mut &raw_bytes[..])
                .map_err(|e| CError::new(e.to_string(), "XCM message could not be interpreted."))?;
            let xcm_len = vxcm.len();
            (xcm_len, 5, VersionedXcm::V5(vxcm))
        }
    };

    let instructions = match &xcm {
        VersionedXcm::V3(xcm) => xcm.parse_message(),
        VersionedXcm::V4(xcm) => xcm.parse_message(),
        VersionedXcm::V5(xcm) => xcm.parse_message(),
    };

    Ok(XcmMessage::new(version, size, xcm.encode().len(), instructions))
}
