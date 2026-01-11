import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js"
import SynidSDK from "../sdk"

const TEST_CONFIG = {
  rpcUrl: "https://api.devnet.solana.com",
}

async function runTests() {
  const sdk = new SynidSDK(TEST_CONFIG)

  const testWallet = Keypair.generate()
  console.log("Test wallet:", testWallet.publicKey.toString())

  const [configPDA] = await sdk.getConfigPDA()
  console.log("Config PDA:", configPDA.toString())

  const [synidPDA] = await sdk.getSynidPDA(testWallet.publicKey)
  console.log("SynID PDA:", synidPDA.toString())

  const [mintAuthority] = await sdk.getMintAuthorityPDA()
  console.log("Mint Authority PDA:", mintAuthority.toString())

  const [escrow] = await sdk.getEscrowPDA()
  console.log("Escrow PDA:", escrow.toString())

  const balance = await sdk.getBalance(testWallet.publicKey)
  console.log("Balance:", balance / LAMPORTS_PER_SOL, "SOL")

  console.log("All PDA derivations successful")
}

runTests().catch(console.error)
