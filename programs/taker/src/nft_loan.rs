use anchor_lang::prelude::Pubkey;
use solana_program::clock::UnixTimestamp;

use crate::{utils, DerivedAccountIdentifier, NFTLoan, TakerError};
use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use fehler::{throw, throws};

impl DerivedAccountIdentifier for NFTLoan {
    const SEED: &'static [u8] = b"TakerNFTLoan";
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub enum LoanState {
    Active,
    Liquidated,
    Repayed,
    Finalized, // The loan is repayed and the nft is withdrawn
}

impl NFTLoan {
    #[throws(ProgramError)]
    pub fn start_borrow<'info>(
        program_id: &Pubkey,
        loan_id: &Pubkey,
        mint: &Pubkey,
        borrower_wallet: &AccountInfo<'info>,
        lender_wallet: &AccountInfo<'info>,
        loan_account: &AccountInfo<'info>,
        rent: &Sysvar<'info, Rent>,
        system: &AccountInfo<'info>,
        cash: u64,
        start: UnixTimestamp,
        length: i64,
    ) -> ProgramAccount<'info, Self> {
        let (_, bump) = Self::get_address_with_bump(
            program_id,
            mint,
            borrower_wallet.key,
            lender_wallet.key,
            loan_id,
        );

        Self::verify_address(
            program_id,
            mint,
            borrower_wallet.key,
            lender_wallet.key,
            loan_id,
            bump,
            loan_account.key,
        )?;

        // Do not reuse the loan record
        // TODO: deallocate the loan record and give lamports back to the user.
        if crate::utils::is_account_allocated(loan_account) {
            throw!(TakerError::LoanAlreadyExist);
        }

        let instance = NFTLoan {
            cash,
            started_at: start,
            expired_at: start + length,
            state: LoanState::Active,
        };

        let acc_size = 8 + instance
            .try_to_vec()
            .map_err(|_| ProgramError::Custom(1))?
            .len() as u64;

        let seeds_with_bump: &[&[_]] = &[
            Self::SEED,
            &mint.to_bytes(),
            &borrower_wallet.key.to_bytes(),
            &lender_wallet.key.to_bytes(),
            &loan_id.to_bytes(),
            &[bump],
        ];

        utils::create_derived_account_with_seed(
            program_id,
            borrower_wallet,
            loan_account,
            seeds_with_bump,
            acc_size,
            &rent,
            &system,
        )?;

        {
            let mut data = loan_account.try_borrow_mut_data()?;
            let mut cursor = std::io::Cursor::new(&mut **data);
            instance.try_serialize(&mut cursor)?;
        }

        let loan_account = ProgramAccount::try_from(loan_account)?;

        loan_account
    }

    #[throws(TakerError)]
    pub fn repay(&mut self) {
        self.state = LoanState::Repayed;
    }

    #[throws(TakerError)]
    pub fn liquidate(&mut self) {
        self.state = LoanState::Liquidated;
    }

    // An program derived account that stores nft loan
    // The address of the account is computed as follow:
    // address = find_program_address([NFTLoan::SEED, nft_mint_address, borrower_wallet_address, lender_wallet_address, loan_id], program_id)
    // only the taker_contract_address can change the data in this account
    pub fn get_address(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        borrower_wallet: &Pubkey,
        lender_wallet: &Pubkey,
        loan_id: &Pubkey,
    ) -> Pubkey {
        Self::get_address_with_bump(
            program_id,
            nft_mint,
            borrower_wallet,
            lender_wallet,
            loan_id,
        )
        .0
    }

    pub(crate) fn get_address_with_bump(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        borrower_wallet: &Pubkey,
        lender_wallet: &Pubkey,
        loan_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::SEED,
                &nft_mint.to_bytes(),
                &borrower_wallet.to_bytes(),
                &lender_wallet.to_bytes(),
                &loan_id.to_bytes(),
            ],
            program_id,
        )
    }

    #[throws(ProgramError)]
    pub fn verify_address(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        borrower_wallet: &Pubkey,
        lender_wallet: &Pubkey,
        loan_id: &Pubkey,
        bump: u8,
        address: &Pubkey,
    ) {
        let addr = Pubkey::create_program_address(
            &[
                Self::SEED,
                &nft_mint.to_bytes(),
                &borrower_wallet.to_bytes(),
                &lender_wallet.to_bytes(),
                &loan_id.to_bytes(),
                &[bump],
            ],
            program_id,
        )?;

        if &addr != address {
            throw!(TakerError::NFTLoanAddressNotCorrect);
        }
    }
}
