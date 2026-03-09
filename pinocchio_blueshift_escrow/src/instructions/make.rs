use pinocchio::{                     // 从 pinocchio 主 crate 引入常用类型和宏
    cpi::{Seed},                     // 引入 Seed，用于构造 PDA signer 的种子
    AccountView,                     // 引入 AccountView，封装链上账户视图
    error::ProgramError,             // 引入 ProgramError，统一表示程序错误类型
    Address,                         // 引入 Address，表示账户地址（Pubkey）
    ProgramResult,                   // 引入 ProgramResult，= Result<(), ProgramError>
};                                   // 结束 use pinocchio 块

use pinocchio_token::instructions::Transfer; // 引入 pinocchio_token 中的 Transfer CPI 指令
use core::mem::size_of;                      // 引入 size_of，用于检查指令数据长度
use crate::{state::Escrow};                  // 引入本 crate 中的 Escrow 账户状态类型

use super::helpers::{                       // 从同目录下的 helpers 模块引入辅助检查工具
    SignerAccount,                          // SignerAccount：检查账户是否为签名者
    MintInterface,                          // MintInterface：检查 mint 账户是否合法
    AssociatedTokenAccount,                 // AssociatedTokenAccount：操作/检查 ATA 账户
    ProgramAccount,                         // ProgramAccount：初始化/管理程序自有账户（PDA）
};                                          // 结束 use super::helpers 块
// use pinocchio_token::state::{SignerAccount, MintInterface, AssociatedTokenAccount, ProgramAccount}; // 备用：也可以直接从 pinocchio_token::state 引入

/*===================*Make所需账号数据结构=======================*/ 
pub struct MakeAccounts<'a> {               // MakeAccounts：封装 Make 指令的账户集合，生命周期为 'a
    pub maker: &'a AccountView,             // maker：发起方钱包账户
    pub escrow: &'a AccountView,            // escrow：托管账户（PDA）
    pub mint_a: &'a AccountView,            // mint_a：用户支付的 token 的 mint
    pub mint_b: &'a AccountView,            // mint_b：用户希望收到的 token 的 mint
    pub maker_ata_a: &'a AccountView,       // maker_ata_a：发起方持有 mint_a 的 ATA
    pub vault: &'a AccountView,             // vault：用于存放用户锁定资金的 vault ATA
    pub system_program: &'a AccountView,    // system_program：系统程序（创建账户等）
    pub token_program: &'a AccountView,     // token_program：SPL Token 程序
}                                           // 结束 MakeAccounts 结构体定义

// 检查 MakeAccounts 的实现，从原始账户数组中解析并校验
impl<'a> TryFrom<&'a [AccountView]> for MakeAccounts<'a> { // 为 MakeAccounts 实现 TryFrom，输入为账户切片
    type Error = ProgramError;                              // 解析失败时返回 ProgramError
    // 从账户数组构造 MakeAccounts
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> { 
        let [                                                   // 使用解构模式按顺序取出各账户
            maker,                                              // 0: 发起方
            escrow,                                             // 1: 托管账户
            mint_a,                                             // 2: 支付 token 的 mint
            mint_b,                                             // 3: 收取 token 的 mint
            maker_ata_a,                                        // 4: 发起方 ATA (mint_a)
            vault,                                              // 5: vault ATA
            system_program,                                     // 6: 系统程序账户
            token_program,                                      // 7: Token 程序账户
            _                                                   // 8: 额外占位（一般为 rent/sysvar 之类）
        ] = accounts else {                                     // 如果数量或顺序不符合，直接报错
            return Err(ProgramError::NotEnoughAccountKeys);     // 账户数量不足错误
        };
  
        // Basic Accounts Checks
        //因为很多账号都需要确认信息，所以把检查相关代码都写在helpers.rs里。
        SignerAccount::check(maker)?;                          // 确认 maker 是签名者
        MintInterface::check(mint_a)?;                         // 检查 mint_a 是否为合法的 mint 账户
        MintInterface::check(mint_b)?;                         // 检查 mint_b 是否为合法的 mint 账户
        AssociatedTokenAccount::check(                         // 检查 maker_ata_a 是否为合法 ATA
            maker_ata_a,                                       // 待检查的 ATA 账户
            maker,                                             // 该 ATA 的 owner
            mint_a,                                            // 该 ATA 对应的 mint
            token_program,                                     // Token 程序账户
        )?;
  
        // Return the accounts                                // 构造并返回 MakeAccounts 实例
        Ok(Self {
            maker,                                            // 赋值 maker 字段
            escrow,                                           // 赋值 escrow 字段
            mint_a,                                           // 赋值 mint_a 字段
            mint_b,                                           // 赋值 mint_b 字段
            maker_ata_a,                                      // 赋值 maker_ata_a 字段
            vault,                                            // 赋值 vault 字段
            system_program,                                   // 赋值 system_program 字段
            token_program,                                    // 赋值 token_program 字段
        })
    }
}

