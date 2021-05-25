use anchor_lang::prelude::Pubkey;

use crate::{utils, DerivedAccountIdentifier, NFTListing, TakerError};
use anchor_lang::prelude::*;
use fehler::{throw, throws};

impl DerivedAccountIdentifier for NFTListing {
    const SEED: &'static [u8] = b"TakerNFTListing";
}

impl NFTListing {
    #[throws(ProgramError)]
    pub fn ensure<'info>(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        user_wallet: &AccountInfo<'info>,
        listing: &AccountInfo<'info>,
        rent: &Sysvar<'info, Rent>,
        system_program: &AccountInfo<'info>,
    ) -> ProgramAccount<'info, NFTListing> {
        let (_, bump) = Self::get_address_with_bump(program_id, nft_mint, user_wallet.key);

        Self::verify_address(program_id, nft_mint, user_wallet.key, bump, listing.key)?;

        if !crate::utils::is_account_allocated(listing) {
            let instance = NFTListing {
                count: 0,
                available: 0,
            };

            let acc_size = 8 + instance
                .try_to_vec()
                .map_err(|_| ProgramError::Custom(1))?
                .len() as u64;

            let seeds_with_bump: &[&[_]] = &[
                Self::SEED,
                &nft_mint.to_bytes(),
                &user_wallet.key.to_bytes(),
                &[bump],
            ];

            utils::create_derived_account_with_seed(
                program_id,
                user_wallet,
                listing,
                seeds_with_bump,
                acc_size,
                &rent,
                &system_program,
            )?;

            {
                let mut data = listing.try_borrow_mut_data()?;
                let mut cursor = std::io::Cursor::new(&mut **data);
                instance.try_serialize(&mut cursor)?;
            }
        }

        ProgramAccount::try_from(listing)?
    }

    pub fn deposit(&mut self, count: u64) {
        self.count += count;
        self.available += count;
    }

    #[throws(TakerError)]
    pub fn withdraw(&mut self, count: u64) {
        if count > self.available {
            throw!(TakerError::NFTOverdrawn)
        }

        self.count -= count;
        self.available -= count;
    }

    #[throws(TakerError)]
    pub fn liquidate(&mut self, count: u64) {
        if count > self.available {
            throw!(TakerError::NFTOverdrawn)
        }

        self.count -= count;
        self.available -= count;
    }

    #[throws(TakerError)]
    pub fn borrow_success(&mut self) {
        if self.available <= 0 {
            throw!(TakerError::EmptyNFTReserve)
        }

        self.available -= 1;
    }

    pub fn repay_success(&mut self) {
        self.available += 1;

        assert!(self.available <= self.count);
    }

    // An program derived account that stores nft listing
    // The address of the account is computed as follow:
    // address = find_program_address([NFTListing::SEED, nft_mint_address, user_wallet_address], program_id)
    // only the taker_contract_address can change the data in this account
    pub fn get_address(program_id: &Pubkey, nft_mint: &Pubkey, wallet: &Pubkey) -> Pubkey {
        Self::get_address_with_bump(program_id, nft_mint, wallet).0
    }

    pub(crate) fn get_address_with_bump(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        wallet: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::SEED, &nft_mint.to_bytes(), &wallet.to_bytes()],
            program_id,
        )
    }

    #[throws(ProgramError)]
    pub fn verify_address(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        wallet: &Pubkey,
        bump: u8,
        address: &Pubkey,
    ) {
        let addr = Pubkey::create_program_address(
            &[
                Self::SEED,
                &nft_mint.to_bytes(),
                &wallet.to_bytes(),
                &[bump],
            ],
            program_id,
        )?;

        if &addr != address {
            throw!(TakerError::NFTListingAddressNotCorrect);
        }
    }
}
