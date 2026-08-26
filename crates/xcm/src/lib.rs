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
    v5::{
        Asset as V5Asset, AssetId as V5Aid, Instruction as InstrV5, Junctions as V5Junctions,
        OriginKind, Xcm as V5Xcm,
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

pub trait XcmAssetFormatter {
    fn format_fungible_to_string(&self) -> String;

    fn format_amount_to_string(&self) -> String;

    fn format_assetid_to_string(&self) -> (u8, String);
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

impl XcmParser for V5Xcm<()> {
    fn parse_message(&self) -> Vec<XcmInstruction> {
        self.0
            .iter()
            .map(|instr| {
                let mut params = BTreeMap::new();
                let name;

                match instr {
                    InstrV5::WithdrawAsset(assets) => {
                        name = "WithdrawAsset".to_string();
                        let mut assets_rows = Vec::new();
                        for asset in assets.inner().iter() {
                            assets_rows
                                .push(format!("Location: {}", asset.format_assetid_to_string().1));
                            assets_rows
                                .push(format!("Asset: {}", asset.format_fungible_to_string()));
                            assets_rows
                                .push(format!("Amount: {}", asset.format_amount_to_string()));
                        }
                        params.insert("Assets".to_string(), assets_rows);
                    }
                    InstrV5::PayFees { asset } => {
                        name = "PayFees".to_string();
                        let mut assets_rows = Vec::new();
                        assets_rows
                            .push(format!("Location: {}", asset.format_assetid_to_string().1));
                        assets_rows.push(format!("Asset: {}", asset.format_fungible_to_string()));
                        assets_rows.push(format!("Amount: {}", asset.format_amount_to_string()));
                        params.insert("Asset".to_string(), assets_rows);
                    }
                    InstrV5::Transact {
                        origin_kind,
                        fallback_max_weight,
                        call,
                    } => {
                        name = "Transact".to_string();

                        // Parse the origin kind
                        let origin = match origin_kind {
                            OriginKind::Native => "Native".to_string(),
                            OriginKind::SovereignAccount => "SovereignAccount".to_string(),
                            OriginKind::Xcm => "XCM".to_string(),
                            OriginKind::Superuser => "Super-User".to_string(),
                        };
                        params.insert("Origin Kind".to_string(), vec![origin]);
                        params.insert(
                            "Maximum Fallback Weight".to_string(),
                            vec![format!("{:?}", fallback_max_weight)],
                        );

                        let encoded_call = call.clone().into_encoded();
                        let call_str = format!("0x{}", hex::encode(&encoded_call));
                        let call_summary: String = if call_str.len() > 34 {
                            format!(
                                "{:?}.. ({} bytes total",
                                &call_str[0..32],
                                encoded_call.len()
                            )
                        } else {
                            call_str
                        };
                        params.insert("CallPayload".to_string(), vec![call_summary]);
                    }
                    InstrV5::RefundSurplus => {
                        name = "Refund Surplus".to_string();
                        params.insert("Execution Note".to_string(), vec!["Calculates unspent execution fees dynamically at runtime.".to_string(),
                        "Moves surplus weight from the holding register to the asset register.".to_string()]);
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

impl XcmAssetFormatter for V5Asset {
    fn format_fungible_to_string(&self) -> String {
        match self.fun {
            staging_xcm::v5::Fungibility::Fungible(_) => "Native".to_string(),
            staging_xcm::v5::Fungibility::NonFungible(_) => "NFT".to_string(),
        }
    }

    fn format_amount_to_string(&self) -> String {
        let amount = match self.fun {
            staging_xcm::v5::Fungibility::Fungible(amt) => amt,
            staging_xcm::v5::Fungibility::NonFungible(_) => 0,
        };

        amount
            .to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(std::str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()
            .unwrap()
            .join(",")
    }

    fn format_assetid_to_string(&self) -> (u8, String) {
        let interior = match self.id.0.interior() {
            V5Junctions::Here => "Here".to_string(),
            other => format!("{:?}", other),
        };
        (self.id.0.parents, interior)
    }
}

#[derive(Default, Debug)]
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

#[derive(Debug, Default, Clone)]
pub struct XcmInstruction {
    pub name: String,
    pub params: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Default, Clone)]
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
    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn instructions(&self) -> Vec<XcmInstruction> {
        self.clone().instructions
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }
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
            instructions = xcm.parse_message();
        }
    }

    Ok(XcmMessage {
        version,
        size,
        bytes: xcm.encode().len(),
        instructions,
    })
}
