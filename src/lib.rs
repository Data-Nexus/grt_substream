mod abi;
mod pb;
mod utils;

use crate::utils::helper::{append_0x, generate_id};
use abi::abi::grt::v1 as grt_events;

use pb::eth::grt::v1::{self as grt, Approvals};
use substreams::{log, Hex};
use substreams_entity_change::{pb::entity::EntityChanges, tables::Tables};
use substreams_ethereum::pb::eth;
use utils::constants::GRT_CONTRACT;

use substreams::errors::Error;

#[substreams::handlers::map]
fn map_transfer(block: eth::v2::Block) -> Result<grt::Transfers, Error> {
    Ok(grt::Transfers {
        transfers: block
            .events::<grt_events::events::Transfer>(&[&GRT_CONTRACT])
            .map(|(transfer, log)| {
                log::info!("GRT Transfer Event");

                grt::Transfer {
                    from: append_0x(&Hex(transfer.from).to_string()),
                    to: append_0x(&Hex(transfer.to).to_string()),
                    block_number: block.number,
                    timestamp: block.timestamp_seconds(),
                    amount: transfer.value.to_string(),
                    tx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                    log_index: log.index(),
                }
            })
            .collect(),
    })
}

#[substreams::handlers::map]
fn map_approval(block: eth::v2::Block) -> Result<grt::Approvals, Error> {
    Ok(grt::Approvals {
        approvals: block
            .events::<grt_events::events::Approval>(&[&GRT_CONTRACT])
            .map(|(approval, log)| {
                log::info!("GRT Approval Event");

                grt::Approval {
                    owner: append_0x(&Hex(approval.owner).to_string()),
                    spender: append_0x(&Hex(approval.spender).to_string()),
                    value: approval.value.to_string(),
                    block_number: block.number,
                    timestamp: block.timestamp_seconds(),
                    tx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                    log_index: log.index(),
                }
            })
            .collect(),
    })
}

#[substreams::handlers::map]
fn graph_out(transfers: grt::Transfers, approvals: grt::Approvals) -> Result<EntityChanges, Error> {
    let mut tables = Tables::new();

    for transfer in &transfers.transfers {
        let id: String = generate_id(&transfer.tx_hash, &transfer.log_index.to_string().as_str());

        let row = tables.create_row("Transfer", &id);
        row.set("timestamp", transfer.timestamp);
        row.set("blockNumber", transfer.block_number);
        row.set("logIndex", transfer.log_index);
        row.set("txHash", &transfer.tx_hash);
        row.set("amount", &transfer.amount);
        row.set("sender", &transfer.from);
        row.set("receiver", &transfer.to);
    }

    for approval in &approvals.approvals {
        let id: String = generate_id(&approval.tx_hash, &approval.log_index.to_string().as_str());

        let row = tables.create_row("Approval", &id);
        row.set("timestamp", approval.timestamp);
        row.set("blockNumber", approval.block_number);
        row.set("logIndex", approval.log_index);
        row.set("txHash", &approval.tx_hash);
        row.set("value", &approval.value);
        row.set("spender", &approval.spender);
        row.set("owner", &approval.owner);
    }
    let entity_changes = tables.to_entity_changes();
    log::info!("Entity changes: {:?}", entity_changes);
    Ok(entity_changes)
}
