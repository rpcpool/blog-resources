use reqwest::Client;
use serde::Deserialize;

const ENDPOINT: &str = "https://your-endpoint.rpcpool.com";
const TOKEN: &str = "your-token";
const RAYDIUM_AMM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

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

async fn get_priority_fee(
    endpoint: &str,
    token: &str,
    accounts: &[&str],
    percentile: u32, // 1–10000, where 5000 = median, 9000 = 90th percentile
) -> anyhow::Result<u64> {
    let client = Client::new();
    let url = format!("{}/{}", endpoint, token);

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getRecentPrioritizationFees",
        "params": [accounts, { "percentile": percentile }]
    });

    let mut response: RpcResponse = client
        .post(&url)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    response.result.sort_by(|a, b| b.slot.cmp(&a.slot));
    Ok(response.result[0].prioritization_fee)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fee = get_priority_fee(ENDPOINT, TOKEN, &[RAYDIUM_AMM], 9000).await?;
    println!("Priority fee (90th pct): {} micro-lamports/CU", fee);
    Ok(())
}
