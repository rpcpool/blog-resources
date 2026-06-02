import Client, {
  SubscribeRequestFilterDeshredTransactions,
} from "@triton-one/yellowstone-grpc";
import bs58 from "bs58";

const ENDPOINT = "https://your-endpoint.rpcpool.com";
const TOKEN = "your-token";
const RAYDIUM_AMM = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

async function streamDeshred() {
  const client = new Client(ENDPOINT, TOKEN, {});
  await client.connect();
  const stream = await client.subscribeDeshred();

  const request = {
    deshredTransactions: {
      dex: {
        vote: false,
        accountInclude: [RAYDIUM_AMM],
        accountExclude: [],
        accountRequired: [],
      } as SubscribeRequestFilterDeshredTransactions,
    },
  };

  stream.on("data", (data) => {
    if (data.deshredTransaction) {
      const sig = bs58.encode(
        data.deshredTransaction.transaction!.signature
      );
      console.log(
        "Deshred tx (pre-execution):",
        sig,
        "slot:",
        data.deshredTransaction.slot
      );
    }
  });

  stream.on("error", (err) => {
    console.error("Deshred stream error:", err);
  });

  await new Promise<void>((resolve, reject) => {
    stream.write(request, (err: Error | null) => {
      if (err) reject(err);
      else resolve();
    });
  });
}

streamDeshred().catch(console.error);
