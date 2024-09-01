use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct FaucetState {
    pub admin: Pubkey,
    pub distribution_amount: u64,
}
