#![forbid(unsafe_code)]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

use solana_program;

solana_program::declare_id!("dYq3uiw4k91nbzZS7mAyVeV1o1XLDnepAiiyCbVgkpN");
