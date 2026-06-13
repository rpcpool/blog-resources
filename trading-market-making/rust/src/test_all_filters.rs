use std::collections::HashMap;
use futures::StreamExt;
use tonic::transport::ClientTlsConfig;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
    SubscribeRequestFilterAccounts, SubscribeRequestFilterBlocks,
    SubscribeRequestFilterBlocksMeta, SubscribeRequestFilterSlots,
    SubscribeRequestFilterTransactions,
};

const ENDPOINT: &str = "http://pit180.rpcpool.wg:10000";
const TOKEN: &str = "";
const TIMEOUT_SECS: u64 = 20;

fn is_expected_update(update: &UpdateOneof, filter: &str) -> bool {
    match filter {
        "slots"         => matches!(update, UpdateOneof::Slot(_)),
        "blocksMeta"    => matches!(update, UpdateOneof::BlockMeta(_)),
        "blocks"        => matches!(update, UpdateOneof::Block(_)),
        "transactions"  => matches!(update, UpdateOneof::Transaction(_)),
        "accounts"      => matches!(update, UpdateOneof::Account(_)),
        _               => false,
    }
}

async fn test_subscription(
    client: &mut GeyserGrpcClient,
    label: &str,
    filter_type: &str,
    request: SubscribeRequest,
) -> bool {
    print!("  {:<35} ", label);
    let (_tx, mut stream) = match client.subscribe_with_request(Some(request)).await {
        Ok(s) => s,
        Err(e) => { println!("ERROR: {}", e); return false; }
    };

    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(TIMEOUT_SECS);
    loop {
        tokio::select! {
            msg = stream.next() => {
                match msg {
                    Some(Ok(m)) => {
                        if let Some(update) = &m.update_oneof {
                            if is_expected_update(update, filter_type) {
                                println!("OK");
                                return true;
                            }
                            // ignore pings/pongs/other — keep waiting
                        }
                    }
                    _ => { println!("BROKEN (stream closed)"); return false; }
                }
            }
            _ = tokio::time::sleep_until(deadline) => {
                println!("BROKEN (no data in {}s)", TIMEOUT_SECS);
                return false;
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Connecting to {}", ENDPOINT);
    let builder = GeyserGrpcClient::build_from_shared(ENDPOINT.to_string())?
        .x_token(if TOKEN.is_empty() { None } else { Some(TOKEN.to_string()) })?;
    let mut client = if ENDPOINT.starts_with("https") {
        builder.tls_config(ClientTlsConfig::new().with_enabled_roots())?
    } else {
        builder
    }
    .connect()
    .await?;

    let version = client.get_version().await?;
    println!("Node: {}\n", version.version);

    let commitments = [
        ("Processed", CommitmentLevel::Processed),
        ("Confirmed", CommitmentLevel::Confirmed),
        ("Finalized", CommitmentLevel::Finalized),
    ];

    println!("=== Slots ===");
    for (name, level) in &commitments {
        let mut slots = HashMap::new();
        slots.insert("test".to_string(), SubscribeRequestFilterSlots {
            filter_by_commitment: Some(true),
            interslot_updates: Some(false),
        });
        test_subscription(&mut client, &format!("slots/{}", name), "slots", SubscribeRequest {
            slots, commitment: Some(*level as i32), ..Default::default()
        }).await;
    }

    println!("\n=== BlocksMeta ===");
    for (name, level) in &commitments {
        let mut blocks_meta = HashMap::new();
        blocks_meta.insert("test".to_string(), SubscribeRequestFilterBlocksMeta {});
        test_subscription(&mut client, &format!("blocksMeta/{}", name), "blocksMeta", SubscribeRequest {
            blocks_meta, commitment: Some(*level as i32), ..Default::default()
        }).await;
    }

    println!("\n=== Blocks ===");
    for (name, level) in &commitments {
        let mut blocks = HashMap::new();
        blocks.insert("test".to_string(), SubscribeRequestFilterBlocks {
            include_transactions: Some(false),
            include_accounts: Some(false),
            include_entries: Some(false),
            ..Default::default()
        });
        test_subscription(&mut client, &format!("blocks/{}", name), "blocks", SubscribeRequest {
            blocks, commitment: Some(*level as i32), ..Default::default()
        }).await;
    }

    println!("\n=== Transactions ===");
    let mut transactions = HashMap::new();
    transactions.insert("test".to_string(), SubscribeRequestFilterTransactions {
        vote: Some(false), failed: Some(false), ..Default::default()
    });
    test_subscription(&mut client, "transactions/Processed", "transactions", SubscribeRequest {
        transactions, commitment: Some(CommitmentLevel::Processed as i32), ..Default::default()
    }).await;

    println!("\n=== Accounts ===");
    let mut accounts = HashMap::new();
    accounts.insert("test".to_string(), SubscribeRequestFilterAccounts { ..Default::default() });
    test_subscription(&mut client, "accounts/Processed", "accounts", SubscribeRequest {
        accounts, commitment: Some(CommitmentLevel::Processed as i32), ..Default::default()
    }).await;

    println!("\nDone.");
    Ok(())
}
