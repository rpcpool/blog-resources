# spl.token.instructions is included in `pip install solana` — no separate package needed.
import asyncio
import httpx
from solana.rpc.api import Client
from solana.rpc.types import TxOpts
from solders.keypair import Keypair
from solders.pubkey import Pubkey
from solders.transaction import Transaction as SolTransaction
from solders.message import Message
from solders.compute_budget import set_compute_unit_price, set_compute_unit_limit
from spl.token.instructions import transfer_checked, TransferCheckedParams
from spl.token.constants import TOKEN_PROGRAM_ID

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

def send_token(
    sender: Keypair,
    source_ata: Pubkey,
    dest_ata: Pubkey,
    mint: Pubkey,
    amount: int,      # atomic units
    decimals: int,
    priority_fee: int,
) -> str:
    client = Client(f"{ENDPOINT}/{TOKEN}")
    blockhash = client.get_latest_blockhash().value.blockhash

    instructions = [
        set_compute_unit_limit(300_000),
        set_compute_unit_price(priority_fee),
        transfer_checked(TransferCheckedParams(
            program_id=TOKEN_PROGRAM_ID,
            source=source_ata,
            mint=mint,
            dest=dest_ata,
            owner=sender.pubkey(),
            amount=amount,
            decimals=decimals,
            signers=[],
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

# Example usage — replace with real values
# async def main():
#     sender = Keypair()  # replace with real keypair
#     priority_fee = await get_priority_fee([str(sender.pubkey())], 9000)
#     send_token(
#         sender,
#         source_ata=Pubkey.from_string("SourceATAAddressHere"),
#         dest_ata=Pubkey.from_string("DestinationATAAddressHere"),
#         mint=Pubkey.from_string("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),  # USDC
#         amount=1_000_000,  # 1 USDC
#         decimals=6,
#         priority_fee=priority_fee,
#     )
# asyncio.run(main())
