import Client, { CommitmentLevel } from "@triton-one/yellowstone-grpc";
import { Connection, Transaction, ComputeBudgetProgram } from "@solana/web3.js";

const ENDPOINT = "https://your-endpoint.rpcpool.com";
const TOKEN = "your-token";
const RAYDIUM_AMM = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

async function getPriorityFee(
  endpoint: string,
  token: string,
  accounts: string[],
  percentile: number
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
  return fees.sort((a, b) => b.slot - a.slot)[0].prioritizationFee;
}

function analyseTransaction(tx: any): boolean {
  // Your signal logic here
  return true;
}

async function tradingLoop() {
  const connection = new Connection(`${ENDPOINT}/${TOKEN}`, "confirmed");
  const client = new Client(ENDPOINT, TOKEN, {});
  await client.connect();
  const stream = await client.subscribe();

  stream.on("data", async (data) => {
    if (!data.transaction) return;

    // 1. Signal detected — decide if you want to act
    const shouldTrade = analyseTransaction(data.transaction);
    if (!shouldTrade) return;

    // 2. Get competitive priority fee
    const fee = await getPriorityFee(ENDPOINT, TOKEN, [RAYDIUM_AMM], 9000);

    // 3. Build your transaction
    const tx = new Transaction();
    tx.add(ComputeBudgetProgram.setComputeUnitPrice({ microLamports: fee }));
    // ... add your swap instruction here

    // 4. Send via Jet
    const sig = await connection.sendRawTransaction(tx.serialize(), {
      skipPreflight: true,
      maxRetries: 0,
    });

    console.log("Sent:", sig);
  });

  await new Promise<void>((resolve, reject) => {
    stream.write(
      {
        accounts: {},
        transactions: {
          dex_transactions: {
            vote: false,
            failed: false,
            accountInclude: [RAYDIUM_AMM],
            accountExclude: [],
            accountRequired: [],
          },
        },
        slots: {},
        transactionsStatus: {},
        entry: {},
        blocks: {},
        blocksMeta: {},
        commitment: CommitmentLevel.PROCESSED,
        accountsDataSlice: [],
        ping: undefined,
      },
      (err: Error | null) => {
        if (err) reject(err);
        else resolve();
      }
    );
  });

  // Keep stream alive with periodic pings
  setInterval(() => {
    stream.write({ ping: { id: 1 } } as any, () => {});
  }, 30_000);
}

tradingLoop().catch(console.error);
