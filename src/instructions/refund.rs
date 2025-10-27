use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

use pinocchio_token::instructions::Transfer;

use crate::state::{Contributor, Fundraiser};

pub fn process_refund(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [user, creator, mint, fundraiser, vault, contributor_ata, contributor_pda, _system_program, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    // check that user is signer ✅
    assert!(&user.is_signer(), "Conributor should be a signer");

    let fundraiser_bump = {
        let fundraiser_data = fundraiser.try_borrow_data().unwrap();
        let fundraiser_state =
            bytemuck::try_pod_read_unaligned::<Fundraiser>(&fundraiser_data).unwrap();

        // check that duration has elapsed
        let clock = Clock::get();
        let current_time = clock?.unix_timestamp as u64;
        assert!(
            u64::from_le_bytes(fundraiser_state.duration) <= current_time,
            "Time has not elapsed"
        );

        let vault_state = pinocchio_token::state::TokenAccount::from_account_info(vault).unwrap();
        // check that target was not met
        assert!(
            u64::from_le_bytes(fundraiser_state.amount_to_raise) <= vault_state.amount(),
            "Financial taget was met, be faithful to your commitment"
        );

        // check that fundraiser is owner of vault
        assert_eq!(
            vault_state.owner(),
            fundraiser.key(),
            "Your Fundraiser does not own mint, Fix that"
        );

        // check that vault is of the mint
        assert_eq!(
            vault_state.mint(),
            mint.key(),
            "You provided wrong vault ata"
        );

        fundraiser_state.bump
    };

    let conmtributor_amount = {
        let contributor_data = contributor_pda.try_borrow_data().unwrap();
        let contributor_state =
            bytemuck::try_pod_read_unaligned::<Contributor>(&contributor_data).unwrap();

        // check that user contributor state exists
        assert!(!contributor_pda.data_is_empty(), "Contributor PDA is empty");
        // check that user has some balance deposited
        assert!(
            u64::from_le_bytes(contributor_state.amount) > 0,
            "You have zero balance deposited"
        );

        contributor_state.amount
    };

    {
        let contributor_ata_state =
            pinocchio_token::state::TokenAccount::from_account_info(contributor_ata).unwrap();
        // check that user ata is of the mint
        assert_eq!(
            contributor_ata_state.mint(),
            mint.key(),
            "You provided wrong user ata"
        );
    }

    let initial_bump = u8::from_le_bytes(fundraiser_bump);
    let bump = [initial_bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(creator.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);
    // transfer to the user ata
    Transfer {
        amount: u64::from_le_bytes(conmtributor_amount),
        authority: fundraiser,
        from: vault,
        to: contributor_ata,
    }
    .invoke_signed(&[seeds])?;
    // close user contributor state

    Ok(())
}
