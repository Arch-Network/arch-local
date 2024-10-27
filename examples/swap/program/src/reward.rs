// reward.rs

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RewardParams {
    pub total_staked: u64,  
    pub total_rewards: u64, 
    pub reward_rate: f64,   
}

impl RewardParams {
    // Constructor to initialize RewardParams
    pub fn new() -> Self {
        RewardParams {
            total_staked: 0,
            total_rewards: 0,
            reward_rate: 0.0,
        }
    }

    // Update reward parameters based on total staked tokens
    pub fn update_reward_rate(&mut self) {
        
        let total_staked_f64 = self.total_staked as f64;

        
        let adjusted_staked = total_staked_f64 / 10.0 + 1.0;
        self.reward_rate = 5.0 * self.reward_rate / adjusted_staked;
    }

    // Calculate rewards based on the stake amount
    pub fn calculate_reward(&self, stake_amount: u64) -> f64 {
        stake_amount as f64 * self.reward_rate
    }

    // Add rewards to the total rewards
    pub fn add_rewards(&mut self, amount: f64) {
        self.total_rewards = (self.total_rewards as f64 + amount).round() as u64;
    }

    // Method to distribute trading fees as yields
    pub fn distribute_yield(&mut self, trading_fee: u64) {
        let yield_to_distribute = (trading_fee as f64 * 0.8) as u64; // 80% of trading fees go to yield
        self.total_rewards += yield_to_distribute;
    }
}
