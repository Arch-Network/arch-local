use arch_program::program_error::ProgramError;

#[derive(Debug, Clone, Copy)]
pub enum SwapError {
    InsufficientLiquidity,
    SlippageError,
    InvalidInput,
    RateManipulation,
    MarketConditionError,
}

impl From<SwapError> for ProgramError {
    fn from(e: SwapError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

