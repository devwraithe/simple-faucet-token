use solana_program::program_error::ProgramError;

pub enum FaucetInstruction {
    RequestTokens,
}

impl FaucetInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, _rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match variant {
            0 => Self::RequestTokens,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}