/*===================escrow账户=======================*/   
pub struct MakeInstructionData {            // MakeInstructionData：封装 Make 指令的参数
    pub seed: u64,                          // seed：PDA 派生用的种子（业务自定义）
    pub receive: u64,                       // receive：期望收到的 token B 数量
    pub amount: u64,                        // amount：发起方锁定的 token A 数量
}

// 为 MakeInstructionData 实现从字节数组解析的逻辑
impl<'a> TryFrom<&'a [u8]> for MakeInstructionData { // 输入为指令数据字节切片
    type Error = ProgramError;                        // 解析失败时返回 ProgramError
  
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> { // 从原始字节流构造 MakeInstructionData
        if data.len() != size_of::<u64>() * 3 {                 // 检查长度是否为 3 个 u64
            return Err(ProgramError::InvalidInstructionData);   // 长度不对则返回指令数据无效
        }
  
        let seed = u64::from_le_bytes(data[0..8].try_into().unwrap());   // 从前 8 字节解析 seed（小端）
        let receive = u64::from_le_bytes(data[8..16].try_into().unwrap()); // 从中间 8 字节解析 receive
        let amount = u64::from_le_bytes(data[16..24].try_into().unwrap()); // 从最后 8 字节解析 amount
  
        // Instruction Checks                               // 指令参数有效性检查
        if amount == 0 {                                   // 若锁定数量为 0
            return Err(ProgramError::InvalidInstructionData); // 视为指令数据无效
        }
  
        Ok(Self {                                         // 构造并返回 MakeInstructionData
            seed,                                         // 设置 seed 字段
            receive,                                      // 设置 receive 字段
            amount,                                       // 设置 amount 字段
        })
    }
}

