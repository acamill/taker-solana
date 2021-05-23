use std::str::FromStr;
use derive_more::Deref;

#[derive(Debug, Deref)]
pub struct Keypair(#[deref] pub solana_sdk::signature::Keypair, String);

impl FromStr for Keypair {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            solana_sdk::signature::Keypair::from_base58_string(s),
            s.into(),
        ))
    }
}

impl Clone for Keypair {
    fn clone(&self) -> Self {
        self.1.parse().unwrap()
    }
}
