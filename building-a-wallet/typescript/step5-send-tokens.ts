import {
  Connection,
  PublicKey,
  Transaction,
  ComputeBudgetProgram,
  Keypair,
} from "@solana/web3.js";
import {
  getOrCreateAssociatedTokenAccount,
  createTransferCheckedInstruction,
  getMint,
  getAssociatedTokenAddress,
  getAccount,
} from "@solana/spl-token";

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

async function sendToken(
  sender: Keypair,
  recipientAddress: string,
  mintAddress: string,
  amount: number // display units, e.g. 1.5 for 1.5 USDC
): Promise<string> {
  const connection = new Connection(`${ENDPOINT}/${TOKEN}`);
  const recipient = new PublicKey(recipientAddress);
  const mint = new PublicKey(mintAddress);

  const mintInfo = await getMint(connection, mint);
  const atomicAmount = BigInt(Math.round(amount * 10 ** mintInfo.decimals));

  const senderAta = await getOrCreateAssociatedTokenAccount(
    connection, sender, mint, sender.publicKey
  );
  const recipientAta = await getOrCreateAssociatedTokenAccount(
    connection, sender, mint, recipient
  );

  const priorityFee = await getPriorityFee([sender.publicKey.toBase58()], 9000);

  const { blockhash } = await connection.getLatestBlockhash();
  const tx = new Transaction();
  tx.recentBlockhash = blockhash;
  tx.feePayer = sender.publicKey;

  tx.add(
    ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 }),
    ComputeBudgetProgram.setComputeUnitPrice({ microLamports: priorityFee }),
    createTransferCheckedInstruction(
      senderAta.address,
      mint,
      recipientAta.address,
      sender.publicKey,
      atomicAmount,
      mintInfo.decimals
    )
  );

  tx.sign(sender);

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

// Example usage — replace with a real funded keypair, recipient, and mint
// const sender = Keypair.fromSecretKey(Uint8Array.from([...]));
// sendToken(sender, "RecipientAddressHere", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", 1.0).catch(console.error);
