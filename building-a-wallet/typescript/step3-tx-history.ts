import {
  Connection,
  PublicKey,
  ParsedTransactionWithMeta,
} from "@solana/web3.js";

const ENDPOINT = "https://your-endpoint.rpcpool.com";
const TOKEN = "your-token";

async function getTransactionHistory(
  walletAddress: string,
  limit: number = 10
): Promise<ParsedTransactionWithMeta[]> {
  const connection = new Connection(`${ENDPOINT}/${TOKEN}`);
  const pubkey = new PublicKey(walletAddress);

  const signatures = await connection.getSignaturesForAddress(pubkey, { limit });
  console.log(`Found ${signatures.length} transactions`);

  const transactions = await connection.getParsedTransactions(
    signatures.map((s) => s.signature),
    { maxSupportedTransactionVersion: 0 }
  );

  for (const tx of transactions) {
    if (!tx) continue;
    const sig = tx.transaction.signatures[0];
    const slot = tx.slot;
    const status = tx.meta?.err ? "failed" : "ok";
    const fee = tx.meta?.fee ?? 0;
    console.log(`${sig.slice(0, 20)}... slot=${slot} status=${status} fee=${fee}`);
  }

  return transactions.filter(Boolean) as ParsedTransactionWithMeta[];
}

const wallet = process.argv[2] ?? "YourWalletAddressHere";
getTransactionHistory(wallet).catch(console.error);
