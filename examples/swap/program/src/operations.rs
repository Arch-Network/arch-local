use arch_program::{account::AccountInfo, program_error::ProgramError};

use crate::calculate_swap_amount;
pub use crate::LiquidityParams;
pub use crate::RewardParams;

pub fn add_liquidity(
    liquidity_account: &AccountInfo,
    liquidity_params: &mut LiquidityParams,
    token_a: Pubkey,
    token_b: Pubkey,
    token_a_amount: u64,
    token_b_amount: u64,
) -> Result<(), ProgramError> {
    // Ensure the tokens match those in the liquidity_params
    if liquidity_params.token_a != token_a || liquidity_params.token_b != token_b {
        return Err(ProgramError::Custom(506)); // Token mismatch error
    }
    liquidity_params.token_a_amount += token_a_amount;
    // Existing logic to add liquidity
    liquidity_params.token_a_amount += token_a_amount;
    liquidity_params.token_b_amount += token_b_amount;
    liquidity_params.liquidity_amount = liquidity_params.token_a_amount + liquidity_params.token_b_amount;
        liquidity_params.token_a_amount + liquidity_params.token_b_amount;
    // Serialize and save
    let serialized_data = borsh::to_vec(&*liquidity_params).map_err(|_| ProgramError::Custom(503))?;
    liquidity_account.data.borrow_mut().copy_from_slice(&serialized_data);
        borsh::to_vec(&*liquidity_params).map_err(|_| ProgramError::Custom(503))?;
    Ok(())
}


pub fn remove_liquidity(
    liquidity_account: &AccountInfo,
    liquidity_params: &mut LiquidityParams,
    token_a_amount: u64,
    token_b_amount: u64,
) -> Result<(), ProgramError> {
    let mut liquidity_data = liquidity_account
        .data
        .try_borrow_mut()
        .map_err(|_| ProgramError::Custom(502))?;
        .try_borrow_mut()
    // Remove the specified amounts of Token A and Token B from the pool
    liquidity_params.token_a_amount = liquidity_params
        .token_a_amount
        .saturating_sub(token_a_amount);
    liquidity_params.token_b_amount = liquidity_params
        .token_b_amount
        .saturating_sub(token_b_amount);
        .token_b_amount
    // Update total liquidity in the pool
    liquidity_params.liquidity_amount =
        liquidity_params.token_a_amount + liquidity_params.token_b_amount;
    liquidity_params.liquidity_amount =
    // Serialize updated liquidity params back to account data
    let serialized_data =
        borsh::to_vec(&*liquidity_params).map_err(|_| ProgramError::Custom(503))?;
    liquidity_data.copy_from_slice(&serialized_data);
        borsh::to_vec(&*liquidity_params).map_err(|_| ProgramError::Custom(503))?;
    Ok(())
}

pub fn swap_tokens(
    liquidity_account: &AccountInfo,
    liquidity_params: &mut LiquidityParams,
    token_a_amount: u64,
    min_token_b_amount: u64,
) -> Result<(), ProgramError> {
    let mut liquidity_data = liquidity_account
        .data
        .try_borrow_mut()
        .map_err(|_| ProgramError::Custom(502))?;

    // Calculate the amount of Token B that should be received
    let token_b_amount = calculate_swap_amount(
        liquidity_params.token_a_amount,
        liquidity_params.token_b_amount,
        token_a_amount,
    );

    if token_b_amount < min_token_b_amount {
        return Err(ProgramError::Custom(503)); // Slippage protection failed
    }

    // Update the liquidity pool
    liquidity_params.token_a_amount += token_a_amount;
    liquidity_params.token_b_amount -= token_b_amount;
    liquidity_params.liquidity_amount =
        liquidity_params.token_a_amount + liquidity_params.token_b_amount;

    // Serialize updated liquidity params back to account data
    let serialized_data =
        borsh::to_vec(&*liquidity_params).map_err(|_| ProgramError::Custom(504))?;
    liquidity_data.copy_from_slice(&serialized_data);

    Ok(())
}

pub fn swap_tokens(
    liquidity_account: &AccountInfo,
    liquidity_params: &mut LiquidityParams,
    token_a_amount: u64,
    min_token_b_amount: u64,
) -> Result<(), ProgramError> {
    let mut liquidity_data = liquidity_account
        .data
        .try_borrow_mut()
        .map_err(|_| ProgramError::Custom(502))?;
        .try_borrow_mut()
    // Calculate the amount of Token B that should be received
    let token_b_amount = calculate_swap_amount(
        liquidity_params.token_a_amount,
        liquidity_params.token_b_amount,
        token_a_amount,
    );
        token_a_amount,
    if token_b_amount < min_token_b_amount {
        return Err(ProgramError::Custom(503)); // Slippage protection failed
    }
        return Err(ProgramError::Custom(503)); // Slippage protection failed
    // Update the liquidity pool
    liquidity_params.token_a_amount += token_a_amount;
    liquidity_params.token_b_amount -= token_b_amount;
    liquidity_params.liquidity_amount =
        liquidity_params.token_a_amount + liquidity_params.token_b_amount;
    liquidity_params.liquidity_amount =
    // Serialize updated liquidity params back to account data
    let serialized_data =
        borsh::to_vec(&*liquidity_params).map_err(|_| ProgramError::Custom(504))?;
    liquidity_data.copy_from_slice(&serialized_data);
        borsh::to_vec(&*liquidity_params).map_err(|_| ProgramError::Custom(504))?;
    Ok(())
}


pub fn unstake_tokens(
    reward_account: &AccountInfo,
    reward_params: &mut RewardParams,
    unstake_amount: u64,
) -> Result<(), ProgramError> {
    let mut reward_data = reward_account
        .data
        .try_borrow_mut()
        .map_err(|_| ProgramError::Custom(502))?;

    // Update total staked amount
    reward_params.total_staked = reward_params.total_staked.saturating_sub(unstake_amount);

    // Update reward rate based on new total staked amount
    reward_params.update_reward_rate();

    // Serialize updated reward params back to account data
    let serialized_data = borsh::to_vec(&*reward_params).map_err(|_| ProgramError::Custom(503))?;
    reward_data.copy_from_slice(&serialized_data);

    Ok(())
}

pub fn claim_rewards(
    reward_account: &AccountInfo,
    reward_params: &mut RewardParams,
) -> Result<f64, ProgramError> {
    let mut reward_data = reward_account
        .data
        .try_borrow_mut()
        .map_err(|_| ProgramError::Custom(502))?;

    // Calculate the reward based on the total staked amount
    let reward_amount = reward_params.calculate_reward(reward_params.total_staked);

    // Update total rewards
    reward_params.add_rewards(reward_amount);

    // Serialize updated reward params back to account data
    let serialized_data = borsh::to_vec(&*reward_params).map_err(|_| ProgramError::Custom(503))?;
    reward_data.copy_from_slice(&serialized_data);

    Ok(reward_amount)
}