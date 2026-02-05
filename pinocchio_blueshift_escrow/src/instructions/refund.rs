use pinocchio::{                     // 从 pinocchio 主 crate 引入常用类型和宏
    cpi::{Seed,Signer},                     // 引入 Seed，用于构造 PDA signer 的种子
    AccountView,                     // 引入 AccountView，封装链上账户视图
    error::ProgramError,             // 引入 ProgramError，统一表示程序错误类型
    Address,                         // 引入 Address，表示账户地址（Pubkey）
    ProgramResult,                   // 引入 ProgramResult，= Result<(), ProgramError>
};                                   // 结束 use pinocchio 块

use pinocchio_token::instructions::{Transfer,CloseAccount}; // 引入 pinocchio_token 中的 Transfer CPI 指令
//use core::mem::size_of;                      // 引入 size_of，用于检查指令数据长度
use crate::{state::Escrow};                  // 引入本 crate 中的 Escrow 账户状态类型

use super::helpers::{                       // 从同目录下的 helpers 模块引入辅助检查工具
    SignerAccount,                          // SignerAccount：检查账户是否为签名者
    MintInterface,                          // MintInterface：检查 mint 账户是否合法
    AssociatedTokenAccount,                 // AssociatedTokenAccount：操作/检查 ATA 账户
    ProgramAccount,   
    TokenAccount,                      // ProgramAccount：初始化/管理程序自有账户（PDA）
};   

/*===================*Refund所需账号数据结构=======================*/ 
pub struct RefundAccounts<'a>{
    pub maker: &'a AccountView,             // maker：发起方钱包账户
    pub escrow: &'a AccountView,            // escrow：托管账户（PDA）
    pub mint_a: &'a AccountView,            // mint_a：用户支付的 token 的 mint
    pub vault: &'a AccountView,             // vault：用于存放用户锁定资金的 vault ATA
    pub maker_ata_a: &'a AccountView,       // maker_ata_a：发起方持有 mint_a 的 ATA
    pub system_program: &'a AccountView,    // system_program：系统程序（创建账户等）
    pub token_program: &'a AccountView,     // token_program：SPL Token 程序
}

// 检查 RefundAccounts 的实现，从原始账户数组中解析并校验
impl<'a> TryFrom<&'a [AccountView]> for RefundAccounts<'a> { // 为 RefundAccounts 实现 TryFrom，输入为账户切片
    type Error = ProgramError;                              // 解析失败时返回 ProgramError
  
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> { // 从账户数组构造 MakeAccounts
        let [                                                   // 使用解构模式按顺序取出各账户
            maker,                                              // 0: 发起方
            escrow,                                             // 1: 托管账户
            mint_a,                                             // 2: 支付 token 的 mint                      
            vault,                                              // 3: vault ATA
            maker_ata_a,                                        // 4: 发起方 ATA (mint_a)
            system_program,                                     // 5: 系统程序账户
            token_program,                                      // 6: Token 程序账户
            _                                                   // 7: 额外占位（一般为 rent/sysvar 之类）
        ] = accounts else {                                     // 如果数量或顺序不符合，直接报错
            return Err(ProgramError::NotEnoughAccountKeys);     // 账户数量不足错误
        };
  
        // Basic Accounts Checks                               // 基础账户合法性检查
        SignerAccount::check(maker)?;                          // 确认 maker 是签名者
        ProgramAccount::check(escrow)?;                        //
        MintInterface::check(mint_a)?;                         // 检查 mint_a 是否为合法的 mint 账户
        AssociatedTokenAccount::check(                         // 检查 maker_ata_a 是否为合法 ATA
            vault,                                              // 待检查的 ATA 账户
            escrow,                                             // 该 ATA 的 owner
            mint_a,                                            // 该 ATA 对应的 mint
            token_program,                                     // Token 程序账户
        )?;
  
        // Return the accounts                                // 构造并返回 MakeAccounts 实例
        Ok(Self {
            maker,                                            // 赋值 maker 字段
            escrow,                                           // 赋值 escrow 字段
            mint_a,                                           // 赋值 mint_a 字段 
            vault,                                            // 赋值 vault 字段
            maker_ata_a,                                      // 赋值 maker_ata_a 字段
            system_program,                                   // 赋值 system_program 字段
            token_program,                                    // 赋值 token_program 字段
        })
    }
}

/*===================*Refund指令数据结构=======================*/   
pub struct Refund<'a>{            // MakeInstructionData：封装 Make 指令的参数
    pub accounts: RefundAccounts<'a>,
}
//检查refund
impl<'a> TryFrom<&'a [AccountView]> for Refund<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = RefundAccounts::try_from(accounts)?;

        // 初始化 maker_ata_a（如果不存在）
        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_a,
            accounts.mint_a,
            accounts.maker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self { accounts })
    }
}

/*===================指令逻辑=======================*/
//1.从vault转token_a到ATA_a
//2.关闭vault
impl<'a> Refund<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2; //指令辨识符（discriminator），用于区分不同指令

    pub fn process(&mut self) -> ProgramResult{
        let data = self.accounts.escrow.try_borrow()?;
        let escrow = Escrow::load(&data)?;
    
        // Check if the escrow is valid
        let escrow_key = Address::create_program_address(&[b"escrow", self.accounts.maker.address().as_ref()   , &escrow.seed.to_le_bytes(), &escrow.bump], &crate::ID)?;
        if &escrow_key != self.accounts.escrow.address() {
        return Err(ProgramError::InvalidAccountOwner);
        };

        let seed_binding = escrow.seed.to_le_bytes();
        let bump_binding = escrow.bump;
        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.address().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];
        let signer = Signer::from(&escrow_seeds);
        let amount = TokenAccount::from_account_info(self.accounts.vault)?.amount()?;
      
        // Transfer from the Vault to the Maker
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.maker_ata_a,
            authority: self.accounts.escrow,
            amount,
        }.invoke_signed(&[signer.clone()])?;
  
        // Close the Vault
        CloseAccount {
            account: self.accounts.vault,
            destination: self.accounts.maker,
            authority: self.accounts.escrow,
        }.invoke_signed(&[signer.clone()])?;

        // Close the Escrow
        drop(data);
        ProgramAccount::close(self.accounts.escrow, self.accounts.maker)?;
  
        Ok(())
    }
}