import asyncio
import grpc
from grpc import aio
import base58

# Requires yellowstone gRPC Python stubs generated from the proto files.
# python -m grpc_tools.protoc -I./proto --python_out=. --grpc_python_out=. geyser.proto
# Proto files: https://github.com/rpcpool/yellowstone-grpc/tree/master/yellowstone-grpc-proto/proto
from geyser_pb2 import (
    SubscribeRequest,
    SubscribeRequestFilterAccounts,
    SubscribeRequestFilterTransactions,
    CommitmentLevel,
    SubscribeRequestPing,
)
from geyser_pb2_grpc import GeyserStub

ENDPOINT = "your-endpoint.rpcpool.com:443"
TOKEN = "your-token"
RAYDIUM_AMM = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"

async def stream_dex_data():
    credentials = grpc.ssl_channel_credentials()
    async with aio.secure_channel(ENDPOINT, credentials) as channel:
        stub = GeyserStub(channel)
        metadata = (("x-token", TOKEN),)

        async def request_iterator():
            yield SubscribeRequest(
                accounts={
                    "pool_accounts": SubscribeRequestFilterAccounts(
                        owner=[RAYDIUM_AMM],
                    )
                },
                transactions={
                    "dex_transactions": SubscribeRequestFilterTransactions(
                        vote=False,
                        failed=False,
                        account_include=[RAYDIUM_AMM],
                    )
                },
                commitment=CommitmentLevel.PROCESSED,
            )
            while True:
                await asyncio.sleep(30)
                yield SubscribeRequest(ping=SubscribeRequestPing(id=1))

        async for update in stub.Subscribe(request_iterator(), metadata=metadata):
            if update.HasField("account"):
                pubkey = base58.b58encode(bytes(update.account.account.pubkey)).decode()
                print(f"Pool state updated: {pubkey}")
            elif update.HasField("transaction"):
                sig = base58.b58encode(bytes(update.transaction.transaction.signature)).decode()
                print(f"Transaction processed: {sig}")

asyncio.run(stream_dex_data())
