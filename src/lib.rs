use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};

use crate::instructions::FundraisingInstructions;

mod instructions;
mod state;
mod tests;

// vault_state, fundraiser_state

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
            instructions::process_intialize_fundraiser(accounts, data)?
        }
        FundraisingInstructions::Contribute => instructions::process_contribute(accounts, data)?,
        FundraisingInstructions::CheckContributions => {
            instructions::admin_claim(accounts, data)?;
            // instructions::transfer_tokens(accounts, data)?
        }
        FundraisingInstructions::Refund => {
            instructions::process_refund(accounts, data)?;
        }
        _ => return Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
    }
    Ok(())
}
