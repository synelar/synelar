import { Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js"
import * as fs from "fs"
import * as path from "path"

const RPC_URL = "https://api.devnet.solana.com"

async function deploy() {
  const connection = new Connection(RPC_URL, "confirmed")

  const walletPath = path.join(process.env.HOME || "", ".config/solana/id.json")
  const walletData = JSON.parse(fs.readFileSync(walletPath, "utf-8"))
  const wallet = Keypair.fromSecretKey(new Uint8Array(walletData))

  console.log("Deployer:", wallet.publicKey.toString())

  const balance = await connection.getBalance(wallet.publicKey)
  console.log("Balance:", balance / LAMPORTS_PER_SOL, "SOL")

  if (balance < LAMPORTS_PER_SOL) {
    console.log("Requesting airdrop...")
    const sig = await connection.requestAirdrop(wallet.publicKey, 2 * LAMPORTS_PER_SOL)
    await connection.confirmTransaction(sig)
    console.log("Airdrop confirmed")
  }

  console.log("Run 'anchor deploy' to deploy the program")
}

deploy().catch(console.error)
