import sys
from solana.rpc.api import Client
from solana.rpc.types import TokenAccountOpts
from solders.pubkey import Pubkey

ENDPOINT = "https://your-endpoint.rpcpool.com"
TOKEN = "your-token"
TOKEN_PROGRAM_ID = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"

def get_wallet_state(wallet_address: str):
    client = Client(f"{ENDPOINT}/{TOKEN}")
    pubkey = Pubkey.from_string(wallet_address)

    # SOL balance
    resp = client.get_balance(pubkey)
    print(f"SOL: {resp.value / 1_000_000_000:.9f}")

    # SPL token accounts
    opts = TokenAccountOpts(program_id=Pubkey.from_string(TOKEN_PROGRAM_ID))
    token_resp = client.get_token_accounts_by_owner_json_parsed(pubkey, opts)
    for account in token_resp.value:
        info = account.account.data.parsed["info"]
        mint = info["mint"]
        amount = info["tokenAmount"]["uiAmount"]
        print(f"Token {mint}: {amount}")

wallet = sys.argv[1] if len(sys.argv) > 1 else "YourWalletAddressHere"
get_wallet_state(wallet)
