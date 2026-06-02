import sys
from solana.rpc.api import Client
from solders.pubkey import Pubkey

ENDPOINT = "https://your-endpoint.rpcpool.com"
TOKEN = "your-token"

def get_transaction_history(wallet_address: str, limit: int = 10):
    client = Client(f"{ENDPOINT}/{TOKEN}")
    pubkey = Pubkey.from_string(wallet_address)

    sigs = client.get_signatures_for_address(pubkey, limit=limit)
    print(f"Found {len(sigs.value)} transactions")

    for sig_info in sigs.value:
        status = "failed" if sig_info.err else "ok"
        print(f"{str(sig_info.signature)[:20]}... slot={sig_info.slot} status={status}")

wallet = sys.argv[1] if len(sys.argv) > 1 else "YourWalletAddressHere"
get_transaction_history(wallet)
