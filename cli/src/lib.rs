use derive_more::Deref;
use serde::Deserialize;
use serde_json::from_reader;
use solana_sdk::pubkey::Pubkey;
use std::{fs::File, str::FromStr};

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
#[derive(Deserialize)]
struct IDL {
    metadata: Metadata,
}

#[derive(Deserialize)]
struct Metadata {
    address: String,
}

pub fn load_program_from_idl() -> Pubkey {
    let f = File::open("target/idl/taker.json").unwrap();
    let m: IDL = from_reader(f).unwrap();
    m.metadata.address.parse().unwrap()
}
