use solana_client::rpc_client::RpcClient;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

const ENDPOINT: &str = "https://your-endpoint.rpcpool.com";
const TOKEN: &str = "your-token";
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

fn main() -> anyhow::Result<()> {
    let wallet_address = std::env::args().nth(1).unwrap_or_else(|| {
        "YourWalletAddressHere".to_string()
    });

    let client = RpcClient::new(format!("{}/{}", ENDPOINT, TOKEN));
    let pubkey = Pubkey::from_str(&wallet_address)?;

    // SOL balance
    let lamports = client.get_balance(&pubkey)?;
    println!("SOL: {:.9}", lamports as f64 / 1_000_000_000.0);

    // SPL token accounts
    let token_program = Pubkey::from_str(TOKEN_PROGRAM_ID)?;
    let accounts = client.get_token_accounts_by_owner(
        &pubkey,
        TokenAccountsFilter::ProgramId(token_program),
    )?;

    for account in &accounts {
        println!("Token account: {}", account.pubkey);
    }

    Ok(())
}
