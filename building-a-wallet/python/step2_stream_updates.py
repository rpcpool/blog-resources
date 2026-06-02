import asyncio
import sys
import grpc
from grpc import aio
import base58

from geyser_pb2 import (
    SubscribeRequest,
    SubscribeRequestFilterAccounts,
    CommitmentLevel,
    SubscribeRequestPing,
)
from geyser_pb2_grpc import GeyserStub

ENDPOINT = "your-endpoint.rpcpool.com:443"
TOKEN = "your-token"

async def stream_wallet_updates(wallet_address: str):
    credentials = grpc.ssl_channel_credentials()
    async with aio.secure_channel(ENDPOINT, credentials) as channel:
        stub = GeyserStub(channel)
        metadata = (("x-token", TOKEN),)

        async def request_iterator():
            yield SubscribeRequest(
                accounts={
                    "wallet": SubscribeRequestFilterAccounts(
                        account=[wallet_address],
                    )
                },
                commitment=CommitmentLevel.CONFIRMED,
            )
            while True:
                await asyncio.sleep(30)
                yield SubscribeRequest(ping=SubscribeRequestPing(id=1))

        async for update in stub.Subscribe(request_iterator(), metadata=metadata):
            if update.HasField("account"):
                pubkey = base58.b58encode(bytes(update.account.account.pubkey)).decode()
                lamports = update.account.account.lamports
                slot = update.account.slot
                print(f"Account updated: {pubkey}")
                print(f"  New balance: {lamports / 1e9:.9f} SOL")
                print(f"  Slot: {slot}")

wallet = sys.argv[1] if len(sys.argv) > 1 else "YourWalletAddressHere"
asyncio.run(stream_wallet_updates(wallet))
