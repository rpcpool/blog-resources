import Client, {
  CommitmentLevel,
  SubscribeRequest,
} from "@triton-one/yellowstone-grpc";
import bs58 from "bs58";

const ENDPOINT = "https://your-endpoint.rpcpool.com";
const TOKEN = "your-token";

// Raydium AMM v4 program — replace with the DEX program you're trading
const RAYDIUM_AMM = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

async function streamDexData() {
  const client = new Client(ENDPOINT, TOKEN, {});
  await client.connect();
  const stream = await client.subscribe();

  const request: SubscribeRequest = {
    accounts: {
      pool_accounts: {
        account: [],
        owner: [RAYDIUM_AMM],
        filters: [],
      },
    },
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
  };

  stream.on("data", (data) => {
    if (data.account) {
      const pubkey = bs58.encode(data.account.account!.pubkey);
      console.log("Pool state updated:", pubkey);
    }

    if (data.transaction) {
      const sig = bs58.encode(data.transaction.transaction!.signature);
      console.log("Transaction processed:", sig);
    }
  });

  stream.on("error", (err) => {
    console.error("Stream error:", err);
  });

  await new Promise<void>((resolve, reject) => {
    stream.write(request, (err: Error | null) => {
      if (err) reject(err);
      else resolve();
    });
  });

  // Keep stream alive with periodic pings
  setInterval(() => {
    stream.write({ ping: { id: 1 } } as SubscribeRequest, () => {});
  }, 30_000);
}

streamDexData().catch(console.error);
