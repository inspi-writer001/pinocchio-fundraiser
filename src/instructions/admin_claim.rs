use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    pubkey::{self, find_program_address},
    sysvars::{self, clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};

use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;

use crate::state::{Contributor, Fundraiser};

fn admin_claim(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint, fundraiser, vault, maker_ata, _system_program, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    // check that maker is signer
    assert!(maker.is_signer(), "Maker should be a signer");
    // check that fundraiser exists [ is not closed ]
    assert!(
        !fundraiser.data_is_empty(),
        "Fundraiser is closed or doesn't exist"
    );

    let fundraiser_state =
        bytemuck::pod_read_unaligned::<Fundraiser>(&fundraiser.try_borrow_data().unwrap());

    // check that threshold has been met [target amount has been reached]
    let clock = Clock::get();
    let current_time = clock?.unix_timestamp as u64;

    let vault_state = pinocchio_token::state::TokenAccount::from_account_info(vault).unwrap();

    assert!(
        fundraiser_state.amount_to_raise <= u64::to_le_bytes(vault_state.amount()),
        "You have not reached the target amount"
    );

    assert!(
        fundraiser_state.duration <= u64::to_le_bytes(current_time),
        "Time has not passed"
    );

    // check that provided mint is exactly same as in the fundraiser state
    assert_eq!(mint.key(), &fundraiser_state.mint_to_raise, "Wrong Mint");

    // check that fundraiser is authority of vault
    assert_eq!(
        vault_state.close_authority(),
        Some(fundraiser.key()),
        "Fundraiser does not own Vault"
    );

    // check that vault is derived from the mint
    assert_eq!(vault_state.mint(), mint.key(), "Vault has wrong mint");

    // check that maker ata is of the mint address
    let maker_ata_state =
        pinocchio_token::state::TokenAccount::from_account_info(&maker_ata).unwrap();
    assert_eq!(
        maker_ata_state.mint(),
        mint.key(),
        "Maker ata has wrong mint"
    );

    // transfer to admin
    let initial_bump = u8::from_le_bytes(fundraiser_state.bump);
    let bump = [initial_bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker_ata.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);
    // Transfer {
    //     amount: vault_state.amount(),
    //     authority: fundraiser,
    //     from: vault,
    //     to: maker_ata,
    // }
    // .invoke_signed(&seeds);
    // close vault
    // close fundraiser account
    Ok(())
}
