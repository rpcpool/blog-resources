use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::transaction::Transaction;

const ENDPOINT: &str = "https://your-endpoint.rpcpool.com";
const TOKEN: &str = "your-token";

fn send_with_jet(
    transaction: &Transaction,
    endpoint: &str,
    token: &str,
) -> anyhow::Result<String> {
    let rpc_url = format!("{}/{}", endpoint, token);
    let client = RpcClient::new(rpc_url);

    let signature = client.send_transaction_with_config(
        transaction,
        RpcSendTransactionConfig {
            skip_preflight: true,
            max_retries: Some(0), // disable server-side retries; implement your own retry logic
            ..Default::default()
        },
    )?;

    println!("Sent: {}", signature);
    Ok(signature.to_string())
}

fn main() {
    println!("See send_with_jet() — wire up a Transaction and call it with your endpoint.");
    println!("ENDPOINT: {}", ENDPOINT);
    println!("TOKEN: {}", TOKEN);
}
