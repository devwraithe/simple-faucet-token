use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum FaucetInstruction {
    Initialize { distribution_amount: u64 }, // instruction variant with struct-like pattern
    RequestTokens,                           // instruction variant
    ReplenishTokens { replenish_amount: u64 },
}

#[derive(BorshDeserialize)]
struct InitializePayload {
    distribution_amount: u64,
}
#[derive(BorshDeserialize)]
struct ReplenishTokensPayload {
    replenish_amount: u64,
}

impl FaucetInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match variant {
            0 => {
                let payload = InitializePayload::try_from_slice(rest).unwrap();
                Self::Initialize {
                    // same as FaucetInstruction::Initialize {
                    distribution_amount: payload.distribution_amount,
                }
            }
            1 => Self::RequestTokens,
            2 => {
                let payload = ReplenishTokensPayload::try_from_slice(rest).unwrap();
                Self::ReplenishTokens {
                    replenish_amount: payload.replenish_amount,
                }
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
