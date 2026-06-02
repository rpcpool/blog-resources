use std::collections::HashMap;
use futures::StreamExt;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
    SubscribeRequestFilterAccounts,
};

const ENDPOINT: &str = "https://your-endpoint.rpcpool.com";
const TOKEN: &str = "your-token";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wallet_address = std::env::args().nth(1).unwrap_or_else(|| {
        "YourWalletAddressHere".to_string()
    });

    let mut client = GeyserGrpcClient::build_from_shared(ENDPOINT.to_string())?
        .x_token(Some(TOKEN.to_string()))?
        .connect()
        .await?;

    let mut accounts = HashMap::new();
    accounts.insert(
        "wallet".to_string(),
        SubscribeRequestFilterAccounts {
            account: vec![wallet_address],
            ..Default::default()
        },
    );

    let request = SubscribeRequest {
        accounts,
        commitment: Some(CommitmentLevel::Confirmed as i32),
        ..Default::default()
    };

    let (_tx, mut stream) = client.subscribe_with_request(Some(request)).await?;

    while let Some(Ok(msg)) = stream.next().await {
        if let Some(UpdateOneof::Account(acct)) = msg.update_oneof {
            let lamports = acct.account.as_ref().map(|a| a.lamports).unwrap_or(0);
            let pubkey = bs58::encode(
                acct.account.as_ref().map(|a| a.pubkey.as_slice()).unwrap_or(&[])
            ).into_string();
            println!("Account updated: {}", pubkey);
            println!("  New balance: {:.9} SOL", lamports as f64 / 1e9);
            println!("  Slot: {}", acct.slot);
        }
    }

    Ok(())
}
