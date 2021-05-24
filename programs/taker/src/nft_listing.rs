use anchor_lang::prelude::Pubkey;

use crate::utils;
use crate::NFTListing;
use anchor_lang::prelude::*;
use fehler::throws;

impl NFTListing {
    #[throws(ProgramError)]
    pub fn ensure<'info>(
        program_id: &Pubkey,
        mint: &Pubkey,
        wallet: &AccountInfo<'info>,
        listing: &AccountInfo<'info>,
        rent: &Sysvar<'info, Rent>,
        system: &AccountInfo<'info>,
    ) -> ProgramAccount<'info, NFTListing> {
        let (_, bump) = utils::get_nft_listing_address_with_bump(program_id, mint, wallet.key);

        utils::verify_nft_listing_address(program_id, mint, wallet.key, bump, listing.key)?;

        if !crate::utils::is_account_allocated(listing) {
            let instance = NFTListing { count: 0 };

            let acc_size = 8 + instance
                .try_to_vec()
                .map_err(|_| ProgramError::Custom(1))?
                .len() as u64;

            let seeds_with_bump_for_listing: &[&[_]] =
                &[&mint.to_bytes(), &wallet.key.to_bytes(), &[bump]];

            utils::create_derived_account_with_seed(
                program_id,
                wallet,
                listing,
                seeds_with_bump_for_listing,
                acc_size,
                &rent,
                &system,
            )?;

            {
                let mut data = listing.try_borrow_mut_data()?;
                let mut cursor = std::io::Cursor::new(&mut **data);
                instance.try_serialize(&mut cursor)?;
            }
        }

        ProgramAccount::try_from(listing)?
    }
}
