use pinocchio::{
    cpi::{Seed,Signer},
    AccountView,
    error::ProgramError,
    ProgramResult,
    Address,
};
use solana_program_log::log;

use pinocchio_token::instructions::{Transfer, CloseAccount};
use crate::{ID,state::Escrow};
use super::helpers::{
    SignerAccount, MintInterface, AssociatedTokenAccount, ProgramAccount, TokenAccount,
};
/*===================*Take所需账号数据结构=======================*/
pub struct TakeAccounts<'a> {
    pub taker: &'a AccountView,
    pub maker: &'a AccountView,
    pub escrow: &'a AccountView,
    pub mint_a: &'a AccountView,
    pub mint_b: &'a AccountView,
    pub vault: &'a AccountView,
    pub taker_ata_a: &'a AccountView,
    pub taker_ata_b: &'a AccountView,
    pub maker_ata_b: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}
//检查take数据解构
impl<'a> TryFrom<&'a [AccountView]> for TakeAccounts<'a> {
  type Error = ProgramError;

  fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
    //检查账户顺序和数量
    let [taker, maker, escrow, mint_a, mint_b, vault, taker_ata_a, taker_ata_b, maker_ata_b, system_program, token_program, _] = accounts else {
      return Err(ProgramError::NotEnoughAccountKeys);
     };

    // Basic Accounts Checks
    SignerAccount::check(taker)?;
    //验证程序账户的DATA长度是否等于0
    ProgramAccount::check(escrow)?;
    MintInterface::check(mint_a)?;
    MintInterface::check(mint_b)?;
    AssociatedTokenAccount::check(taker_ata_b, taker, mint_b, token_program)?;
    AssociatedTokenAccount::check(vault, escrow, mint_a, token_program)?;        
    
    // Return the accounts//    
    Ok(Self {
        taker,
        maker,
        escrow,
        mint_a,
        mint_b,
        taker_ata_a,
        taker_ata_b,
        maker_ata_b,
        vault,
        system_program,
        token_program,
    })
  }
}

/*===================初始化=======================*/
pub struct Take<'a> {
    pub accounts: TakeAccounts<'a>,
  }

//检查
impl<'a> TryFrom<&'a [AccountView]> for Take<'a> {
  type Error = ProgramError;
  
  fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
    let accounts = TakeAccounts::try_from(accounts)?;

    // take流程是TOKEN_a转给taker，TOKEN_b转给maker，所以要
    //验证双方是否有相应的ATA账户，如果没有则需要创建
    AssociatedTokenAccount::init_if_needed(
      accounts.taker_ata_a,
      accounts.mint_a,
      accounts.taker,
      accounts.taker,
      accounts.system_program,
      accounts.token_program,
    )?;

    AssociatedTokenAccount::init_if_needed(
      accounts.maker_ata_b,
      accounts.mint_b,
      accounts.taker,
      accounts.maker,
      accounts.system_program,
      accounts.token_program,
    )?;

    Ok(Self {
      accounts,
    })
  }
}

/*===================指令逻辑=======================*/
impl<'a> Take<'a> {
    //定义Take的DISCRIMINATOR为1
    pub const DISCRIMINATOR: &'a u8 = &1;
    
    pub fn process(&mut self) -> ProgramResult {
      log!("[Take] process start");
      //try_borrow():创建一个不可变引用，用于取出escrow中的data的值
      let data = self.accounts.escrow.try_borrow()?;
      //把data从&[u8]格式转换成struct格式
      let escrow = Escrow::load(&data)?;

      // Check if the escrow is valid
      let escrow_key = Address::create_program_address(&[b"escrow", self.accounts.maker.address().as_ref()   , &escrow.seed.to_le_bytes(), &escrow.bump], &ID)?;
      if &escrow_key != self.accounts.escrow.address() {
        return Err(ProgramError::InvalidAccountOwner);
      }

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
      log!("[Take] vault token amount (before transfer)：{}",amount);
      log!("[Take] vault lamports (before close)：{}",self.accounts.vault.lamports());
      log!("[Take] escrow PDA lamports (before close):{}",self.accounts.escrow.lamports());

      // Transfer from the Vault to the Taker
      Transfer {
        from: self.accounts.vault,
        to: self.accounts.taker_ata_a,
        authority: self.accounts.escrow,
        amount,
      }.invoke_signed(&[signer.clone()])?;
      log!("[Take] vault -> taker_ata_a transfer ok");

      // Close the Vault
      CloseAccount {
        account: self.accounts.vault,
        destination: self.accounts.maker,
        authority: self.accounts.escrow,
      }.invoke_signed(&[signer.clone()])?;
      log!("[Take] vault closed, vault lamports after CloseAccount");

      // Transfer from the Taker to the Maker
      Transfer {
        from: self.accounts.taker_ata_b,
        to: self.accounts.maker_ata_b,
        authority: self.accounts.taker,
        amount: escrow.receive,
      }.invoke()?;
      log!("[Take] taker_ata_b -> maker_ata_b transfer ok");

      // Close the Escrow（通过系统程序 CPI 转出 lamports，需 PDA signer）
      drop(data);
      //&[signer.clone()]加签名
      ProgramAccount::close(self.accounts.escrow, self.accounts.maker)?;
      log!("[Take] escrow closed, escrow lamports after close:{}",self.accounts.escrow.lamports());
      log!("[Take] escrow closed, vault lamports after close:{}",self.accounts.vault.lamports());
      //log!("[Take] escrow closed, escrow lamports after close:{}",self.accounts.escrow.lamports());
      //drop(data);
      log!("[Take] process ok");
      Ok(())
    }
  }