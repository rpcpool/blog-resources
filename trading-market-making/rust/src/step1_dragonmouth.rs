use std::collections::HashMap;
use futures::StreamExt;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
    SubscribeRequestFilterAccounts, SubscribeRequestFilterTransactions,
};

const ENDPOINT: &str = "https://your-endpoint.rpcpool.com";
const TOKEN: &str = "your-token";
const RAYDIUM_AMM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = GeyserGrpcClient::build_from_shared(ENDPOINT.to_string())?
        .x_token(Some(TOKEN.to_string()))?
        .connect()
        .await?;

    let mut accounts = HashMap::new();
    accounts.insert(
        "pool_accounts".to_string(),
        SubscribeRequestFilterAccounts {
            owner: vec![RAYDIUM_AMM.to_string()],
            ..Default::default()
        },
    );

    let mut transactions = HashMap::new();
    transactions.insert(
        "dex_transactions".to_string(),
        SubscribeRequestFilterTransactions {
            vote: Some(false),
            failed: Some(false),
            account_include: vec![RAYDIUM_AMM.to_string()],
            ..Default::default()
        },
    );

    let request = SubscribeRequest {
        accounts,
        transactions,
        commitment: Some(CommitmentLevel::Processed as i32),
        ..Default::default()
    };

    let (_tx, mut stream) = client.subscribe_with_request(Some(request)).await?;

    while let Some(Ok(msg)) = stream.next().await {
        match msg.update_oneof {
            Some(UpdateOneof::Account(acct)) => {
                let pubkey = bs58::encode(acct.account.unwrap().pubkey).into_string();
                println!("Pool state updated: {pubkey}");
            }
            Some(UpdateOneof::Transaction(tx)) => {
                let sig = bs58::encode(tx.transaction.unwrap().signature).into_string();
                println!("Transaction processed: {sig}");
            }
            _ => {}
        }
    }

    Ok(())
}
