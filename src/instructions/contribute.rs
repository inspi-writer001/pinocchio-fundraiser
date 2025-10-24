use bytemuck::{Pod, Zeroable};

use pinocchio::{
    account_info::AccountInfo,
    instruction::{Account, Seed, Signer},
    log, msg,
    pubkey::{self, find_program_address, log},
    sysvars::{self, rent::Rent, Sysvar},
    ProgramResult,
};
// use pinocchio_log::log;
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;

use crate::state::{Contributor, Fundraiser};

pub fn process_contribute(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [contributor, mint, fundraiser, vault, contributor_ata, contributor_pda, system_program, token_program, associated_token_program, rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };
    let amount = data;

    // check that contributor is signer ✅
    assert!(&contributor.is_signer(), "Conributor should be a signer");

    // check that this program owns fundraiser ✅
    assert!(
        &fundraiser.is_owned_by(&crate::ID),
        "User derived Wrong Fundraiser we do not own"
    );

    // check that fundraiser exists ✅
    let data = &mut fundraiser.try_borrow_mut_data()?;
    let fundraiser_state = &mut bytemuck::from_bytes_mut::<Fundraiser>(data);

    // check that the mint is correct in fundraiser field ✅
    assert_eq!(
        mint.key(),
        &fundraiser_state.mint_to_raise,
        "User Provided Wrong Mint"
    );

    // check that provided vault is owned by fundraiser state
    let vault_state = pinocchio_token::state::TokenAccount::from_account_info(&vault)?;
    assert_eq!(
        vault_state.owner(),
        fundraiser.key(),
        "Illegal Owner of Vault"
    );

    // check that contributor has suffifient amount to transfer
    let contributor_ata_state =
        pinocchio_token::state::TokenAccount::from_account_info(&*contributor_ata)?;

    assert!(
        contributor_ata_state.amount() >= u64::from_le_bytes(amount.try_into().unwrap()),
        "Insufficient amount to send"
    );

    // check that contributor is sending above minimum
    assert!(
        u64::from_le_bytes(amount.try_into().unwrap()) >= fundraiser_state.min_sendable(),
        "Insufficient amount to send"
    );

    // check that contributor is sending below maximum
    assert!(
        u64::from_le_bytes(amount.try_into().unwrap()) <= fundraiser_state.max_sendable(),
        "Insufficient amount to send"
    );

    // create contributor pda if it's not initialized [init-if-needed]
    let contributor_seeds: &[&[u8]] = &[b"contributor", contributor.key()];

    // let derived_contributor_pda_state = Account::try_from(contributor).unwrap();
    // // bytemuck::try_pod_read_unaligned::<Contributor>(&derived_contributor_pda).unwrap();

    if contributor_pda.lamports() == 0 || contributor_pda.data_is_empty() {
        let (contributor_pda_state, bump) = find_program_address(&contributor_seeds, &crate::ID);
        // assert that the provided conttibutor_state key is same as the one derived
        assert_eq!(
            contributor_pda.key(),
            &contributor_pda_state,
            "You provided the wrong contributor pda"
        );
        // create the account
        let initial_bump = bump.to_le();
        let bump = [initial_bump];
        let seed = [
            Seed::from(b"fundraiser"),
            Seed::from(contributor.key()),
            Seed::from(&bump),
        ];
        let seeds = Signer::from(&seed);
        CreateAccount {
            from: contributor,
            lamports: Rent::get()?.minimum_balance(Contributor::LEN),
            owner: &crate::ID,
            space: Contributor::LEN as u64,
            to: fundraiser,
        }
        .invoke_signed(&[seeds])?;
        // deposit to the vault
        Transfer {
            amount: u64::from_le_bytes(amount.try_into().unwrap()),
            authority: contributor,
            from: contributor_ata,
            to: vault,
        }
        .invoke()?;

        // increase contributor amount by how much was deposited
        let data = &mut contributor.try_borrow_mut_data()?;
        let derived_contributor_pda_state = bytemuck::from_bytes_mut::<Contributor>(data);

        derived_contributor_pda_state.amount =
            (u64::from_le_bytes(derived_contributor_pda_state.amount)
                + u64::from_le_bytes(amount.try_into().unwrap()))
            .to_le_bytes()
    } else {
        // Account exists - deserialize it
        let data = &mut contributor.try_borrow_mut_data()?;
        let derived_contributor_pda_state = bytemuck::from_bytes_mut::<Contributor>(data);

        // deposit to the vault
        // increase contributor amount by how much was deposited
    }

    Ok(())
}
