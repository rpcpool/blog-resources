import asyncio
import httpx
from solana.rpc.api import Client
from solana.rpc.types import TxOpts
from solders.keypair import Keypair
from solders.pubkey import Pubkey
from solders.system_program import TransferParams, transfer
from solders.transaction import Transaction as SolTransaction
from solders.message import Message
from solders.compute_budget import set_compute_unit_price, set_compute_unit_limit

ENDPOINT = "https://your-endpoint.rpcpool.com"
TOKEN = "your-token"

async def get_priority_fee(accounts: list[str], percentile: int) -> int:
    async with httpx.AsyncClient() as http:
        resp = await http.post(
            f"{ENDPOINT}/{TOKEN}",
            json={
                "jsonrpc": "2.0", "id": 1,
                "method": "getRecentPrioritizationFees",
                "params": [accounts, {"percentile": percentile}],
            },
        )
        fees = resp.json()["result"]
        fees.sort(key=lambda x: x["slot"], reverse=True)
        return fees[0]["prioritizationFee"]

def send_sol(sender: Keypair, recipient_address: str, lamports: int, priority_fee: int) -> str:
    client = Client(f"{ENDPOINT}/{TOKEN}")
    recipient = Pubkey.from_string(recipient_address)

    blockhash = client.get_latest_blockhash().value.blockhash

    instructions = [
        set_compute_unit_limit(200_000),
        set_compute_unit_price(priority_fee),
        transfer(TransferParams(
            from_pubkey=sender.pubkey(),
            to_pubkey=recipient,
            lamports=lamports,
        )),
    ]

    msg = Message.new_with_blockhash(instructions, sender.pubkey(), blockhash)
    tx = SolTransaction([sender], msg, blockhash)

    response = client.send_raw_transaction(
        bytes(tx),
        opts=TxOpts(skip_preflight=True, max_retries=0),
    )
    print(f"Sent: {response.value}")
    return str(response.value)

# Example usage — replace with a real funded keypair and recipient
# async def main():
#     sender = Keypair()  # replace with real keypair
#     priority_fee = await get_priority_fee([str(sender.pubkey())], 9000)
#     send_sol(sender, "RecipientAddressHere", 1_000_000, priority_fee)
# asyncio.run(main())
