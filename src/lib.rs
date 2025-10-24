use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};

use crate::instructions::FundraisingInstructions;

mod instructions;
mod state;
mod tests;

entrypoint!(process_instruction);

pinocchio_pubkey::declare_id!("27abzM8KfWuiYyiy6T3Dv1EeJWSPuBK7DDjtBQoapEfP");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(pinocchio::program_error::ProgramError::InvalidInstructionData)?;

    match FundraisingInstructions::try_from(discriminator)? {
        FundraisingInstructions::Initialize => {
            pinocchio_log::log!("got to entry");
            instructions::process_intialize_fundraiser(accounts, data)?
        }
        FundraisingInstructions::Contribute => instructions::process_contribute(accounts, data)?,
        // FundraisingInstructions::CheckContributions => {}
        // FundraisingInstructions::Refund => {}
        // FundraisingInstructions::MakeV2 => instructions::process_make_instruction_v2(accounts, data)?,
        _ => return Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
    }
    Ok(())
}
