use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSignaturesForAddressConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

const ENDPOINT: &str = "https://your-endpoint.rpcpool.com";
const TOKEN: &str = "your-token";

fn main() -> anyhow::Result<()> {
    let wallet_address = std::env::args().nth(1).unwrap_or_else(|| {
        "YourWalletAddressHere".to_string()
    });

    let client = RpcClient::new(format!("{}/{}", ENDPOINT, TOKEN));
    let pubkey = Pubkey::from_str(&wallet_address)?;

    let config = RpcSignaturesForAddressConfig {
        limit: Some(10),
        commitment: Some(CommitmentConfig::finalized()),
        ..Default::default()
    };

    let sigs = client.get_signatures_for_address_with_config(&pubkey, config)?;
    println!("Found {} transactions", sigs.len());

    for sig_info in &sigs {
        let status = if sig_info.err.is_some() { "failed" } else { "ok" };
        println!(
            "{:.20}... slot={} status={}",
            sig_info.signature,
            sig_info.slot.unwrap_or(0),
            status
        );
    }

    Ok(())
}
