import Client, {
  CommitmentLevel,
  SubscribeRequest,
} from "@triton-one/yellowstone-grpc";
import bs58 from "bs58";

const ENDPOINT = "https://your-endpoint.rpcpool.com";
const TOKEN = "your-token";

async function streamWalletUpdates(walletAddress: string) {
  const client = new Client(ENDPOINT, TOKEN, {});
  await client.connect();
  const stream = await client.subscribe();

  const request: SubscribeRequest = {
    accounts: {
      wallet: {
        account: [walletAddress],
        owner: [],
        filters: [],
      },
    },
    transactions: {},
    slots: {},
    transactionsStatus: {},
    entry: {},
    blocks: {},
    blocksMeta: {},
    commitment: CommitmentLevel.CONFIRMED,
    accountsDataSlice: [],
    ping: undefined,
  };

  stream.on("data", (data) => {
    if (data.account) {
      const pubkey = bs58.encode(data.account.account!.pubkey);
      const lamports = data.account.account!.lamports;
      const slot = data.account.slot;
      console.log(`Account updated: ${pubkey}`);
      console.log(`  New balance: ${Number(lamports) / 1e9} SOL`);
      console.log(`  Slot: ${slot}`);
    }
  });

  stream.on("error", (err) => console.error("Stream error:", err));

  await new Promise<void>((resolve, reject) => {
    stream.write(request, (err: Error | null) => {
      if (err) reject(err);
      else resolve();
    });
  });

  setInterval(() => {
    stream.write({ ping: { id: 1 } } as SubscribeRequest, () => {});
  }, 30_000);
}

const wallet = process.argv[2] ?? "YourWalletAddressHere";
streamWalletUpdates(wallet).catch(console.error);
