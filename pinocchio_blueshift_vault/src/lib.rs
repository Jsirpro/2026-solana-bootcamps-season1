#![no_std]

use pinocchio::{
    AccountView,//account_info::AccountInfo, 
    entrypoint, nostd_panic_handler, 
    error::ProgramError, //program_error::ProgramError
    Address,//pubkey::Pubkey, 
    ProgramResult,
    address::address,
};

entrypoint!(process_instruction);
nostd_panic_handler!();

pub mod instructions;
pub use instructions::*;

// 22222222222222222222222222222222222222222222
pub const ID: Address = address!("22222222222222222222222222222222222222222222");

fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Deposit::DISCRIMINATOR, data)) => Deposit::try_from((data, accounts))?.process(),
        Some((Withdraw::DISCRIMINATOR, _)) => Withdraw::try_from(accounts)?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}