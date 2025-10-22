use bytemuck::{Pod, Zeroable};

use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    log, msg,
    pubkey::{self, log},
    sysvars::{self, rent::Rent, Sysvar},
    ProgramResult,
};
// use pinocchio_log::log;
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct InitializeFundraiser {
    pub amount_to_raise: u64,
    pub duration: u64,
}

impl InitializeFundraiser {
    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [u8; 8],
    pub duration: [u8; 8],
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
    // check that maker is a signer ✅
    assert!(&maker.is_signer(), "Maker should be a signer");

    let ix_data = bytemuck::try_pod_read_unaligned::<InitializeFundraiser>(data)
        .map_err(|_| pinocchio::program_error::ProgramError::InvalidInstructionData)?;

    // check that fundraiser derived from client == derived fundraiser in program to see that program id and seeds match ✅

    let fundraiser_keys = [b"fundraiser".as_ref(), maker.key().as_ref()];
    let (fundraiser_pda, bump) = pubkey::find_program_address(&fundraiser_keys, &crate::ID);

    assert_eq!(
        &fundraiser_pda,
        fundraiser.key(),
        "You provided the wrong fundraiser pda"
    );

    // check fundraiser is authority of vault ✅
    let vault_state = pinocchio_token::state::TokenAccount::from_account_info(&vault)?;

    assert_eq!(vault_state.owner(), fundraiser.key(), "Illegal Owner");

    // check that mint is created ✅
    let mint_state = pinocchio_token::state::Mint::from_account_info(&mint)?;
    assert!(mint_state.is_initialized(), "Yo!, mint do not exist");

    // check that vault mint is mint ✅
    assert_eq!(
        vault_state.mint(),
        mint.key(),
        "Yo!, You provided wrong mint address"
    );

    // create fundraiser account
    let initial_bump = bump.to_le();
    let bump = [initial_bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);
    CreateAccount {
        from: maker,
        lamports: Rent::get()?.minimum_balance(Fundraiser::LEN),
        owner: &crate::ID,
        space: Fundraiser::LEN as u64,
        to: fundraiser,
    }
    .invoke_signed(&[seeds])?;

    let data = &mut fundraiser.try_borrow_mut_data()?;
    let fundraiser_state = &mut bytemuck::from_bytes_mut::<Fundraiser>(data);

    // let mut fundraiser_state = Fundraiser {
    //     maker: *maker.key(),
    //     mint_to_raise: *mint.key(),
    //     amount_to_raise: ix_data.amount_to_raise.to_le_bytes(),
    //     current_amount: 0u64.to_le_bytes(),
    //     time_started: (sysvars::clock::Clock::get()?.unix_timestamp as u64).to_le_bytes(),
    //     duration: ix_data.duration.to_le_bytes(),
    //     bump,
    // };

    // let mut data = fundraiser.try_borrow_mut_data()?;
    // data[..Fundraiser::LEN].copy_from_slice(bytemuck::bytes_of(&fundraiser_state));

    fundraiser_state.amount_to_raise = ix_data.amount_to_raise.to_le_bytes();
    fundraiser_state.bump = bump;
    fundraiser_state.current_amount = 0u64.to_le_bytes();
    fundraiser_state.duration = ix_data.duration.to_le_bytes();
    fundraiser_state.maker = *maker.key();
    fundraiser_state.mint_to_raise = *mint.key();
    fundraiser_state.time_started =
        (sysvars::clock::Clock::get()?.unix_timestamp as u64).to_le_bytes();

    Ok(())
}
