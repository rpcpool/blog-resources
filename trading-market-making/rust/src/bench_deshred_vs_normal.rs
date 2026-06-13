// Benchmark: deshred vs normal subscribe on the same endpoint.
// Matches transactions by signature and compares recv_ms.
// delta = deshred_recv_ms - normal_recv_ms
// Negative = deshred arrived earlier (correct pre-execution behavior).
// Positive = deshred arrived later (regression).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use futures::StreamExt;
use tonic::transport::ClientTlsConfig;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof,
    subscribe_update_deshred,
    CommitmentLevel,
    SubscribeDeshredRequest,
    SubscribeRequest,
    SubscribeRequestFilterDeshredTransactions,
    SubscribeRequestFilterTransactions,
};

const ENDPOINT: &str = "https://waynele-mainnet-ed26.mainnet.rpcpool.com";
const TOKEN: &str = "416f3036-fc85-4778-b17c-66fd83eacdc4";
const PUMP_FUN: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const TARGET_MATCHES: usize = 30;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

async fn build_client() -> anyhow::Result<GeyserGrpcClient> {
    Ok(GeyserGrpcClient::build_from_shared(ENDPOINT.to_string())?
        .tls_config(ClientTlsConfig::new().with_enabled_roots())?
        .x_token(Some(TOKEN.to_string()))?
        .connect()
        .await?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let deshred_map: Arc<Mutex<HashMap<String, i64>>> = Arc::new(Mutex::new(HashMap::new()));
    let normal_map: Arc<Mutex<HashMap<String, i64>>> = Arc::new(Mutex::new(HashMap::new()));

    // Task 1: deshred stream
    let deshred_map_t = deshred_map.clone();
    tokio::spawn(async move {
        let mut client = build_client().await.expect("deshred client");
        let mut deshred_transactions = HashMap::new();
        deshred_transactions.insert(
            "bench".to_string(),
            SubscribeRequestFilterDeshredTransactions {
                vote: Some(false),
                account_required: vec![PUMP_FUN.to_string()],
                ..Default::default()
            },
        );
        let req = SubscribeDeshredRequest { deshred_transactions, ..Default::default() };
        let (_sink, mut stream) = client.subscribe_deshred_with_request(Some(req)).await.expect("deshred stream");

        while let Some(Ok(msg)) = stream.next().await {
            if let Some(subscribe_update_deshred::UpdateOneof::DeshredTransaction(tx)) = msg.update_oneof {
                let recv_ms = now_ms();
                if let Some(info) = tx.transaction {
                    let sig = bs58::encode(&info.signature).into_string();
                    deshred_map_t.lock().unwrap().insert(sig, recv_ms);
                }
            }
        }
    });

    // Task 2: normal transaction stream
    let normal_map_t = normal_map.clone();
    tokio::spawn(async move {
        let mut client = build_client().await.expect("normal client");
        let mut transactions = HashMap::new();
        transactions.insert(
            "bench".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                account_include: vec![PUMP_FUN.to_string()],
                ..Default::default()
            },
        );
        let req = SubscribeRequest {
            transactions,
            commitment: Some(CommitmentLevel::Processed as i32),
            ..Default::default()
        };
        let (_sink, mut stream) = client.subscribe_with_request(Some(req)).await.expect("normal stream");

        while let Some(Ok(msg)) = stream.next().await {
            if let Some(UpdateOneof::Transaction(tx)) = msg.update_oneof {
                let recv_ms = now_ms();
                if let Some(info) = tx.transaction {
                    let sig = bs58::encode(&info.signature).into_string();
                    normal_map_t.lock().unwrap().insert(sig, recv_ms);
                }
            }
        }
    });

    println!("Both streams started. Waiting for {} matched signatures...", TARGET_MATCHES);

    // Poll for matches
    let mut deltas: Vec<(String, i64)> = Vec::new();
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let d = deshred_map.lock().unwrap();
        let n = normal_map.lock().unwrap();

        deltas.clear();
        for (sig, d_ms) in d.iter() {
            if let Some(n_ms) = n.get(sig) {
                deltas.push((sig.clone(), d_ms - n_ms));
            }
        }

        println!("  {}/{} matches...", deltas.len(), TARGET_MATCHES);

        if deltas.len() >= TARGET_MATCHES {
            break;
        }
    }

    // Print results
    let mut values: Vec<i64> = deltas.iter().map(|(_, d)| *d).collect();
    values.sort();
    let count = values.len();
    let neg = values.iter().filter(|&&d| d < 0).count();
    let avg = values.iter().sum::<i64>() / count as i64;
    let p50 = values[count / 2];
    let p90 = values[(count * 9 / 10).min(count - 1)];

    println!("\n--- Deshred vs Normal Subscribe ({} matched sigs) ---", count);
    println!("delta = deshred_recv_ms - normal_recv_ms");
    println!("negative = deshred arrived EARLIER (expected)");
    println!("positive = deshred arrived LATER (regression)");
    println!();
    println!("early={}/{} ({:.0}%)", neg, count, neg as f64 / count as f64 * 100.0);
    println!("min={}  max={}  avg={}", values[0], values[count - 1], avg);
    println!("p50={}  p90={}", p50, p90);

    if neg as f64 / count as f64 > 0.5 {
        println!("\n[NORMAL] Deshred arriving earlier than normal subscribe on majority of samples.");
    } else {
        println!("\n[REGRESSION] Deshred NOT arriving earlier — behaving like a normal subscribe stream.");
    }

    Ok(())
}
