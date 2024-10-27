use borsh::{BorshDeserialize, BorshSerialize};
use arch_program::pubkey::Pubkey;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct LiquidityParams {
    pub liquidity_amount: u64, 
    pub token_a: Pubkey,       
    pub token_b: Pubkey,       
    pub token_a_amount: u64,   
    pub token_b_amount: u64,   
    pub yield_accumulated: u64, 
    pub last_yield_update_time: u64, 
}

impl LiquidityParams {
    // Constructor function to initialize a new liquidity pool
    pub fn new(initial_token_a: u64, initial_token_b: u64) -> Self {
        LiquidityParams {
            liquidity_amount: initial_token_a + initial_token_b, 
            token_a: Pubkey::new_unique(),                     
            token_b: Pubkey::new_unique(),                     
            token_a_amount: initial_token_a,
            token_b_amount: initial_token_b,
            yield_accumulated: 0,
            last_yield_update_time: 0,
        }
    }

    // Method to get the current liquidity amount
    pub fn get_liquidity_amount(&self) -> u64 {
        self.liquidity_amount
    }

    pub fn get_token_details(&self) -> (u64, u64, u64) {
        (self.token_a_amount, self.token_b_amount, self.liquidity_amount)
    }
}

pub struct LimitOrder {
    pub owner: Pubkey,
    pub token_pair: (Pubkey, Pubkey),
    pub amount: u64,
    pub price: u64,
    pub order_type: OrderType,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum OrderStatus {
    Open,
    Executed,
    Cancelled,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Vault {
    pub owner: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}
