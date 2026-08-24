use std::{collections::BTreeMap, println, vec};

use parity_scale_codec::{Decode, Encode};
use staging_xcm::{
    VersionedXcm,
    v3::{
        AssetId as V3Aid, Fungibility as V3Fun, Instruction as InstrV3, Junction as V3Junction,
        Junctions as V3Junctions, MultiAsset, Xcm as V3Xcm,
    },
    v4::{
        Asset as V4Asset, AssetId as V4Aid, Fungibility as V4Fun, Instruction as InstrV4,
        Junction as V4Junction, Junctions as V4Junctions, Xcm as V4Xcm,
    },
};

fn format_junction(junction: &V3Junction) -> String {
    match junction {
        V3Junction::AccountId32 { id, .. } => {
            let hex_id = hex::encode(id);
            return format!("AccountId32: 0x{}...", &hex_id[0..6]);
        }
        other => format!("{:?}", other),
    }
}

fn format_multi_location(id: V3Aid) -> String {
    match id {
        V3Aid::Concrete(location) => {
            let parent_str = if location.parents > 0 {
                return format!("parents: {}, ", location.parents);
            } else {
                "".to_string()
            };
            match &location.interior {
                V3Junctions::Here => format!("{}Here", parent_str),
                V3Junctions::X1(junction) => format!("{}{}", parent_str, format_junction(junction)),
                other => format!("{}{:?}", parent_str, other),
            }
        }
        V3Aid::Abstract(bytes) => format!("Abstract(0x{})", hex::encode(bytes)),
    }
}
fn format_multi_asset(multi_asset: &MultiAsset) -> String {
    let location = format_multi_location(multi_asset.id);

    let (asset_type, amount) = match multi_asset.fun {
        V3Fun::Fungible(amount) => ("Native", amount),
        V3Fun::NonFungible(amt) => ("NFT", 0),
    };
    let formatted_amount = amount
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",");

    return format!(
        "Location: {}, Asset: {}, Amount: {}",
        location, asset_type, formatted_amount
    );
}

fn format_asset(asset: &V4Asset) -> String {
    let location = format_location(&asset.id);
    let (asset_type, amount) = match asset.fun {
        V4Fun::Fungible(amount) => ("Native", amount),
        V4Fun::NonFungible(amt) => ("NFT", 0),
    };
    let formatted_amount = amount
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",");

    return format!(
        "Location: {}, Asset: {}, Amount: {}",
        location, asset_type, formatted_amount
    );
}
fn format_location(aid: &V4Aid) -> String {
    let parent_str = if aid.0.parents > 0 {
        return format!("parents: {}, ", aid.0.parents);
    } else {
        "".to_string()
    };

    match &aid.0.interior {
        V4Junctions::Here => format!("{}Here", parent_str),
        V4Junctions::X1(junction) => format!("{}{}", parent_str, "hel"),
        other => format!("{}{:?}", parent_str, other),
    }
}
fn format_v4_junction(junction: &V4Junction) -> String {
    match junction {
        V4Junction::AccountId32 { id, .. } => {
            let hex_id = hex::encode(id);
            return format!("AccountId32: 0x{}...", &hex_id[0..6]);
        }
        other => format!("{:?}", other),
    }
}

pub trait XcmParser {
    fn parse_message(&self) -> Vec<XcmInstruction>;
}

impl XcmParser for V3Xcm<()> {
    fn parse_message(&self) -> Vec<XcmInstruction> {
        self.0
            .iter()
            .map(|instr| {
                let mut params = BTreeMap::new();
                let name;

                match instr {
                    InstrV3::WithdrawAsset(multi_asset) => {
                        name = "WithdrawAsset".to_string();
                        params.insert(
                            "Assets".to_string(),
                            multi_asset
                                .inner()
                                .iter()
                                .map(|asset| format_multi_asset(asset))
                                .collect(),
                        );
                    }
                    other => {
                        name = format!("{:?}", other)
                            .split('(')
                            .next()
                            .unwrap_or("Unknown")
                            .to_string();
                        params.insert("Raw".to_string(), vec![format!("{:#?}", other)]);
                    }
                };

                XcmInstruction { name, params }
            })
            .collect()
    }
}

impl XcmParser for V4Xcm<()> {
    fn parse_message(&self) -> Vec<XcmInstruction> {
        self.inner()
            .iter()
            .map(|instr| {
                let mut params = BTreeMap::new();
                let name;

                match instr {
                    InstrV4::WithdrawAsset(assets) => {
                        name = "WithdrawAsset".to_string();
                        params.insert(
                            "Assets".to_string(),
                            assets
                                .inner()
                                .iter()
                                .map(|asset| format_asset(asset))
                                .collect(),
                        );
                    }
                    other => {
                        name = format!("{:?}", other)
                            .split('(')
                            .next()
                            .unwrap_or("Unknown")
                            .to_string();
                        params.insert("Raw".to_string(), vec![format!("{:#?}", other)]);
                    }
                }
                XcmInstruction { name, params }
            })
            .collect()
    }
}

#[derive(Default)]
pub struct CError {
    pub error: String,
    pub message: String,
}

impl CError {
    pub fn new(err: String, message: &str) -> Self {
        Self {
            error: err,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct XcmInstruction {
    name: String,
    params: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Default)]
pub struct XcmMessage {
    /// XCM Version, (3 | 4 | 5)
    version: u32,
    /// Total Number of Instructions,
    /// excludes internal messages withing a Transact, for instance)
    size: usize,
    /// The total bytes in the encoded XCM message
    bytes: usize,
    /// A list of each Instruction
    instructions: Vec<XcmInstruction>,
}

impl XcmMessage {
    fn new() -> Self {
        Self::default()
    }
}

pub fn decode_xcm_message(message: String) -> Result<XcmMessage, CError> {
    let raw_bytes = match hex::decode(message) {
        Ok(bytes) => bytes,
        Err(e) => {
            return Err(CError {
                error: e.to_string(),
                message: "Invalid character suspected!".to_string(),
            });
        }
    };

    let mut instructions = Vec::new();

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

    match &xcm {
        VersionedXcm::V3(xcm) => {
            instructions = xcm.parse_message();
        }
        VersionedXcm::V4(xcm) => {
            instructions = xcm.parse_message();
        }
        VersionedXcm::V5(xcm) => {
            println!("XCM message, {xcm:?}, {}", size);
        }
    }

    Ok(XcmMessage {
        version,
        size,
        bytes: xcm.encode().len(),
        instructions,
    })
}
