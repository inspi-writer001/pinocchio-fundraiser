use crate::state::Fundraiser;

#[cfg(test)]
mod tests {

    use std::{io::Error, path::PathBuf};

    use litesvm::LiteSVM;
    use litesvm_token::{
        spl_token::{
            self,
            solana_program::{msg, rent::Rent, sysvar::SysvarId},
        },
        CreateAssociatedTokenAccount, CreateMint, MintTo,
    };

    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_associated_token_account::solana_program::program_pack::Pack;

    use crate::instructions::InitializeFundraiser;

    const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID); //"CntDHuHyUa1sEyLEYoHbrYdzM2G4VeDHSdQjQXXdRh6E";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    fn program_id() -> Pubkey {
        PROGRAM_ID
    }

    fn setup() -> (LiteSVM, ReusableState) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Load program SO file
        msg!("The path is!! {}", env!("CARGO_MANIFEST_DIR"));

        let bytes = include_bytes!("../../target/deploy/pinocchio_fundraising.so");
        svm.add_program(program_id(), bytes);

        let mint = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Mint A: {}", mint);

        // Create the maker's associated token account for Mint A
        let maker_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint)
            .owner(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Maker ATA A: {}\n", maker_ata);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let fundraiser = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID,
        );
        msg!("Fundraiser PDA: {}\n", fundraiser.0);

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = spl_associated_token_account::get_associated_token_address(
            &fundraiser.0, // owner will be the escrow PDA
            &mint,         // mint
        );

        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let asspciated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        let reusable_state = ReusableState {
            mint,
            ata_program: asspciated_token_program,
            token_program,
            system_program,
            vault,
            maker: payer,
            maker_ata,
            fundraiser,
        };
        (svm, reusable_state)
    }

    pub struct ReusableState {
        pub mint: Pubkey,
        pub maker_ata: Pubkey,
        pub vault: Pubkey,
        pub ata_program: Pubkey,
        pub token_program: Pubkey,
        pub system_program: Pubkey,
        pub fundraiser: (Pubkey, u8),
        pub maker: Keypair,
    }

    pub fn create_fundraiser(svm: &mut LiteSVM, state: &ReusableState) -> Result<(), Error> {
        let mint = state.mint;
        let payer = &state.maker;
        let maker_ata = state.maker_ata;
        let vault = state.vault;
        let system_program = state.system_program;
        let token_program = state.token_program;
        let ata_program = state.ata_program;
        let fundraiser = state.fundraiser;

        MintTo::new(svm, &payer, &mint, &maker_ata, 1_000_000_000)
            .send()
            .unwrap();
        litesvm_token::CreateAssociatedTokenAccount::new(svm, payer, &mint)
            .owner(&fundraiser.0)
            .token_program_id(&TOKEN_PROGRAM_ID)
            .send()
            .unwrap();

        let amount_to_receive: u64 = 100_000_000; // 100 tokens with 6 decimal places
        let amount_to_give: u64 = 400_000_000; // 500 tokens with 6 decimal places

        let init_data_ix: InitializeFundraiser = InitializeFundraiser {
            amount_to_raise: amount_to_give,
            duration: amount_to_receive,
        };

        let init_data_ser = init_data_ix.to_bytes();

        let make_data = [
            vec![crate::instructions::FundraisingInstructions::Initialize as u8],
            init_data_ser, // Discriminator for "Make" instruction
        ]
        .concat();

        msg!("here's the fundraiser from client: {}", fundraiser.0);

        let init_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(mint, false),
                AccountMeta::new(fundraiser.0, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(ata_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: make_data,
        };

        let message = Message::new(&[init_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();
        msg!("tx logs: {:#?}", tx.logs);
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);

        Ok(())
    }

    // pub fn take(svm: &mut LiteSVM, state: &ReusableState) -> Result<(Pubkey, Pubkey), Error> {
    //     let mint_a = state.mint_a;
    //     let maker = &state.maker;
    //     let maker_ata_a = state.maker_ata_a;
    //     let maker_ata_b = state.maker_ata_b;
    //     let mint_b = state.mint_b;
    //     let vault = state.vault;
    //     let system_program = state.system_program;
    //     let token_program = state.token_program;
    //     let ata_program = state.ata_program;
    //     let escrow = state.escrow;

    //     let taker = Keypair::new();
    //     svm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
    //         .expect("Airdrop failed");

    //     let taker_ata_a = CreateAssociatedTokenAccount::new(svm, &taker, &mint_a)
    //         .owner(&taker.pubkey())
    //         .send()
    //         .unwrap();
    //     let taker_ata_b = CreateAssociatedTokenAccount::new(svm, &taker, &mint_b)
    //         .owner(&taker.pubkey())
    //         .send()
    //         .unwrap();
    //     msg!("Taker ATA A: {}\nTaker ATA B {}", taker_ata_a, taker_ata_b);

    //     MintTo::new(svm, &maker, &mint_b, &taker_ata_b, 1_000_000_000)
    //         .send()
    //         .unwrap();

    //     let take_data = [
    //         vec![crate::instructions::EscrowInstrctions::Take as u8], // Discriminator for "Take" instruction
    //     ]
    //     .concat();

    //     let take_ix = Instruction {
    //         program_id: program_id(),
    //         accounts: vec![
    //             AccountMeta::new(taker.pubkey(), true),
    //             AccountMeta::new(maker.pubkey(), false),
    //             AccountMeta::new(maker_ata_a, false),
    //             AccountMeta::new(maker_ata_b, false),
    //             AccountMeta::new(mint_a, false),
    //             AccountMeta::new(mint_b, false),
    //             AccountMeta::new(escrow.0, false),
    //             AccountMeta::new(taker_ata_a, false),
    //             AccountMeta::new(taker_ata_b, false),
    //             AccountMeta::new(vault, false),
    //             AccountMeta::new(system_program, false),
    //             AccountMeta::new(token_program, false),
    //             AccountMeta::new(ata_program, false),
    //             AccountMeta::new(Rent::id(), false),
    //         ],
    //         data: take_data,
    //     };

    //     let message = Message::new(&[take_ix], Some(&taker.pubkey()));
    //     let recent_blockhash = svm.latest_blockhash();

    //     let transaction = Transaction::new(&[&taker], message, recent_blockhash);

    //     // Send the transaction and capture the result
    //     let tx = svm.send_transaction(transaction).unwrap();
    //     msg!("tx logs: {:#?}", tx.logs);
    //     msg!("\n\nMake transaction sucessfull");
    //     msg!("CUs Consumed: {}", tx.compute_units_consumed);

    //     Ok((taker_ata_a, taker_ata_b))
    // }

    #[test]
    pub fn test_init_instruction() {
        let (mut svm, state) = setup();

        let program_id = program_id();

        assert_eq!(program_id, PROGRAM_ID);
        create_fundraiser(&mut svm, &state).unwrap();

        let fundraiser_state = svm.get_account(&state.fundraiser.0).unwrap();

        let maker_deserialized_ata =
            bytemuck::try_from_bytes::<crate::state::Fundraiser>(&fundraiser_state.data).unwrap();
        // spl_token::state::Account::unpack(fundraiser_state.data.as_slice()).unwrap();
        msg!(
            "new user token bump: {:#?}",
            maker_deserialized_ata.amount_to_raise
        );
    }
}
