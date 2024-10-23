use borsh::{BorshDeserialize, BorshSerialize};
use arch_program::pubkey::Pubkey;
use crate::operations::{execute_limit_orders, LimitOrder, OrderStatus, OrderType};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct LimitOrderBook {
    orders: Vec<LimitOrder>,
}

impl LimitOrderBook {
    pub fn new() -> Self {
        LimitOrderBook { orders: Vec::new() }
    }

    pub fn add_order(&mut self, order: LimitOrder) {
        self.orders.push(order);
    }

    pub fn process_orders(&mut self, current_market_prices: &[(Pubkey, Pubkey, u64)]) {
        execute_limit_orders(&mut self.orders, current_market_prices);
    }
}

