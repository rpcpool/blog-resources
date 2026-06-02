from solana.rpc.api import Client
from solana.rpc.types import TxOpts

ENDPOINT = "https://your-endpoint.rpcpool.com"
TOKEN = "your-token"

def send_with_jet(serialized_tx: bytes, endpoint: str, token: str) -> str:
    client = Client(f"{endpoint}/{token}")
    response = client.send_raw_transaction(
        serialized_tx,
        opts=TxOpts(skip_preflight=True, max_retries=0),
    )
    signature = str(response.value)
    print(f"Sent: {signature}")
    return signature
