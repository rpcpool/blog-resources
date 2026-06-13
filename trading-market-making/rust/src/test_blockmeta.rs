use std::collections::HashMap;
use futures::StreamExt;
use tonic::transport::ClientTlsConfig;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
    SubscribeRequestFilterBlocksMeta,
};

const ENDPOINT: &str = "https://brianlo-brian-c9fc.mainnet.rpcpool.com";
const TOKEN: &str = "196d1ede-048d-478c-ba28-7de09ff21156";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Connecting to {}", ENDPOINT);

    let mut client = GeyserGrpcClient::build_from_shared(ENDPOINT.to_string())?
        .tls_config(ClientTlsConfig::new().with_enabled_roots())?
        .x_token(Some(TOKEN.to_string()))?
        .connect()
        .await?;

    let version = client.get_version().await?;
    println!("Node: {}", version.version);

    for (name, level) in &[
        ("Processed", CommitmentLevel::Processed),
        ("Confirmed", CommitmentLevel::Confirmed),
        ("Finalized", CommitmentLevel::Finalized),
    ] {
        let mut blocks_meta = HashMap::new();
        blocks_meta.insert("test".to_string(), SubscribeRequestFilterBlocksMeta {});

        let request = SubscribeRequest {
            blocks_meta,
            commitment: Some(*level as i32),
            ..Default::default()
        };

        println!("\nTesting commitment={} ...", name);
        let (_tx, mut stream) = client.subscribe_with_request(Some(request)).await?;

        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(20);
        let mut count = 0;
        loop {
            tokio::select! {
                msg = stream.next() => {
                    match msg {
                        Some(Ok(m)) => {
                            if let Some(UpdateOneof::BlockMeta(m)) = m.update_oneof {
                                count += 1;
                                println!("  slot={} blockhash={}", m.slot, m.blockhash);
                                if count >= 3 { break; }
                            }
                        }
                        _ => break,
                    }
                }
                _ = tokio::time::sleep_until(deadline) => { break; }
            }
        }
        if count == 0 {
            println!("  BROKEN — no updates in 20s");
        } else {
            println!("  OK — got {} updates", count);
        }
    }

    Ok(())
}
