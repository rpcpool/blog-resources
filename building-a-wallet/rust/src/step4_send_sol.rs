use anyhow::anyhow;
use reqwest::Client as HttpClient;
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;

const ENDPOINT: &str = "https://your-endpoint.rpcpool.com";
const TOKEN: &str = "your-token";

#[derive(Deserialize)]
struct PrioritizationFee {
    slot: u64,
    #[serde(rename = "prioritizationFee")]
    prioritization_fee: u64,
}

#[derive(Deserialize)]
struct RpcResponse {
    result: Vec<PrioritizationFee>,
}

async fn get_priority_fee(accounts: &[&str], percentile: u32) -> anyhow::Result<u64> {
    let http = HttpClient::new();
    let url = format!("{}/{}", ENDPOINT, TOKEN);
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "method": "getRecentPrioritizationFees",
        "params": [accounts, { "percentile": percentile }]
    });
    let mut resp: RpcResponse = http.post(&url).json(&body).send().await?.json().await?;
    resp.result.sort_by(|a, b| b.slot.cmp(&a.slot));
    resp.result
        .first()
        .map(|f| f.prioritization_fee)
        .ok_or_else(|| anyhow!("no fee data"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Replace with a real funded keypair
    let sender = Keypair::new();
    let recipient_address = "RecipientAddressHere";
    let lamports: u64 = 1_000_000; // 0.001 SOL

    let client = RpcClient::new(format!("{}/{}", ENDPOINT, TOKEN));
    let recipient = Pubkey::from_str(recipient_address)?;

    let priority_fee = get_priority_fee(&[&sender.pubkey().to_string()], 9000).await?;
    let blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(200_000),
            ComputeBudgetInstruction::set_compute_unit_price(priority_fee),
            system_instruction::transfer(&sender.pubkey(), &recipient, lamports),
        ],
        Some(&sender.pubkey()),
        &[&sender],
        blockhash,
    );

    let config = RpcSendTransactionConfig {
        skip_preflight: true,
        max_retries: Some(0),
        ..Default::default()
    };

    let sig = client.send_transaction_with_config(&tx, config)?;
    println!("Sent: {}", sig);

    Ok(())
}
