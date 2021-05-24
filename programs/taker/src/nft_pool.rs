use crate::{utils, AccountsInitialize, NFTPool, TakerError};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use fehler::{throw, throws};

type Result<T> = std::result::Result<T, ProgramError>;

impl NFTPool {
    pub fn new<'info>(
        ctx: &Context<AccountsInitialize<'info>>,
        bump: u8,
    ) -> Result<ProgramAccount<'info, Self>> {
        let accounts = &ctx.accounts;
        let this = &accounts.pool;

        let instance = Self {
            bump_seed: bump,
            authority: *accounts.pool_owner.key,
            tkr_mint: *accounts.tkr_mint.to_account_info().key,
            tai_mint: *accounts.tai_mint.to_account_info().key,
            dai_mint: *accounts.dai_mint.to_account_info().key,
            deposit_incentive: 100,
            max_loan_duration: 30,
            // 5%
            service_fee_rate: 500,
            // 1%
            interest_rate: 100,
            // Total number of loans generated
            total_num_loans: 0,
        };

        let acc_size = 8 + instance
            .try_to_vec()
            .map_err(|_| ProgramError::Custom(1))?
            .len() as u64;

        // allocate the space for the contract account
        utils::create_derived_account_with_seed(
            ctx.program_id, // The program ID of Taker Contract
            &accounts.pool_owner,
            &this,
            &[&[bump]],
            acc_size,
            &accounts.rent,
            &accounts.system,
        )?;

        // let the data borrow invalid after exiting the scope. Otherwise can cannot borrow it again in the ProgramAccount::try_from
        {
            let mut data = this.try_borrow_mut_data()?;
            let mut cursor = std::io::Cursor::new(&mut **data);
            instance.try_serialize(&mut cursor)?;
        }

        let this = ProgramAccount::try_from(this)?;
        Ok(this)
    }

    pub fn ensure_pool_nft_account<'info>(
        pool: &ProgramAccount<'info, NFTPool>,
        nft_mint: &CpiAccount<'info, Mint>,
        pool_nft_account: &AccountInfo<'info>,
        user_wallet_account: &AccountInfo<'info>,
        ata_program: &AccountInfo<'info>,
        spl_program: &AccountInfo<'info>,
        system: &AccountInfo<'info>,
        rent: &Sysvar<'info, Rent>,
    ) -> Result<()> {
        if !utils::is_account_allocated(pool_nft_account) {
            utils::create_associated_token_account(
                &pool.to_account_info(),
                user_wallet_account,
                nft_mint,
                pool_nft_account,
                ata_program,
                spl_program,
                system,
                rent,
            )?;
        }

        Ok(())
    }

    pub fn ensure_user_tkr_account<'info>(
        user_wallet_account: &AccountInfo<'info>,
        tkr_mint: &CpiAccount<'info, Mint>,
        user_tkr_account: &AccountInfo<'info>,
        ata_program: &AccountInfo<'info>,
        spl_program: &AccountInfo<'info>,
        system: &AccountInfo<'info>,
        rent: &Sysvar<'info, Rent>,
    ) -> Result<()> {
        if !utils::is_account_allocated(user_tkr_account) {
            utils::create_associated_token_account(
                user_wallet_account,
                user_wallet_account,
                tkr_mint,
                user_tkr_account,
                ata_program,
                spl_program,
                system,
                rent,
            )?;
        }

        Ok(())
    }

    // pub fn deposit_nft(program_id: &Pubkey, src: &AccountInfo, dst: AccountInfo) {
    //     anchor_spl::token::transfer(
    //         CpiContext::new_with_signer(
    //             spl_program.clone(),
    //             anchor_spl::token::Transfer {
    //                 from: tkr_src.to_account_info(),
    //                 to: tkr_dst.clone(),
    //                 authority: pool.to_account_info(),
    //             },
    //             &[&[&[pool.bump_seed]]],
    //         ),
    //         pool.deposit_incentive,
    //     )?;
    // }

    // An program derived account that stores nft listing
    // The address of the account is computed as follow:
    // address = find_program_address([nft_mint_address, user_wallet_address], program_id)
    // only the taker_contract_address can change the data in this account
    pub fn get_nft_listing_address(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        wallet: &Pubkey,
    ) -> Pubkey {
        Self::get_nft_listing_address_with_bump(program_id, nft_mint, wallet).0
    }

    pub(crate) fn get_nft_listing_address_with_bump(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        wallet: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[&nft_mint.to_bytes(), &wallet.to_bytes()], program_id)
    }

    #[throws(ProgramError)]
    pub fn verify_nft_listing_address(
        program_id: &Pubkey,
        nft_mint: &Pubkey,
        wallet: &Pubkey,
        bump: u8,
        listing_address: &Pubkey,
    ) {
        let addr = Pubkey::create_program_address(
            &[&nft_mint.to_bytes(), &wallet.to_bytes(), &[bump]],
            program_id,
        )?;

        if &addr != listing_address {
            throw!(TakerError::NFTListingAddressNotCorrect);
        }
    }
}
