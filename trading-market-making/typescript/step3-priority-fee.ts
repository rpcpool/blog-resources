const ENDPOINT = "https://your-endpoint.rpcpool.com";
const TOKEN = "your-token";
const RAYDIUM_AMM = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

async function getPriorityFee(
  endpoint: string,
  token: string,
  accounts: string[],
  percentile: number // 1–10000, where 5000 = median, 9000 = 90th percentile
): Promise<number> {
  const response = await fetch(`${endpoint}/${token}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getRecentPrioritizationFees",
      params: [accounts, { percentile }],
    }),
  });

  const { result } = await response.json();
  const fees: { slot: number; prioritizationFee: number }[] = result;
  const latest = fees.sort((a, b) => b.slot - a.slot)[0];
  return latest.prioritizationFee; // micro-lamports per compute unit
}

// Example usage
const fee = await getPriorityFee(ENDPOINT, TOKEN, [RAYDIUM_AMM], 9000);
console.log("Priority fee (90th pct):", fee, "micro-lamports/CU");
