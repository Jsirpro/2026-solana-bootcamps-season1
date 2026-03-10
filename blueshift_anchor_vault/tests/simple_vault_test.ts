import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BlueshiftAnchorVault } from "../target/types/blueshift_anchor_vault";
import { LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";
import BN from "bn.js";

// Helper to set balance using Surfpool/Surfnet cheatcodes
async function setAccountBalance(
  connection: anchor.web3.Connection,
  pubkey: PublicKey,
  lamports: number
) {
  const surfnetCall = {
    jsonrpc: "2.0",
    id: 1,
    method: "surfnet_setAccount", // 恢复为之前文件使用的正确方法名
    params: [
      pubkey.toBase58(),
      {
        data: "",
        executable: false,
        lamports: lamports,
        owner: "11111111111111111111111111111111", // System Program
      },
    ],
  };

  console.log(`[Cheatcode] Sending request to: ${connection.rpcEndpoint}`);
  const response = await fetch(connection.rpcEndpoint, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(surfnetCall),
  });

  const result = await response.json();
  console.log("[Cheatcode] Response:", JSON.stringify(result));
}

describe("Simple Vault Test", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  // Note: Ensure your IDL is generated and workspace configured correctly
  const program = anchor.workspace
    .BlueshiftAnchorVault as Program<BlueshiftAnchorVault>;
  const signer = provider.wallet.publicKey;

  let vaultPda: PublicKey;

  before(async () => {
    const beforeBalance = await provider.connection.getBalance(signer);
    console.log(
      `[Before Cheatcode] Signer Balance: ${
        beforeBalance / LAMPORTS_PER_SOL
      } SOL`
    );

    // Use cheatcode to set signer balance to 10 SOL
    await setAccountBalance(provider.connection, signer, 10 * LAMPORTS_PER_SOL);

    const afterBalance = await provider.connection.getBalance(signer);
    console.log(
      `[After Cheatcode] Signer Balance: ${afterBalance / LAMPORTS_PER_SOL} SOL`
    );

    // Derive PDA
    [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), signer.toBuffer()],
      program.programId
    );
  });

  it("Deposits SOL into the vault", async () => {
    const amount = new BN(1_000_000);

    const vaultBalanceBefore = await provider.connection
      .getBalance(vaultPda)
      .catch(() => 0);
    console.log(
      `[Before Deposit] Vault Balance: ${
        vaultBalanceBefore / LAMPORTS_PER_SOL
      } SOL`
    );

    await program.methods
      .deposit(amount)
      .accounts({
        signer: signer,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const vaultInfo = await provider.connection.getAccountInfo(vaultPda);
    console.log(
      `[After Deposit] Vault Balance: ${
        vaultInfo!.lamports / LAMPORTS_PER_SOL
      } SOL`
    );

    assert.ok(vaultInfo && vaultInfo.lamports >= amount.toNumber());
  });
});
