// Note: assumes the source and destination ATAs already exist.
// Use spl-associated-token-account to derive or create them if needed.
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::instruction::transfer_checked;
use std::str::FromStr;

const ENDPOINT: &str = "https://your-endpoint.rpcpool.com";
const TOKEN: &str = "your-token";

fn main() -> anyhow::Result<()> {
    let sender = Keypair::new(); // replace with real funded keypair
    let source_ata = Pubkey::from_str("SourceATAAddressHere")?;
    let dest_ata = Pubkey::from_str("DestinationATAAddressHere")?;
    let mint = Pubkey::from_str("MintAddressHere")?;
    let amount: u64 = 1_000_000; // atomic units (e.g. 1 USDC = 1_000_000)
    let decimals: u8 = 6;
    let priority_fee: u64 = 10_000; // replace with Priority Fee API result

    let client = RpcClient::new(format!("{}/{}", ENDPOINT, TOKEN));
    let blockhash = client.get_latest_blockhash()?;

    let transfer_ix = transfer_checked(
        &spl_token::id(),
        &source_ata,
        &mint,
        &dest_ata,
        &sender.pubkey(),
        &[],
        amount,
        decimals,
    )?;

    let tx = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(300_000),
            ComputeBudgetInstruction::set_compute_unit_price(priority_fee),
            transfer_ix,
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
