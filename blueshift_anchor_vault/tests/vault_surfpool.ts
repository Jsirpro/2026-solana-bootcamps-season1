import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

describe("vault test (surfpool)", () => {

  /**
   * Anchor Provider
   * provider 里面包含：
   * - connection (RPC)
   * - wallet
   */
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  /**
   * Program instance
   * 从 target/idl 自动读取 IDL
   */
  const program = anchor.workspace.BlueshiftAnchorVault;

  /**
   * 当前测试 signer
   */
  const signer = provider.wallet;

  /**
   * vault PDA
   */
  let vaultPda: PublicKey;

  /**
   * 在所有测试之前执行
   */
  before(async () => {

    /**
     * ============================
     * Surfpool Cheatcode
     * ============================
     *
     * 类似 Foundry:
     *
     * vm.deal(address, amount)
     *
     * 这里直接修改账户 lamports
     */

    const richBalance = 10_000_000_000; // 10 SOL

    await provider.connection._rpcRequest("surfpool_set_lamports", [
      signer.publicKey.toBase58(),
      richBalance
    ]);

    /**
     * 再确认余额
     */
    const balance =
      await provider.connection.getBalance(signer.publicKey);

    console.log("Signer balance:", balance);

    assert.ok(balance >= richBalance);

    /**
     * 推导 vault PDA
     */
    [vaultPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault"),
        signer.publicKey.toBuffer()
      ],
      program.programId
    );

  });

  /**
   * deposit 测试
   */
  it("deposit", async () => {

    const amount = new BN(1_000_000);

    await program.methods
      .deposit(amount)
      .accounts({
        signer: signer.publicKey,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const vaultAccount =
      await provider.connection.getAccountInfo(vaultPda);

    assert.ok(vaultAccount !== null);
    assert.ok(vaultAccount!.lamports > 0);

  });

  /**
   * withdraw 测试
   */
  it("withdraw", async () => {

    await program.methods
      .withdraw()
      .accounts({
        signer: signer.publicKey,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const vaultAccount =
      await provider.connection.getAccountInfo(vaultPda);

    /**
     * vault lamports 被转走后
     * 账户可能被 runtime 清理
     */
    if (vaultAccount === null) {

      assert.ok(true);

    } else {

      assert.equal(vaultAccount.lamports, 0);

    }

  });

});