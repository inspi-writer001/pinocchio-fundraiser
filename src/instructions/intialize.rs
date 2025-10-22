use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    pubkey::{self, log},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
// use pinocchio_log::log;
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [u8; 8],
    pub duration: [u8; 1],
    pub bump: [u8; 1],
}

impl Fundraiser {
    const LEN: usize = core::mem::size_of::<Fundraiser>();

    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}

pub fn process_intialize_fundraiser(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint, fundraiser, vault, system_program, token_program, associated_token_program, rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    // checks
    // check that maker is a signer

    // check that fundraiser is == derived fundraiser to see that program id and seeds match
    // check fundraiser is authority of vault
    // check that mint is created
    // check that mint owner is maker

    // create fundraiser account
    Ok(())
}
