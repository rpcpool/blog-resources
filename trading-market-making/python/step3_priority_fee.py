import asyncio
import httpx

ENDPOINT = "https://your-endpoint.rpcpool.com"
TOKEN = "your-token"
RAYDIUM_AMM = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"

async def get_priority_fee(
    endpoint: str,
    token: str,
    accounts: list[str],
    percentile: int,  # 1–10000, where 5000 = median, 9000 = 90th percentile
) -> int:
    async with httpx.AsyncClient() as client:
        response = await client.post(
            f"{endpoint}/{token}",
            json={
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getRecentPrioritizationFees",
                "params": [accounts, {"percentile": percentile}],
            },
        )
        fees = response.json()["result"]
        fees.sort(key=lambda x: x["slot"], reverse=True)
        return fees[0]["prioritizationFee"]

async def main():
    fee = await get_priority_fee(ENDPOINT, TOKEN, [RAYDIUM_AMM], 9000)
    print(f"Priority fee (90th pct): {fee} micro-lamports/CU")

asyncio.run(main())
