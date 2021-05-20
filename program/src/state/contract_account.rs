//! State transition types

use crate::instruction::MAX_SIGNERS;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use borsh::schema_helpers::try_to_vec_with_schema;
use borsh::{BorshDeserialize, BorshSerialize};
use num_enum::TryFromPrimitive;
use solana_program::borsh;
use solana_program::{
    program_error::ProgramError,
    program_option::COption,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

struct PackedPubKey([u32; 4]);
impl From<PubKey> for PackedPubKey {
    fn from(pk: PubKey) -> Self {
        Self(pk.to_aligned_bytes())
    }
}

enum LoanStat {
    Active,
    Repaid,
    Liquidated,
}

pub struct Loan {
    // A unique identifier for a loan
    loan_id: u64,
    borrow_amount: uint256,
    interest: uint256,
    nft_contract: address,
    start_time: u64,
    borrower: Pubkey,
    lender: PubKey,
    stat: LoanStat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct ContractAccount {
    pub tkr_mint: PackedPubKey,
    pub tai_mint: PackedPubKey,
    pub dai_mint: PackedPubKey,
    pub is_initialized: bool,
    pub nft_ownership: HashMap<Pubkey, Pubkey>,
    pub nft_price_at_loan: HashMap<Pubkey, u64>,
    pub deposit_incentive: u64,
    pub max_loan_duration: u64,
    pub service_fee_rate: u64,
    pub interest_rate: u64,
    // Total number of loans generated
    pub total_num_loans: u64,
}

impl Sealed for ContractAccount {}

impl IsInitialized for ContractAccount {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl ContractAccount {
    pub fn new(tkr_mint: Pubkey, tai_mint: Pubkey, dai_mint: Pubkey) -> Self {
        Self {
            tkr_mint,
            tai_mint,
            dai_mint,
            is_initialized: true,
            nft_ownership: HashMap::new(),
            nft_price_at_loan: HashMap::new(),
            deposit_incentive: 100,
            max_loan_duration: 30,
            // 5%
            service_fee_rate: 500,
            // 1%
            interest_rate: 100,
            total_num_loans: 0,
        }
    }
}

impl Pack for ContractAccount {
    const LEN: usize = 9437184; // 9MB
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let contract_account = borsh::try_from_slice_unchecked(src)?;
        Ok(contract_account)
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let v = try_to_vec_with_schema(self).unwrap();
        assert!(v.len() <= dst.len());

        dst.copy_from_slice(&v);
    }
}

#[cfg(test)]
mod test {
    use super::ContractAccount;
    use solana_program::borsh;

    #[test]
    fn check_account_size() {
        assert!(borsh::get_packed_len::<ContractAccount>() <= ContractAccount::LEN)
    }
}
