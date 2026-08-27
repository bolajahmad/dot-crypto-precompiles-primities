use std::collections::BTreeMap;

use staging_xcm::v5::{
    Asset, AssetFilter, AssetId, AssetTransferFilter, Fungibility, Hint, Instruction,
    InteriorLocation, Junction, Junctions, Location, MaybeErrorCode, NetworkId, OriginKind,
    QueryResponseInfo, Response, Weight, WeightLimit, WildAsset, WildFungibility, Xcm,
};

use crate::types::{XcmInstruction, XcmParser};
use crate::util::{format_amount, short_hex, summarize_hex};

fn format_network(network: &Option<NetworkId>) -> String {
    match network {
        None => "Any".to_string(),
        Some(NetworkId::Polkadot) => "Polkadot".to_string(),
        Some(NetworkId::Kusama) => "Kusama".to_string(),
        Some(NetworkId::BitcoinCore) => "BitcoinCore".to_string(),
        Some(NetworkId::BitcoinCash) => "BitcoinCash".to_string(),
        Some(NetworkId::PolkadotBulletin) => "PolkadotBulletin".to_string(),
        Some(NetworkId::Ethereum { chain_id }) => format!("Ethereum({chain_id})"),
        Some(other) => format!("{other:?}"),
    }
}

fn format_junction(junction: &Junction) -> String {
    match junction {
        Junction::Parachain(id) => format!("Parachain({id})"),
        Junction::AccountId32 { network, id } => {
            format!(
                "AccountId32({}, {})",
                format_network(network),
                short_hex(id)
            )
        }
        Junction::AccountIndex64 { network, index } => {
            format!("AccountIndex64({}, {index})", format_network(network))
        }
        Junction::AccountKey20 { network, key } => {
            format!(
                "AccountKey20({}, {})",
                format_network(network),
                short_hex(key)
            )
        }
        Junction::PalletInstance(id) => format!("PalletInstance({id})"),
        Junction::GeneralIndex(id) => format!("GeneralIndex({id})"),
        Junction::GeneralKey { length, data } => {
            format!(
                "GeneralKey(len={length}, data={})",
                short_hex(&data[..*length as usize])
            )
        }
        Junction::OnlyChild => "OnlyChild".to_string(),
        Junction::Plurality { id, part } => format!("Plurality({id:?}, {part:?})"),
        Junction::GlobalConsensus(network) => {
            format!("GlobalConsensus({})", format_network(&Some(*network)))
        }
    }
}

fn format_interior(interior: &InteriorLocation) -> String {
    match interior {
        Junctions::Here => "Here".to_string(),
        other => other
            .iter()
            .map(format_junction)
            .collect::<Vec<_>>()
            .join("/"),
    }
}

fn format_location(location: &Location) -> String {
    let parent_str = if location.parents > 0 {
        format!("parents: {}, ", location.parents)
    } else {
        String::new()
    };
    format!("{parent_str}{}", format_interior(&location.interior))
}

fn format_optional_location(location: &Option<Location>) -> String {
    match location {
        Some(loc) => format_location(loc),
        None => "None".to_string(),
    }
}

fn format_asset_id(id: &AssetId) -> String {
    format_location(&id.0)
}

fn format_fungibility(fun: &Fungibility) -> (&'static str, u128) {
    match fun {
        Fungibility::Fungible(amount) => ("Native", *amount),
        Fungibility::NonFungible(_) => ("NFT", 0),
    }
}

fn format_asset(asset: &Asset) -> String {
    let (asset_type, amount) = format_fungibility(&asset.fun);
    format!(
        "Location: {}, Asset: {}, Amount: {}",
        format_asset_id(&asset.id),
        asset_type,
        format_amount(amount)
    )
}

fn format_asset_rows(asset: &Asset) -> Vec<String> {
    let (asset_type, amount) = format_fungibility(&asset.fun);
    vec![
        format!("Location: {}", format_asset_id(&asset.id)),
        format!("Asset: {asset_type}"),
        format!("Amount: {}", format_amount(amount)),
    ]
}

fn format_assets<'a>(assets: impl IntoIterator<Item = &'a Asset>) -> Vec<String> {
    assets.into_iter().map(format_asset).collect()
}

