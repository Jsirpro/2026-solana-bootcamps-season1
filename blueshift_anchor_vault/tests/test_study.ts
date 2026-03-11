import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
import { PublicKey, SystemProgram,LAMPORTS_PER_SOL, Connection} from "@solana/web3.js";
import { assert } from "chai";
import PoolStats from "undici/types/pool-stats";
import { connect } from "undici";

async function setAccountBalance(
    connection : anchor.web3.Connection,
    pubkey : PublicKey,
    lamports : number
) {
    const SurfnetCall = {
        jsonrpc : "2.0",
        id : 1,
        method : "surfnet_setAccount",
        params:[
            pubkey.toBase58(),
            {
                data:"",
                executable:false,
                lamports:lamports,
                owner:"11111111111111111111111111111111"
            }
        ],
    };
    /**fecth(url,options(可选)) 用于发送HTTP请求*/
    await fetch(
        connection.rpcEndpoint,{
            method:"POST",
            headers:{"Content-Type": "application/json"},
            //JSON.stringify() 的意思是：把 JavaScript 对象/数组等数据，转换成 JSON 格式的字符串。
            body:JSON.stringify(SurfnetCall),
        }
    );
}

describe("test start",() =>{
    const provider = anchor.AnchorProvider.env();

    anchor.setProvider(provider);

    const connection = provider.connection;
    const wallet = provider.wallet;

    const program = anchor.workspace.BlueshiftAnchorVault;

    const signer = provider.wallet.publicKey;

    let vaultPda : PublicKey;

    //在测试开始之前派生PDA账户，初始化signer余额
    before(async() => {
        //初始化signer余额
        const beforeBalance = await provider.connection.getBalance(signer);
        console.log("before cheatcode balance:\n",beforeBalance / LAMPORTS_PER_SOL);
        
        await setAccountBalance(connection,signer,10 * LAMPORTS_PER_SOL);

        const afterBalance = await provider.connection.getBalance(signer);
        console.log("after cheatcode balance:\n",afterBalance / LAMPORTS_PER_SOL);

        //派生PDA账户
        [vaultPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("vault"),signer.toBuffer()],
            program.programId
        );
    });

    //测试deposit函数
    it("Deposit function test",async() =>{
        //用 BN(大整数库) 是为了和链上整数类型对齐并保证精度安全。
        const amount = new BN(1_000_000);

        //获取金库PDA初始余额
        const vaultBalanceBefore = await connection.getBalance(vaultPda)
        //.catch(() => 0) 是 Promise（对于异步操作的期望） 的错误处理，如果报错，则返回0
        .catch(() => 0
        );
        console.log("before deposit:",vaultBalanceBefore);
        //调用deposit
        /** =================================================================
        * .accounts()是把指令需要的账户补齐，顺序与程序规定（#[derive(Accounts)]）
        * 的一致，以这种方式传入context
        * ===================================================================
        * .rpc()是真正发送交易并等待确认,用你设置的 provider 钱包去签名
        */
        await program.methods.deposit(amount)
        .accounts({
            signer:signer,
            vault:vaultPda,
            system_Program:SystemProgram.programId
        })
        .rpc();

        //获取金库PDA存款后余额
        const vaultBalanceAfter = await connection.getBalance(vaultPda)
        //.catch(() => 0) 是 Promise（对于异步操作的期望） 的错误处理，如果报错，则返回0
        .catch(() => 0
        );
        console.log("after deposit:",vaultBalanceAfter / LAMPORTS_PER_SOL);

        assert.ok(vaultBalanceAfter >= amount.toNumber());
    });

    //测试withdraw函数
    it("Withdraw function test",async() =>{

        const beforewithdraw = await connection.getBalance(signer);
        console.log("before withdraw:",beforewithdraw / LAMPORTS_PER_SOL);
        //调用withdraw函数
        await program.methods.withdraw().
        accounts({
            signer:signer,
            vault:vaultPda,
            systemProgram:SystemProgram.programId
        })
        .rpc();

        const afterwithdraw = await connection.getBalance(signer);
        console.log("after withdraw:",afterwithdraw / LAMPORTS_PER_SOL);

        //验证
        const vaultaccount = await connection.getAccountInfo(vaultPda);

        if (vaultaccount === null) {

            assert.ok(true);
      
          } else {
      
            assert.equal(vaultaccount.lamports, 0);
      
          }
        assert.ok(afterwithdraw > beforewithdraw);
    });
});