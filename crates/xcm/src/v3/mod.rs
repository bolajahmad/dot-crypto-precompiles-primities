use std::collections::BTreeMap;

use staging_xcm::v3::{
    AssetId, Fungibility, Instruction, Junction, Junctions, MultiAsset, Xcm,
};

use crate::types::{XcmInstruction, XcmParser};
use crate::util::{format_amount, raw_instruction, short_hex};

fn format_junction(junction: &Junction) -> String {
    match junction {
        Junction::AccountId32 { id, .. } => format!("AccountId32: {}", short_hex(id)),
        Junction::Parachain(id) => format!("Parachain({id})"),
        other => format!("{other:?}"),
    }
}

fn format_multi_location(id: AssetId) -> String {
    match id {
        AssetId::Concrete(location) => {
            let parent_str = if location.parents > 0 {
                format!("parents: {}, ", location.parents)
            } else {
                String::new()
            };
            match &location.interior {
                Junctions::Here => format!("{parent_str}Here"),
                Junctions::X1(junction) => {
                    format!("{parent_str}{}", format_junction(junction))
                }
                other => format!("{parent_str}{other:?}"),
            }
        }
        AssetId::Abstract(bytes) => format!("Abstract(0x{})", hex::encode(bytes)),
    }
}

fn format_multi_asset(multi_asset: &MultiAsset) -> String {
    let location = format_multi_location(multi_asset.id);
    let (asset_type, amount) = match multi_asset.fun {
        Fungibility::Fungible(amount) => ("Native", amount),
        Fungibility::NonFungible(_) => ("NFT", 0),
    };

    format!(
        "Location: {}, Asset: {}, Amount: {}",
        location,
        asset_type,
        format_amount(amount)
    )
}

impl XcmParser for Xcm<()> {
    fn parse_message(&self) -> Vec<XcmInstruction> {
        self.0
            .iter()
            .map(|instr| match instr {
                Instruction::WithdrawAsset(multi_asset) => {
                    let mut params = BTreeMap::new();
                    params.insert(
                        "Assets".to_string(),
                        multi_asset
                            .inner()
                            .iter()
                            .map(format_multi_asset)
                            .collect(),
                    );
                    XcmInstruction::new("WithdrawAsset", params)
                }
                other => raw_instruction(other),
            })
            .collect()
    }
}