fn format_wild_asset(wild: &WildAsset) -> String {
    match wild {
        WildAsset::All => "All".to_string(),
        WildAsset::AllCounted(count) => format!("AllCounted({count})"),
        WildAsset::AllOf { id, fun } => {
            let fun = match fun {
                WildFungibility::Fungible => "Fungible",
                WildFungibility::NonFungible => "NonFungible",
            };
            format!("AllOf(id={}, fun={fun})", format_asset_id(id))
        }
        WildAsset::AllOfCounted { id, fun, count } => {
            let fun = match fun {
                WildFungibility::Fungible => "Fungible",
                WildFungibility::NonFungible => "NonFungible",
            };
            format!(
                "AllOfCounted(id={}, fun={fun}, count={count})",
                format_asset_id(id)
            )
        }
    }
}

fn format_asset_filter(filter: &AssetFilter) -> String {
    match filter {
        AssetFilter::Definite(assets) => {
            let rows: Vec<String> = assets.inner().iter().map(format_asset).collect();
            format!("Definite([{}])", rows.join("; "))
        }
        AssetFilter::Wild(wild) => format!("Wild({})", format_wild_asset(wild)),
    }
}

fn format_asset_transfer_filter(filter: &AssetTransferFilter) -> String {
    match filter {
        AssetTransferFilter::Teleport(inner) => {
            format!("Teleport({})", format_asset_filter(inner))
        }
        AssetTransferFilter::ReserveDeposit(inner) => {
            format!("ReserveDeposit({})", format_asset_filter(inner))
        }
        AssetTransferFilter::ReserveWithdraw(inner) => {
            format!("ReserveWithdraw({})", format_asset_filter(inner))
        }
    }
}

fn format_weight(weight: &Weight) -> String {
    format!("ref_time={}, proof_size={}", weight.ref_time(), weight.proof_size())
}

fn format_weight_limit(limit: &WeightLimit) -> String {
    match limit {
        WeightLimit::Unlimited => "Unlimited".to_string(),
        WeightLimit::Limited(weight) => format!("Limited({})", format_weight(weight)),
    }
}

fn format_origin_kind(kind: &OriginKind) -> String {
    match kind {
        OriginKind::Native => "Native".to_string(),
        OriginKind::SovereignAccount => "SovereignAccount".to_string(),
        OriginKind::Xcm => "XCM".to_string(),
        OriginKind::Superuser => "Superuser".to_string(),
    }
}

fn format_query_response_info(info: &QueryResponseInfo) -> Vec<String> {
    vec![
        format!("Destination: {}", format_location(&info.destination)),
        format!("Query Id: {}", info.query_id),
        format!("Max Weight: {}", format_weight(&info.max_weight)),
    ]
}

fn format_maybe_error_code(code: &MaybeErrorCode) -> String {
    match code {
        MaybeErrorCode::Success => "Success".to_string(),
        MaybeErrorCode::Error(bytes) => format!("Error({})", summarize_hex(bytes, 34)),
        MaybeErrorCode::TruncatedError(bytes) => {
            return format!("TruncatedError({})", summarize_hex(bytes, 34));
        }
    }
}

fn format_response(response: &Response) -> String {
    match response {
        Response::Null => "Null".to_string(),
        Response::Assets(assets) => {
            let rows: Vec<String> = assets.inner().iter().map(format_asset).collect();
            return format!("Assets([{}])", rows.join("; "));
        }
        Response::ExecutionResult(result) => match result {
            None => "ExecutionResult(Ok)".to_string(),
            Some((index, error)) => {
                format!("ExecutionResult(instruction={index}, error={error:?})")
            }
        },
        Response::Version(version) => format!("Version({version})"),
        Response::PalletsInfo(info) => format!("PalletsInfo({} entries)", info.len()),
        Response::DispatchResult(code) => {
            format!("DispatchResult({})", format_maybe_error_code(code))
        }
    }
}

fn format_nested_xcm(xcm: &Xcm<()>) -> Vec<String> {
    xcm.parse_message()
        .into_iter()
        .enumerate()
        .map(|(index, instr)| format!("[{index}] {}", instr.name))
        .collect()
}

fn format_bytes_as_utf8_or_hex(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) if !s.is_empty() && s.chars().all(|c| !c.is_control()) => s.to_string(),
        _ => summarize_hex(bytes, 34),
    }
}

fn format_hint(hint: &Hint) -> String {
    match hint {
        Hint::AssetClaimer { location } => {
            format!("AssetClaimer({})", format_location(location))
        }
    }
}