/*==========================================*/    
// 定义 Make 指令执行对象，包含账户、数据与 bump
pub struct Make<'a> {                         // Make：执行 Make 指令所需的上下文
    pub accounts: MakeAccounts<'a>,           // accounts：前面解析好的账户集合
    pub instruction_data: MakeInstructionData,// instruction_data：前面解析好的指令参数
    pub bump: u8,                             // bump：escrow PDA 的 bump 值
}
//创建Make指令所需的PDA  ATA 账户
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for Make<'a> { // 为 Make 实现 TryFrom<(data, accounts)>
    type Error = ProgramError;                                 // 错误类型同样为 ProgramError
    
    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> { // 从指令数据与账户数组构造 Make
        let accounts = MakeAccounts::try_from(accounts)?;      // 先解析并检查账户集合
        let instruction_data = MakeInstructionData::try_from(data)?; // 再解析escrow账户所需数据
  
        let (_, bump) = Address::find_program_address(        // 根据种子派生 escrow PDA，并获得 bump
            &[
                b"escrow",                                   // 固定前缀 "escrow"
                accounts.maker.address().as_ref(),           // 发起方地址字节
                &instruction_data.seed.to_le_bytes(),        // 指令里传入的 seed
            ],
            &crate::ID,                                      // 当前程序 ID
        );
  
        let seed_binding = instruction_data.seed.to_le_bytes(); // 将 seed 转为字节数组以便复用
        let bump_binding = [bump];                             // bump 也放入单字节数组
        let escrow_seeds = [                                   // 构造 escrow PDA 的种子数组
            Seed::from(b"escrow"),                            // 种子 1：固定字符串 "escrow"
            Seed::from(accounts.maker.address().as_ref()),    // 种子 2：发起方地址
            Seed::from(&seed_binding),                        // 种子 3：业务 seed
            Seed::from(&bump_binding),                        // 种子 4：bump
        ];
        // 使用 ProgramAccount 辅助初始化 Escrow 账户:数据账户    
        ProgramAccount::init::<Escrow>(                       
            accounts.maker,                                   // 付费者账户（创建 PDA 支付租金）
            accounts.escrow,                                  // 目标 escrow 账户（PDA）
            &escrow_seeds,                                    // 用于签名的 PDA 种子
            Escrow::LEN,                                      // Escrow 账户数据长度
        )?;
  
        // 初始化 vault ATA 账户
        AssociatedTokenAccount::init(                        // 调用 ATA 辅助初始化 vault
            accounts.vault,                                  // vault ATA 账户
            accounts.mint_a,                                 // mint_a，表示要锁定的 token 类型
            accounts.maker,                                  // 资金来源 owner（发起方）
            accounts.escrow,                                 // vault 的 owner（托管账户 PDA）
            accounts.system_program,                         // 系统程序账户
            accounts.token_program,                          // Token 程序账户
        )?;
  
        Ok(Self {                                            // 构造并返回 Make 上下文
            accounts,                                        // 存储解析好的账户集合
            instruction_data,                                // 存储解析好的指令数据
            bump,                                            // 存储 PDA 的 bump
        })
    }
}

/*===================转账逻辑=======================*/  
impl<'a> Make<'a> {                        // 为 Make 实现方法
    pub const DISCRIMINATOR: &'a u8 = &0;  // 指令辨识符（discriminator），用于区分不同指令
    
    pub fn process(&mut self) -> ProgramResult { // process：执行 Main 逻辑，返回 ProgramResult
        // Populate the escrow account             // 填充 escrow 账户的数据字段
        // let mut data = self.accounts.escrow.try_borrow_mut_data()?; // 旧写法：直接借用数据字段
        let mut data = self.accounts.escrow.try_borrow_mut()?;        // 新写法：借用整个账户并获取可变引用
        let escrow = Escrow::load_mut(data.as_mut())?;                // 使用 Escrow::load_mut 将原始数据映射为 Escrow 结构
        
        //在state.rs里为Escrow实现了set_inner方法
        escrow.set_inner(                                             
            self.instruction_data.seed,                               // 记录 seed（用于后续 PDA 复现）
            self.accounts.maker.address().clone(),                    // 记录发起方地址
            self.accounts.mint_a.address().clone(),                   // 记录 token A 的 mint 地址
            self.accounts.mint_b.address().clone(),                   // 记录 token B 的 mint 地址
            self.instruction_data.receive,                            // 记录期望收到的 token B 数量
            [self.bump],                                              // 保存 bump 值，以便之后重新派生 escrow PDA
        );
  
        // Transfer tokens to vault                                   // 将发起方的 token A 转入 vault
        Transfer {                                                    // 构造 Transfer CPI 调用
            from: self.accounts.maker_ata_a,                          // from：发起方的 ATA（mint_a）
            to: self.accounts.vault,                                  // to：vault ATA（托管账户持有）
            authority: self.accounts.maker,                           // authority：发起方账户（需要签名）
            amount: self.instruction_data.amount,                     // amount：锁定的 token A 数量
        }.invoke()?;                                                  // 执行 CPI 调用，若失败则返回错误
  
        Ok(())                                                        // 所有操作成功，返回 Ok
    }
}