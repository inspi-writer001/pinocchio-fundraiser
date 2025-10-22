use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    pubkey::{self, log},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_token::instructions::CloseAccount;

use crate::state::Escrow;

pub fn process_take_instruction(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [taker, maker, maker_ata_a, maker_ata_b, mint_a, mint_b, escrow_account, taker_ata_a, taker_ata_b, escrow_ata, _system_program, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    // check that taker is signer

    // check that maker is not signer

    // check that maker and taker mints are correct
    let taker_token_account_a =
        pinocchio_token::state::TokenAccount::from_account_info(&taker_ata_a).unwrap();
    let maker_token_account_a =
        pinocchio_token::state::TokenAccount::from_account_info(&maker_ata_a).unwrap();
    let taker_token_account_b =
        pinocchio_token::state::TokenAccount::from_account_info(&taker_ata_b).unwrap();
    let maker_token_account_b =
        pinocchio_token::state::TokenAccount::from_account_info(&maker_ata_b).unwrap();

    let mint_a_account = pinocchio_token::state::Mint::from_account_info(mint_a).unwrap();

    assert_eq!(
        &taker_token_account_a.mint(),
        &mint_a.key(),
        "Invalid taker_token_account_a"
    );
    assert_eq!(
        &maker_token_account_a.mint(),
        &mint_a.key(),
        "Invalid maker_token_account_a"
    );
    assert_eq!(
        &taker_token_account_b.mint(),
        &mint_b.key(),
        "Invalid taker_token_account_a"
    );
    assert_eq!(
        &maker_token_account_b.mint(),
        &mint_b.key(),
        "Invalid maker_token_account_a"
    );

    // check that maker is the owner of maker_ata provided
    assert_eq!(
        *maker_token_account_b.owner(),
        *maker.key(),
        "maker does not own this token"
    );
    assert_eq!(
        *maker_token_account_a.owner(),
        *maker.key(),
        "maker does not own this token"
    );
    // check that maker is the creator of the make_ix [make state] and is same in escrow
    let escrow_state = Escrow::from_account_info(&escrow_account).unwrap();
    assert_eq!(
        escrow_state.maker(),
        *maker.key(),
        "Wrong maker for this escrow"
    );
    // check that the escrow is owned by this program
    assert_eq!(
        escrow_account.owner(),
        &crate::ID,
        "This program does not own the escrow"
    );

    // transfer token to maker_ata

    // transfer token to taker_ata

    // close escrow ata
    Ok(())
}

pub fn transfer_to_maker(accounts: &[AccountInfo]) -> ProgramResult {
    let [taker, _maker, _maker_ata_a, maker_ata_b, _mint_a, mint_b, escrow_account, _taker_ata_a, taker_ata_b, _escrow_ata, _system_program, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    let escrow_state = Escrow::from_account_info(&escrow_account).unwrap();

    let mint_b_account = pinocchio_token::state::Mint::from_account_info(mint_b).unwrap();

    pinocchio_token::instructions::TransferChecked {
        amount: escrow_state.amount_to_receive(),
        authority: taker,
        decimals: mint_b_account.decimals(),
        from: taker_ata_b,
        mint: mint_b,
        to: maker_ata_b,
    }
    .invoke()?;
    Ok(())
}

pub fn transfer_to_taker(accounts: &[AccountInfo]) -> ProgramResult {
    let [_taker, maker, _maker_ata_a, _maker_ata_b, mint_a, _mint_b, escrow_account, taker_ata_a, _taker_ata_b, escrow_ata, _system_program, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    let escrow_state = Escrow::from_account_info(&escrow_account).unwrap();
    let mint_a_account = pinocchio_token::state::Mint::from_account_info(mint_a).unwrap();
    let escrow_bump = [escrow_state.bump];
    pinocchio_log::log!("this is the escrow bump: {}", &escrow_bump);
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&escrow_bump),
    ];
    let seeds = Signer::from(&seed);

    pinocchio_token::instructions::TransferChecked {
        amount: escrow_state.amount_to_give(),
        authority: escrow_account,
        decimals: mint_a_account.decimals(),
        from: escrow_ata,
        mint: mint_a,
        to: taker_ata_a,
    }
    .invoke_signed(&[seeds])?;

    // close escrow ata
    CloseAccount {
        account: escrow_ata,
        authority: escrow_account,
        destination: maker,
    }
    .invoke_signed(&[Signer::from(&seed)])?;

    Ok(())
}
