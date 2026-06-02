import asyncio
import grpc
from grpc import aio
import base58

from geyser_pb2 import (
    SubscribeDeshredRequest,
    SubscribeRequestFilterDeshredTransactions,
    SubscribeRequestPing,
)
from geyser_pb2_grpc import GeyserStub

ENDPOINT = "your-endpoint.rpcpool.com:443"
TOKEN = "your-token"
RAYDIUM_AMM = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"

async def stream_deshred():
    credentials = grpc.ssl_channel_credentials()
    async with aio.secure_channel(ENDPOINT, credentials) as channel:
        stub = GeyserStub(channel)
        metadata = (("x-token", TOKEN),)

        async def request_iterator():
            yield SubscribeDeshredRequest(
                deshred_transactions={
                    "dex": SubscribeRequestFilterDeshredTransactions(
                        vote=False,
                        account_include=[RAYDIUM_AMM],
                    )
                }
            )
            while True:
                await asyncio.sleep(30)
                yield SubscribeDeshredRequest(ping=SubscribeRequestPing(id=1))

        async for update in stub.SubscribeDeshred(request_iterator(), metadata=metadata):
            if update.HasField("deshred_transaction"):
                sig = base58.b58encode(
                    bytes(update.deshred_transaction.transaction.signature)
                ).decode()
                print(f"Deshred tx (pre-execution): {sig} slot: {update.deshred_transaction.slot}")

asyncio.run(stream_deshred())
