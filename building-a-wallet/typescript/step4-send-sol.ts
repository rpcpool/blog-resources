import {
  Connection,
  PublicKey,
  SystemProgram,
  Transaction,
  ComputeBudgetProgram,
  Keypair,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";

const ENDPOINT = "https://your-endpoint.rpcpool.com";
const TOKEN = "your-token";

async function getPriorityFee(accounts: string[], percentile: number): Promise<number> {
  const response = await fetch(`${ENDPOINT}/${TOKEN}`, {
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

async function sendSol(
  sender: Keypair,
  recipientAddress: string,
  amountSol: number
): Promise<string> {
  const connection = new Connection(`${ENDPOINT}/${TOKEN}`);
  const recipient = new PublicKey(recipientAddress);
  const lamports = Math.round(amountSol * LAMPORTS_PER_SOL);

  const priorityFee = await getPriorityFee([sender.publicKey.toBase58()], 9000);

  const { blockhash } = await connection.getLatestBlockhash();
  const tx = new Transaction();
  tx.recentBlockhash = blockhash;
  tx.feePayer = sender.publicKey;

  tx.add(
    ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }),
    ComputeBudgetProgram.setComputeUnitPrice({ microLamports: priorityFee }),
    SystemProgram.transfer({
      fromPubkey: sender.publicKey,
      toPubkey: recipient,
      lamports,
    })
  );

  tx.sign(sender);

  // Send via Jet — skipPreflight + maxRetries: 0 for direct leader delivery
  const sendResponse = await fetch(`${ENDPOINT}/${TOKEN}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "sendTransaction",
      params: [
        tx.serialize().toString("base64"),
        { encoding: "base64", skipPreflight: true, maxRetries: 0 },
      ],
    }),
  });

  const { result: signature } = await sendResponse.json();
  console.log("Sent:", signature);
  return signature;
}

// Example usage — replace with a real funded keypair and recipient
// const sender = Keypair.fromSecretKey(Uint8Array.from([...]));
// sendSol(sender, "RecipientAddressHere", 0.001).catch(console.error);