fn with_params(name: &str, entries: &[(&str, Vec<String>)]) -> XcmInstruction {
    let mut params = BTreeMap::new();
    for (key, value) in entries {
        params.insert((*key).to_string(), value.clone());
    }
    XcmInstruction::new(name, params)
}

fn parse_instruction(instr: &Instruction<()>) -> XcmInstruction {
    match instr {
        Instruction::WithdrawAsset(assets) => with_params(
            "WithdrawAsset",
            &[("Assets", format_assets(assets.inner()))],
        ),
        Instruction::ReserveAssetDeposited(assets) => with_params(
            "ReserveAssetDeposited",
            &[("Assets", format_assets(assets.inner()))],
        ),
        Instruction::ReceiveTeleportedAsset(assets) => with_params(
            "ReceiveTeleportedAsset",
            &[("Assets", format_assets(assets.inner()))],
        ),
        Instruction::QueryResponse {
            query_id,
            response,
            max_weight,
            querier,
        } => with_params(
            "QueryResponse",
            &[
                ("Query Id", vec![query_id.to_string()]),
                ("Response", vec![format_response(response)]),
                ("Max Weight", vec![format_weight(max_weight)]),
                ("Querier", vec![format_optional_location(querier)]),
            ],
        ),
        Instruction::TransferAsset {
            assets,
            beneficiary,
        } => with_params(
            "TransferAsset",
            &[
                ("Assets", format_assets(assets.inner())),
                ("Beneficiary", vec![format_location(beneficiary)]),
            ],
        ),
        Instruction::TransferReserveAsset { assets, dest, xcm } => with_params(
            "TransferReserveAsset",
            &[
                ("Assets", format_assets(assets.inner())),
                ("Destination", vec![format_location(dest)]),
                ("Onward XCM", format_nested_xcm(xcm)),
            ],
        ),
        Instruction::Transact {
            origin_kind,
            fallback_max_weight,
            call,
        } => {
            let encoded_call = call.clone().into_encoded();
            let weight = match fallback_max_weight {
                Some(w) => format_weight(w),
                None => "None".to_string(),
            };
            with_params(
                "Transact",
                &[
                    ("Origin Kind", vec![format_origin_kind(origin_kind)]),
                    ("Maximum Fallback Weight", vec![weight]),
                    ("Call Payload", vec![summarize_hex(&encoded_call, 34)]),
                ],
            )
        }
        Instruction::HrmpNewChannelOpenRequest {
            sender,
            max_message_size,
            max_capacity,
        } => with_params(
            "HrmpNewChannelOpenRequest",
            &[
                ("Sender", vec![sender.to_string()]),
                ("Max Message Size", vec![max_message_size.to_string()]),
                ("Max Capacity", vec![max_capacity.to_string()]),
            ],
        ),
        Instruction::HrmpChannelAccepted { recipient } => with_params(
            "HrmpChannelAccepted",
            &[("Recipient", vec![recipient.to_string()])],
        ),
        Instruction::HrmpChannelClosing {
            initiator,
            sender,
            recipient,
        } => with_params(
            "HrmpChannelClosing",
            &[
                ("Initiator", vec![initiator.to_string()]),
                ("Sender", vec![sender.to_string()]),
                ("Recipient", vec![recipient.to_string()]),
            ],
        ),
        Instruction::ClearOrigin => XcmInstruction::note(
            "ClearOrigin",
            "Clears the origin register so later instructions cannot use origin authority.",
        ),
        Instruction::DescendOrigin(interior) => with_params(
            "DescendOrigin",
            &[("Interior", vec![format_interior(interior)])],
        ),
        Instruction::ReportError(info) => {
            with_params("ReportError", &[("Response Info", format_query_response_info(info))])
        }
        Instruction::DepositAsset {
            assets,
            beneficiary,
        } => with_params(
            "DepositAsset",
            &[
                ("Assets", vec![format_asset_filter(assets)]),
                ("Beneficiary", vec![format_location(beneficiary)]),
            ],
        ),
        Instruction::DepositReserveAsset { assets, dest, xcm } => with_params(
            "DepositReserveAsset",
            &[
                ("Assets", vec![format_asset_filter(assets)]),
                ("Destination", vec![format_location(dest)]),
                ("Onward XCM", format_nested_xcm(xcm)),
            ],
        ),
        Instruction::ExchangeAsset {
            give,
            want,
            maximal,
        } => with_params(
            "ExchangeAsset",
            &[
                ("Give", vec![format_asset_filter(give)]),
                ("Want", format_assets(want.inner())),
                ("Maximal", vec![maximal.to_string()]),
            ],
        ),
        Instruction::InitiateReserveWithdraw {
            assets,
            reserve,
            xcm,
        } => with_params(
            "InitiateReserveWithdraw",
            &[
                ("Assets", vec![format_asset_filter(assets)]),
                ("Reserve", vec![format_location(reserve)]),
                ("Onward XCM", format_nested_xcm(xcm)),
            ],
        ),
        Instruction::InitiateTeleport { assets, dest, xcm } => with_params(
            "InitiateTeleport",
            &[
                ("Assets", vec![format_asset_filter(assets)]),
                ("Destination", vec![format_location(dest)]),
                ("Onward XCM", format_nested_xcm(xcm)),
            ],
        ),
        Instruction::ReportHolding {
            response_info,
            assets,
        } => with_params(
            "ReportHolding",
            &[
                ("Response Info", format_query_response_info(response_info)),
                ("Assets", vec![format_asset_filter(assets)]),
            ],
        ),
        Instruction::BuyExecution { fees, weight_limit } => with_params(
            "BuyExecution",
            &[
                ("Fees", format_asset_rows(fees)),
                ("Weight Limit", vec![format_weight_limit(weight_limit)]),
            ],
        ),
        Instruction::RefundSurplus => XcmInstruction::note(
            "RefundSurplus",
            "Calculates unspent execution fees dynamically at runtime and moves surplus weight from the holding register to the asset register.",
        ),
        Instruction::SetErrorHandler(xcm) => {
            with_params("SetErrorHandler", &[("Handler XCM", format_nested_xcm(xcm))])
        }
        Instruction::SetAppendix(xcm) => {
            with_params("SetAppendix", &[("Appendix XCM", format_nested_xcm(xcm))])
        }
        Instruction::ClearError => {
            XcmInstruction::note("ClearError", "Clears the error register.")
        }
        Instruction::ClaimAsset { assets, ticket } => with_params(
            "ClaimAsset",
            &[
                ("Assets", format_assets(assets.inner())),
                ("Ticket", vec![format_location(ticket)]),
            ],
        ),
        Instruction::Trap(code) => {
            with_params("Trap", &[("Code", vec![code.to_string()])])
        }
        Instruction::SubscribeVersion {
            query_id,
            max_response_weight,
        } => with_params(
            "SubscribeVersion",
            &[
                ("Query Id", vec![query_id.to_string()]),
                (
                    "Max Response Weight",
                    vec![format_weight(max_response_weight)],
                ),
            ],
        ),
        Instruction::UnsubscribeVersion => XcmInstruction::note(
            "UnsubscribeVersion",
            "Cancels a previous SubscribeVersion request.",
        ),
        Instruction::BurnAsset(assets) => {
            with_params("BurnAsset", &[("Assets", format_assets(assets.inner()))])
        }
        Instruction::ExpectAsset(assets) => {
            with_params("ExpectAsset", &[("Assets", format_assets(assets.inner()))])
        }
        Instruction::ExpectOrigin(origin) => with_params(
            "ExpectOrigin",
            &[("Origin", vec![format_optional_location(origin)])],
        ),
        Instruction::ExpectError(error) => {
            let value = match error {
                None => "None (no error expected)".to_string(),
                Some((index, err)) => format!("instruction={index}, error={err:?}"),
            };
            with_params("ExpectError", &[("Expected", vec![value])])
        }
        Instruction::ExpectTransactStatus(status) => with_params(
            "ExpectTransactStatus",
            &[("Status", vec![format_maybe_error_code(status)])],
        ),
        Instruction::QueryPallet {
            module_name,
            response_info,
        } => with_params(
            "QueryPallet",
            &[
                ("Module Name", vec![format_bytes_as_utf8_or_hex(module_name)]),
                ("Response Info", format_query_response_info(response_info)),
            ],
        ),
        Instruction::ExpectPallet {
            index,
            name,
            module_name,
            crate_major,
            min_crate_minor,
        } => with_params(
            "ExpectPallet",
            &[
                ("Index", vec![index.to_string()]),
                ("Name", vec![format_bytes_as_utf8_or_hex(name)]),
                ("Module Name", vec![format_bytes_as_utf8_or_hex(module_name)]),
                ("Crate Major", vec![crate_major.to_string()]),
                ("Min Crate Minor", vec![min_crate_minor.to_string()]),
            ],
        ),
        Instruction::ReportTransactStatus(info) => with_params(
            "ReportTransactStatus",
            &[("Response Info", format_query_response_info(info))],
        ),
        Instruction::ClearTransactStatus => XcmInstruction::note(
            "ClearTransactStatus",
            "Clears the Transact Status Register.",
        ),
        Instruction::UniversalOrigin(junction) => with_params(
            "UniversalOrigin",
            &[("Junction", vec![format_junction(junction)])],
        ),
        Instruction::ExportMessage {
            network,
            destination,
            xcm,
        } => with_params(
            "ExportMessage",
            &[
                ("Network", vec![format_network(&Some(*network))]),
                ("Destination", vec![format_interior(destination)]),
                ("Exported XCM", format_nested_xcm(xcm)),
            ],
        ),
        Instruction::LockAsset { asset, unlocker } => with_params(
            "LockAsset",
            &[
                ("Asset", format_asset_rows(asset)),
                ("Unlocker", vec![format_location(unlocker)]),
            ],
        ),
        Instruction::UnlockAsset { asset, target } => with_params(
            "UnlockAsset",
            &[
                ("Asset", format_asset_rows(asset)),
                ("Target", vec![format_location(target)]),
            ],
        ),
        Instruction::NoteUnlockable { asset, owner } => with_params(
            "NoteUnlockable",
            &[
                ("Asset", format_asset_rows(asset)),
                ("Owner", vec![format_location(owner)]),
            ],
        ),
        Instruction::RequestUnlock { asset, locker } => with_params(
            "RequestUnlock",
            &[
                ("Asset", format_asset_rows(asset)),
                ("Locker", vec![format_location(locker)]),
            ],
        ),
        Instruction::SetFeesMode { jit_withdraw } => with_params(
            "SetFeesMode",
            &[("JIT Withdraw", vec![jit_withdraw.to_string()])],
        ),
        Instruction::SetTopic(topic) => {
            with_params("SetTopic", &[("Topic", vec![format!("0x{}", hex::encode(topic))])])
        }
        Instruction::ClearTopic => {
            XcmInstruction::note("ClearTopic", "Clears the topic register.")
        }
        Instruction::AliasOrigin(location) => with_params(
            "AliasOrigin",
            &[("Origin", vec![format_location(location)])],
        ),
        Instruction::UnpaidExecution {
            weight_limit,
            check_origin,
        } => with_params(
            "UnpaidExecution",
            &[
                ("Weight Limit", vec![format_weight_limit(weight_limit)]),
                ("Check Origin", vec![format_optional_location(check_origin)]),
            ],
        ),
        Instruction::PayFees { asset } => {
            with_params("PayFees", &[("Asset", format_asset_rows(asset))])
        }
        Instruction::InitiateTransfer {
            destination,
            remote_fees,
            preserve_origin,
            assets,
            remote_xcm,
        } => {
            let fees = match remote_fees {
                Some(filter) => format_asset_transfer_filter(filter),
                None => "None (UnpaidExecution)".to_string(),
            };
            let asset_filters: Vec<String> = assets
                .iter()
                .map(format_asset_transfer_filter)
                .collect();
            with_params(
                "InitiateTransfer",
                &[
                    ("Destination", vec![format_location(destination)]),
                    ("Remote Fees", vec![fees]),
                    ("Preserve Origin", vec![preserve_origin.to_string()]),
                    ("Assets", asset_filters),
                    ("Remote XCM", format_nested_xcm(remote_xcm)),
                ],
            )
        }
        Instruction::ExecuteWithOrigin {
            descendant_origin,
            xcm,
        } => {
            let origin = match descendant_origin {
                Some(interior) => format_interior(interior),
                None => "None".to_string(),
            };
            with_params(
                "ExecuteWithOrigin",
                &[
                    ("Descendant Origin", vec![origin]),
                    ("Inner XCM", format_nested_xcm(xcm)),
                ],
            )
        }
        Instruction::SetHints { hints } => {
            let rows: Vec<String> = hints.iter().map(format_hint).collect();
            with_params("SetHints", &[("Hints", rows)])
        }
    }
}

impl XcmParser for Xcm<()> {
    fn parse_message(&self) -> Vec<XcmInstruction> {
        self.0.iter().map(parse_instruction).collect()
    }
}
