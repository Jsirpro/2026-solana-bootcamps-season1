use anchor_lang::prelude::*; 
use anchor_spl::{ 
    associated_token::AssociatedToken, // 关联代币程序类型。
    token_interface::{ // Token 接口模块，兼容 SPL Token / Token-2022。
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface, // CPI 方法与账户类型。
        TransferChecked, // TransferChecked CPI 账户结构。
    }, 
}; 

use crate::{ 
    errors::EscrowError, // 自定义错误定义。
    state::{Escrow}, // Escrow 状态与 PDA 种子。
}; 

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Refund<'info>{
    #[account(mut)]
    pub maker:Signer<'info>,
    #[account(
      mut,
      close = maker,
      seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
      bump = escrow.bump,
      has_one = maker @ EscrowError::InvalidMaker,
      has_one = mint_a @ EscrowError::InvalidMintA,
      //has_one = mint_b @ EscrowError::InvalidMintB,
    )]
    pub escrow:Account<'info,Escrow>,
    /*============================================mint账户=========================================*/
    #[account(
        mint::token_program = token_program
    )]
    mint_a:InterfaceAccount<'info,Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// Programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info>{
    fn refund_tokens(&self) -> Result<()> {
        // Create the signer seeds for the Vault
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // Transfer Token A (Vault -> Maker)
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vault.to_account_info(),
                    to: self.maker_ata_a.to_account_info(),

                    mint: self.mint_a.to_account_info(),
                    authority: self.escrow.to_account_info(),
                },
                &signer_seeds,
            ),
            self.vault.amount,//vault余额
            self.mint_a.decimals,
        )?;
        // Close the Vault
        close_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.vault.to_account_info(),
                authority: self.escrow.to_account_info(),
                destination: self.maker.to_account_info(),
            },
            &signer_seeds,
        ))?;

        Ok(())
    }
}

pub fn handler(ctx: Context<Refund>) -> Result<()> {
    // Validate the amount
    require_gt!(ctx.accounts.vault.amount, 0, EscrowError::InvalidAmount);
    //require_gt!(amount, 0, EscrowError::InvalidAmount);

    // Deposit Tokens
    ctx.accounts.refund_tokens()?;

    Ok(())
}