use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum LiquidityInstruction {
    AddLiquidity {
        token_a_amount: u64,
        token_b_amount: u64,
    },
    RemoveLiquidity {
        token_a_amount: u64,
        token_b_amount: u64,
    },
    GetLiquidityAmount,
    SwapTokens {
        token_a_amount: u64,
        min_token_b_amount: u64,
    },
    StakeTokens {
        stake_amount: u64,
    },
    UnstakeTokens {
        unstake_amount: u64,
    },
    ClaimRewards,
}
