use std::collections::BTreeMap;

use staging_xcm::v4::{Asset, AssetId, Fungibility, Instruction, Junction, Junctions, Xcm};

use crate::types::{XcmInstruction, XcmParser};
use crate::util::{format_amount, raw_instruction, short_hex};

fn format_junction(junction: &Junction) -> String {
    match junction {
        Junction::AccountId32 { id, .. } => format!("AccountId32: {}", short_hex(id)),
        Junction::Parachain(id) => format!("Parachain({id})"),
        other => format!("{other:?}"),
    }
}

fn format_interior(interior: &Junctions) -> String {
    match interior {
        Junctions::Here => "Here".to_string(),
        other => other
            .iter()
            .map(format_junction)
            .collect::<Vec<_>>()
            .join("/"),
    }
}

fn format_location(aid: &AssetId) -> String {
    let parent_str = if aid.0.parents > 0 {
        format!("parents: {}, ", aid.0.parents)
    } else {
        String::new()
    };
    format!("{parent_str}{}", format_interior(&aid.0.interior))
}

fn format_asset(asset: &Asset) -> String {
    let location = format_location(&asset.id);
    let (asset_type, amount) = match asset.fun {
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
        self.inner()
            .iter()
            .map(|instr| match instr {
                Instruction::WithdrawAsset(assets) => {
                    let mut params = BTreeMap::new();
                    params.insert(
                        "Assets".to_string(),
                        assets.inner().iter().map(format_asset).collect(),
                    );
                    XcmInstruction::new("WithdrawAsset", params)
                }
                other => raw_instruction(other),
            })
            .collect()
    }
}
