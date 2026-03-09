use pinocchio::{
    AccountView, 
    entrypoint, 
    error::ProgramError, 
    Address, 
    address::address,
    ProgramResult};
//定义程序执行的起点，比如这个程序的起点就是process_instruction()
//declared the begining excution
entrypoint!(process_instruction);

pub mod instructions;
pub use instructions::*;

pub mod state;
pub use state::*;

// 22222222222222222222222222222222222222222222
pub const ID: Address = address!("22222222222222222222222222222222222222222222");
//程序起点
fn process_instruction(
    //已部署程序的公钥
    //The address of the program
    _program_id: &Address,
    //指令中传递的所有账户
    //All of the accounts in the instruction
    accounts: &[AccountView],
    //包含 Discriminator 和用户提供数据的不透明字节数组
    //discriminator and any uer-supllied data
    instruction_data: &[u8],
) -> ProgramResult {
    //split_first:提取判别字节
    //fn split_first(&self) -> Option<(&T, &[T])>
    //返回的是第一个字节和剩下的字节
    //用match 来确定要执行的指令
    match instruction_data.split_first() {
        //在Make，Take，Refund中分别定义DISCRIMINATOR用于匹配指令
        //对于Make指令，还需要把剩余数据绑定到data并传入Make()
        Some((Make::DISCRIMINATOR, data)) => Make::try_from((data, accounts))?.process(),
        Some((Take::DISCRIMINATOR, _)) => Take::try_from(accounts)?.process(),
        Some((Refund::DISCRIMINATOR, _)) => Refund::try_from(accounts)?.process(),
        _ => Err(ProgramError::InvalidInstructionData)
    }
}