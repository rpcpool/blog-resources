// Reproduction script for idlebug (waynele) deshred latency regression.
// Measures recv_ms - block_time_ms for each deshred update.
// Healthy pre-execution stream: majority negative (~-200ms).
// Regression indicator: all positive, p50 > 500ms.

use std::collections::HashMap;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use futures::StreamExt;
use tonic::transport::ClientTlsConfig;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update_deshred, SubscribeDeshredRequest, SubscribeRequestFilterDeshredTransactions,
};

const DEFAULT_ENDPOINT: &str = "https://waynele-mainnet-ed26.mainnet.rpcpool.com";
const DEFAULT_TOKEN: &str = "416f3036-fc85-4778-b17c-66fd83eacdc4";
const DEFAULT_RPC_URL: &str = "https://waynele-mainnet-ed26.mainnet.rpcpool.com/416f3036-fc85-4778-b17c-66fd83eacdc4";
const PUMP_FUN: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const SAMPLE_COUNT: usize = 30;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

async fn get_block_time_ms(rpc_url: &str, slot: u64) -> anyhow::Result<Option<i64>> {
    let client = reqwest::Client::new();
    let resp: serde_json::Value = client
        .post(rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBlockTime",
            "params": [slot]
        }))
        .send()
        .await?
        .json()
        .await?;
    Ok(resp["result"].as_i64().map(|t| t * 1000))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Usage: cargo run --bin reproduce_deshred_latency -- <grpc_endpoint> <token> [rpc_url]
    // Internal node: cargo run --bin reproduce_deshred_latency -- http://ams356.rpcpool.wg:10000 "" https://waynele-mainnet-ed26.mainnet.rpcpool.com/416f3036-fc85-4778-b17c-66fd83eacdc4
    let args: Vec<String> = env::args().collect();
    let endpoint = args.get(1).map(|s| s.as_str()).unwrap_or(DEFAULT_ENDPOINT);
    let token = args.get(2).map(|s| s.as_str()).unwrap_or(DEFAULT_TOKEN);
    let rpc_url = args.get(3).map(|s| s.as_str()).unwrap_or(DEFAULT_RPC_URL).to_string();

    println!("Endpoint: {}", endpoint);
    println!("RPC URL:  {}", rpc_url);

    let builder = GeyserGrpcClient::build_from_shared(endpoint.to_string())?
        .x_token(if token.is_empty() { None } else { Some(token.to_string()) })?;
    let mut client = if endpoint.starts_with("https") {
        builder.tls_config(ClientTlsConfig::new().with_enabled_roots())?
    } else {
        builder
    }
    .connect()
    .await?;

    let mut deshred_transactions = HashMap::new();
    deshred_transactions.insert(
        "reproduce".to_string(),
        SubscribeRequestFilterDeshredTransactions {
            vote: Some(false),
            account_required: vec![PUMP_FUN.to_string()],
            ..Default::default()
        },
    );

    let request = SubscribeDeshredRequest {
        deshred_transactions,
        ..Default::default()
    };

    let (_sink, mut stream) = client.subscribe_deshred_with_request(Some(request)).await?;

    // Phase 1: record (slot, recv_ms) at actual receipt time — no sleep in the loop
    let mut samples: Vec<(u64, i64)> = Vec::new();

    println!("Connected. Collecting {} samples (pump.fun filter)...", SAMPLE_COUNT);

    while let Some(Ok(msg)) = stream.next().await {
        if let Some(subscribe_update_deshred::UpdateOneof::DeshredTransaction(deshred_tx)) =
            msg.update_oneof
        {
            samples.push((deshred_tx.slot, now_ms()));
            if samples.len() >= SAMPLE_COUNT {
                break;
            }
        }
    }

    println!("Got {} samples. Waiting 5s then looking up block_times...", samples.len());
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Phase 2: batch block_time lookups
    let mut deltas: Vec<i64> = Vec::new();
    println!("{:>8}  {:>12}  {:>12}  {:>10}", "#", "slot", "recv_ms", "delta_ms");

    for (i, (slot, recv_ms)) in samples.iter().enumerate() {
        match get_block_time_ms(&rpc_url, *slot).await {
            Ok(Some(block_time_ms)) => {
                let delta = recv_ms - block_time_ms;
                println!("{:>8}  {:>12}  {:>12}  {:>+10}", i + 1, slot, recv_ms, delta);
                deltas.push(delta);
            }
            Ok(None) => eprintln!("slot {} has no block_time yet", slot),
            Err(e) => eprintln!("getBlockTime error for slot {}: {}", slot, e),
        }
    }

    if deltas.is_empty() {
        println!("No samples collected.");
        return Ok(());
    }

    deltas.sort();
    let count = deltas.len();
    let neg = deltas.iter().filter(|&&d| d < 0).count();
    let avg = deltas.iter().sum::<i64>() / count as i64;
    let p50 = deltas[count / 2];
    let p90 = deltas[(count * 9 / 10).min(count - 1)];
    let p99 = deltas[(count * 99 / 100).min(count - 1)];

    println!("\n--- Results ({} samples) ---", count);
    println!("negative={}/{} ({:.0}% pre-execution)", neg, count, neg as f64 / count as f64 * 100.0);
    println!("min={}  max={}  avg={}", deltas[0], deltas[count - 1], avg);
    println!("p50={}  p90={}  p99={}", p50, p90, p99);

    if neg == 0 && p50 > 500 {
        println!("\n[REGRESSION CONFIRMED] 0 negative samples, p50={}ms — stream is post-confirmation, not pre-shred.", p50);
    } else if neg as f64 / count as f64 > 0.5 {
        println!("\n[NORMAL] Majority negative — stream delivering pre-execution as expected.");
    } else {
        println!("\n[DEGRADED] Some negative but p50={}ms positive — partial regression or noisy.", p50);
    }

    Ok(())
}
