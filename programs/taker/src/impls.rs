use crate::utils;
use crate::{AccountsInitialize, TakerContract};
use anchor_lang::prelude::*;

type Result<T> = std::result::Result<T, ProgramError>;

impl TakerContract {
    pub fn new<'info>(
        ctx: &Context<AccountsInitialize<'info>>,
        seed: &[u8],
        bump: u8,
    ) -> Result<ProgramAccount<'info, Self>> {
        let accounts = &ctx.accounts;
        let this = &accounts.this;

        let instance = Self {
            seed: seed.to_vec(),
            bump_seed: bump,
            authority: *accounts.authority.key,
            tkr_mint: *accounts.tkr_mint.key,
            tai_mint: *accounts.tai_mint.key,
            dai_mint: *accounts.dai_mint.key,
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
        utils::create_rent_exempt_account(
            ctx.program_id, // The program ID of Taker Contract
            &accounts.authority,
            &this,
            &[&seed[..], &[bump]],
            ctx.program_id,
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
}
