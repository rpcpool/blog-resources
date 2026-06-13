use std::collections::HashMap;
use futures::StreamExt;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update_deshred, SubscribeDeshredRequest, SubscribeRequestFilterDeshredTransactions,
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

    let mut deshred_transactions = HashMap::new();
    deshred_transactions.insert(
        "dex".to_string(),
        SubscribeRequestFilterDeshredTransactions {
            vote: Some(false),
            account_include: vec![RAYDIUM_AMM.to_string()],
            ..Default::default()
        },
    );

    let request = SubscribeDeshredRequest {
        deshred_transactions,
        ..Default::default()
    };

    let (_sink, mut stream) = client.subscribe_deshred_with_request(Some(request)).await?;

    while let Some(Ok(msg)) = stream.next().await {
        if let Some(subscribe_update_deshred::UpdateOneof::DeshredTransaction(deshred_tx)) =
            msg.update_oneof
        {
            let sig = bs58::encode(deshred_tx.transaction.unwrap().signature).into_string();
            println!("Deshred tx (pre-execution): {} slot: {}", sig, deshred_tx.slot);
        }
    }

    Ok(())
}
