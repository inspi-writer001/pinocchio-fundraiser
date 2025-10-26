use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

use pinocchio_token::instructions::{CloseAccount, Transfer};

use crate::state::Fundraiser;

pub fn admin_claim(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
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

    let fundraiser_data = fundraiser.try_borrow_data().unwrap();
    let fundraiser_state = bytemuck::pod_read_unaligned::<Fundraiser>(&fundraiser_data);
    // check that threshold has been met [target amount has been reached]
    let clock = Clock::get();
    let current_time = clock?.unix_timestamp as u64;

    // drop(fundraiser_data);

    let initial_bump = u8::from_le_bytes(fundraiser_state.bump);
    let bump = [initial_bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    let vault_amount = {
        let vault_state = pinocchio_token::state::TokenAccount::from_account_info(vault).unwrap();

        // check that fundraiser is authority of vault
        assert_eq!(
            vault_state.owner(),
            fundraiser.key(),
            "Fundraiser does not own Vault"
        );

        // research log: you don't compare byte_arrays, convert to u64 before comparing - [The problem: You're comparing byte arrays ([u8; 8]) lexicographically, not numerically!

        // When you compare byte arrays with <=, Rust compares them element-by-element from left to right (lexicographic ordering), which is NOT the same as comparing the numeric values they represent.]

        assert!(
            u64::from_le_bytes(fundraiser_state.amount_to_raise) <= vault_state.amount(),
            "You have not reached the target amount"
        );

        // check that vault is derived from the mint
        assert_eq!(vault_state.mint(), mint.key(), "Vault has wrong mint");

        vault_state.amount()
    };

    // check that provided mint is exactly same as in the fundraiser state
    assert_eq!(mint.key(), &fundraiser_state.mint_to_raise, "Wrong Mint");

    {
        // check that maker ata is of the mint address
        let maker_ata_state =
            pinocchio_token::state::TokenAccount::from_account_info(&maker_ata).unwrap();
        assert_eq!(
            maker_ata_state.mint(),
            mint.key(),
            "Maker ata has wrong mint"
        );
    }

    assert!(
        u64::from_le_bytes(fundraiser_state.duration) <= current_time,
        "Time has not passed"
    );

    Transfer {
        amount: vault_amount,
        authority: fundraiser,
        from: vault,
        to: maker_ata,
    }
    .invoke_signed(&[seeds])?;
    // close vault
    CloseAccount {
        account: vault,
        authority: fundraiser,
        destination: maker_ata,
    }
    .invoke_signed(&[Signer::from(&seed)])?;
    Ok(())
}
