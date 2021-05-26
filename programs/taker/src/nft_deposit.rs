use anchor_lang::prelude::Pubkey;
use solana_program::clock::UnixTimestamp;

use crate::{utils, DerivedAccountIdentifier, NFTDeposit, TakerError};
use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use fehler::{throw, throws};

impl DerivedAccountIdentifier for NFTDeposit {
    const SEED: &'static [u8] = b"TakerNFTDeposit";
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub enum DepositState {
    PendingLoan, // Loan hasn't happened yet and the NFT is in the pool
    Withdrawn,   // Loan did not happen and the NFT is withdrawn by the borrower
    LoanActive {
        total_amount: u64,
        borrowed_amount: u64,      // amount of dai
        started_at: UnixTimestamp, // in seconds
        expired_at: UnixTimestamp, // in seconds
        lender: Pubkey,
    }, // Loan is active
    LoanLiquidated, // Loan liquidated and the NFT is withdrawn by the lender
    LoanRepayed {
        lender_withdrawable: u64,
        lender: Pubkey,
    }, // Loan repayed and the NFT is withdrawn by the borrower
}

impl NFTDeposit {
    #[throws(ProgramError)]
    pub fn deposit<'info>(
        program_id: &Pubkey,
        deposit_id: &Pubkey,
        mint: &Pubkey,
        borrower_wallet: &AccountInfo<'info>,
        loan_account: &AccountInfo<'info>,
        rent: &Sysvar<'info, Rent>,
        system_program: &AccountInfo<'info>,
    ) -> ProgramAccount<'info, Self> {
        let (_, bump) =
            Self::get_address_with_bump(program_id, mint, borrower_wallet.key, deposit_id);

        Self::verify_address(
            program_id,
            mint,
            borrower_wallet.key,
            deposit_id,
            bump,
            loan_account.key,
        )?;

        // Do not reuse the loan record
        // TODO: deallocate the loan record and give lamports back to the user.
        if crate::utils::is_account_allocated(loan_account) {
            throw!(TakerError::LoanAlreadyExist);
        }

        let instance = NFTDeposit {
            deposit_id: *deposit_id,
            state: DepositState::PendingLoan,
        };

        let seeds_with_bump: &[&[_]] = &[
            Self::SEED,
            &mint.to_bytes(),
            &borrower_wallet.key.to_bytes(),
            &deposit_id.to_bytes(),
            &[bump],
        ];

        utils::create_derived_account_with_seed(
            program_id,
            borrower_wallet,
            loan_account,
            seeds_with_bump,
            Self::account_size() as u64,
            &rent,
            &system_program,
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
    pub fn withdraw(&mut self) {
        use DepositState::*;

        match self.state {
            PendingLoan => self.state = DepositState::Withdrawn,
            Withdrawn | LoanRepayed { .. } => throw!(TakerError::NFTAlreadyWithdrawn),
            LoanActive { .. } | LoanLiquidated => throw!(TakerError::NFTLocked),
        }
    }

    #[throws(TakerError)]
    pub fn start_borrow(
        &mut self,
        lender: Pubkey,
        total_amount: u64,
        borrowed_amount: u64,
        start: UnixTimestamp,
        length: i64,
    ) {
        if !matches!(self.state, DepositState::PendingLoan) {
            throw!(TakerError::BorrowAlreadyStarted)
        }

        assert!(total_amount >= borrowed_amount);
        self.state = DepositState::LoanActive {
            lender,
            total_amount,
            borrowed_amount,            // amount of dai
            started_at: start,          // in seconds
            expired_at: start + length, // in seconds
        };
    }

    #[throws(TakerError)]
    pub fn repay(&mut self, lender_withdrawable: u64) {
        match self.state {
            DepositState::LoanActive { lender, .. } => {
                self.state = DepositState::LoanRepayed {
                    lender_withdrawable,
                    lender,
                }
            }
            _ => {
                throw!(TakerError::LoanNotActive)
            }
        }
    }

    #[throws(TakerError)]
    pub fn liquidate(&mut self) {
        match self.state {
            DepositState::LoanActive { .. } => {
                self.state = DepositState::LoanLiquidated;
            }
            _ => {
                throw!(TakerError::LoanNotActive)
            }
        }
    }

    // An program derived account that stores nft loan
    // The address of the account is computed as follow:
    // address = find_program_address([NFTLoan::SEED, nft_mint_address, borrower_wallet_address, loan_id], program_id)
    // only the taker_contract_address can change the data in this account
    pub fn get_address(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        borrower_wallet: &Pubkey,
        deposit_id: &Pubkey,
    ) -> Pubkey {
        Self::get_address_with_bump(program_id, nft_mint, borrower_wallet, deposit_id).0
    }

    pub(crate) fn get_address_with_bump(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        borrower_wallet: &Pubkey,
        deposit_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::SEED,
                &nft_mint.to_bytes(),
                &borrower_wallet.to_bytes(),
                &deposit_id.to_bytes(),
            ],
            program_id,
        )
    }

    #[throws(ProgramError)]
    pub fn verify_address(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        borrower_wallet: &Pubkey,
        deposit_id: &Pubkey,
        bump: u8,
        address: &Pubkey,
    ) {
        let addr = Pubkey::create_program_address(
            &[
                Self::SEED,
                &nft_mint.to_bytes(),
                &borrower_wallet.to_bytes(),
                &deposit_id.to_bytes(),
                &[bump],
            ],
            program_id,
        )?;

        if &addr != address {
            throw!(TakerError::NFTLoanAddressNotCorrect);
        }
    }

    fn account_size() -> usize {
        // Borsh does not support vary size structure.
        // Pick the largest variant so that we are safe
        let largest_instance = NFTDeposit {
            deposit_id: Pubkey::new(&[0u8; 32]),
            state: DepositState::LoanActive {
                total_amount: 0,
                borrowed_amount: 0, // amount of dai
                started_at: 0,      // in seconds
                expired_at: 0,      // in seconds
                lender: Pubkey::new(&[0u8; 32]),
            },
        };

        let acc_size = 8 + largest_instance.try_to_vec().unwrap().len();
        acc_size
    }
}