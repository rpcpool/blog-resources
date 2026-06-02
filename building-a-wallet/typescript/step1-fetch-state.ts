import {
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

const ENDPOINT = "https://your-endpoint.rpcpool.com";
const TOKEN = "your-token";

async function getWalletState(walletAddress: string) {
  const connection = new Connection(`${ENDPOINT}/${TOKEN}`);
  const pubkey = new PublicKey(walletAddress);

  // SOL balance
  const lamports = await connection.getBalance(pubkey);
  console.log(`SOL: ${lamports / LAMPORTS_PER_SOL}`);

  // All SPL token accounts owned by this wallet
  const { value: tokenAccounts } = await connection.getParsedTokenAccountsByOwner(
    pubkey,
    { programId: TOKEN_PROGRAM_ID }
  );

  for (const { account } of tokenAccounts) {
    const info = account.data.parsed.info;
    const mint: string = info.mint;
    const amount: number = info.tokenAmount.uiAmount;
    console.log(`Token ${mint}: ${amount}`);
  }
}

const wallet = process.argv[2] ?? "YourWalletAddressHere";
getWalletState(wallet).catch(console.error);
