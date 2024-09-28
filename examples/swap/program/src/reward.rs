// reward.rs

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RewardParams {
    pub total_staked: u64,  // Total amount of tokens staked in the system
    pub total_rewards: u64, // Total rewards distributed
    pub reward_rate: f64,   // Reward rate per staked token (dynamic)
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
        // Example formula: reward rate decreases as total staked increases
        let total_staked_f64 = self.total_staked as f64;

        // Example formula: reward rate decreases as total staked increases
        // Ensure that the formula is valid for floating-point arithmetic
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
}
