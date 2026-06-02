import {
  Connection,
  ComputeBudgetProgram,
  Transaction,
} from "@solana/web3.js";

const ENDPOINT = "https://your-endpoint.rpcpool.com";
const TOKEN = "your-token";

async function sendWithJet(
  serializedTransaction: Buffer,
  endpoint: string,
  token: string
): Promise<string> {
  // Use your Triton endpoint — Jet routing is handled automatically
  const connection = new Connection(`${endpoint}/${token}`, "confirmed");

  const signature = await connection.sendRawTransaction(
    serializedTransaction,
    {
      skipPreflight: true, // simulate separately via simulateTransaction() if needed
      maxRetries: 0,       // disable server-side retries; implement your own retry logic instead
    }
  );

  console.log("Sent:", signature);
  return signature;
}

function addPriorityFee(tx: Transaction, microLamportsPerCU: number): void {
  tx.add(
    ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: microLamportsPerCU,
    })
  );
}

// Confirm the transaction landed
async function confirmTransaction(
  signature: string,
  endpoint: string,
  token: string
): Promise<void> {
  const connection = new Connection(`${endpoint}/${token}`, "confirmed");
  const statuses = await connection.getSignatureStatuses([signature]);
  const status = statuses.value[0];

  if (
    status?.confirmationStatus === "confirmed" ||
    status?.confirmationStatus === "finalized"
  ) {
    console.log("Landed:", signature);
  } else if (status?.err) {
    console.error("Failed:", status.err);
    // Re-fetch a recent blockhash, re-sign, and resend
  }
}

export { sendWithJet, addPriorityFee, confirmTransaction };
