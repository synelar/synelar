import { Connection, Keypair, PublicKey, Transaction, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js"
import * as fs from "fs"
import * as path from "path"

const RPC_URL = "https://api.devnet.solana.com"
const PROGRAM_ID = new PublicKey("SYNiD1111111111111111111111111111111111111")

async function initialize() {
  const connection = new Connection(RPC_URL, "confirmed")

  const walletPath = path.join(process.env.HOME || "", ".config/solana/id.json")
  const walletData = JSON.parse(fs.readFileSync(walletPath, "utf-8"))
  const wallet = Keypair.fromSecretKey(new Uint8Array(walletData))

  console.log("Authority:", wallet.publicKey.toString())

  const [configPDA] = PublicKey.findProgramAddressSync([Buffer.from("config")], PROGRAM_ID)

  console.log("Config PDA:", configPDA.toString())

  const existingConfig = await connection.getAccountInfo(configPDA)
  if (existingConfig) {
    console.log("Config already initialized")
    return
  }

  const mintPrice = 0
  const accessFee = 0.0005 * LAMPORTS_PER_SOL

  const discriminator = Buffer.from([175, 175, 109, 31, 13, 152, 155, 237])
  const mintPriceBuffer = Buffer.alloc(8)
  mintPriceBuffer.writeBigUInt64LE(BigInt(mintPrice))
  const accessFeeBuffer = Buffer.alloc(8)
  accessFeeBuffer.writeBigUInt64LE(BigInt(accessFee))

  const data = Buffer.concat([discriminator, mintPriceBuffer, accessFeeBuffer])

  const ix = {
    programId: PROGRAM_ID,
    keys: [
      { pubkey: configPDA, isSigner: false, isWritable: true },
      { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
      { pubkey: wallet.publicKey, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  }

  const tx = new Transaction().add(ix)
  tx.feePayer = wallet.publicKey
  const { blockhash } = await connection.getLatestBlockhash()
  tx.recentBlockhash = blockhash

  tx.sign(wallet)

  const sig = await connection.sendRawTransaction(tx.serialize())
  await connection.confirmTransaction(sig)

  console.log("Initialized:", sig)
}

initialize().catch(console.error)
