use parity_scale_codec::Encode;
use staging_xcm::VersionedXcm;

fn print_message(name: &str, version: &str, instructions: &[&str], encoded: Vec<u8>) {
    println!();
    println!("============================================================");
    println!("{name}");
    println!("============================================================");

    println!("Version:      {version}");
    println!("Instructions: {}", instructions.len());
    println!("Encoded size: {} bytes", encoded.len());

    println!();
    println!("Instructions:");

    for (index, instruction) in instructions.iter().enumerate() {
        println!("  [{index}] {instruction}");
    }

    println!();
    println!("SCALE:");
    println!("  0x{}", hex::encode(&encoded));

    println!();
    println!("Bytes:");

    for (index, byte) in encoded.iter().enumerate() {
        println!("  [{index:03}] 0x{byte:02x}");
    }

    println!();
}

mod v3 {
    use super::*;

    use parity_scale_codec::Encode;
    use staging_xcm::v3::{
        Instruction, Junction, Junctions, MultiAsset, MultiAssets, MultiLocation, WeightLimit,
        WildMultiAsset, Xcm,
    };

    pub fn generate() {
        basic_transfer();
    }

    pub fn basic_transfer() {
        // Example:
        //
        // WithdrawAsset
        // BuyExecution
        // DepositAsset
        let asset = MultiAsset::from((MultiLocation::here(), 1_000_000u128));
        let beneficiary = MultiLocation::new(
            0,
            Junctions::X1(Junction::AccountId32 {
                network: None,
                id: [1u8; 32],
            }),
        );

        let message = Xcm(vec![
            Instruction::WithdrawAsset::<()>(MultiAssets::from(asset.clone())),
            Instruction::BuyExecution {
                fees: asset,
                weight_limit: WeightLimit::Unlimited,
            },
            Instruction::DepositAsset {
                assets: WildMultiAsset::All.into(),
                beneficiary,
            },
        ]);
        let encoded = VersionedXcm::V3(message).encode();

        print_message(
            "v3_basic_asset_transfer",
            "V3",
            &["WithdrawAsset", "BuyExecution", "DepositAsset"],
            encoded,
        );
    }
}
fn main() {
    v3::generate();
}
