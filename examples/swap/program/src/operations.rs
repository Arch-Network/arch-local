use arch_program::pubkey::Pubkey;
use arch_program::{account::AccountInfo, program_error::ProgramError};
use borsh::BorshDeserialize;
use borsh::BorshSerialize;

use crate::calculate_swap_amount;
pub use crate::LiquidityParams;
pub use crate::RewardParams;
pub use crate::state::Vault;
pub fn add_liquidity(
    liquidity_account: &AccountInfo,
    liquidity_params: &mut LiquidityParams,
    token_a_amount: u64,
    token_b_amount: u64,
) -> Result<(), ProgramError> {
    //  Update liquidity parameters directly
    liquidity_params.token_a_amount += token_a_amount;
    liquidity_params.token_b_amount += token_b_amount;
    liquidity_params.liquidity_amount = liquidity_params.token_a_amount + liquidity_params.token_b_amount;

    // Serialize and save updated state
    let serialized_data = borsh::to_vec(&*liquidity_params).map_err(|_| ProgramError::Custom(503))?;
    liquidity_account.data.borrow_mut().copy_from_slice(&serialized_data);

    Ok(())
}


pub fn remove_liquidity(
    liquidity_account: &AccountInfo,
    liquidity_params: &mut LiquidityParams,
    token_a_amount: u64,
    token_b_amount: u64,
    current_time: u64,
) -> Result<(), ProgramError> {
    update_yield(liquidity_params, current_time)?;

    let mut liquidity_data = liquidity_account
        .data
        .try_borrow_mut()
        .map_err(|_| ProgramError::Custom(502))?;

    liquidity_params.token_a_amount = liquidity_params
        .token_a_amount
        .saturating_sub(token_a_amount);
    liquidity_params.token_b_amount = liquidity_params
        .token_b_amount
        .saturating_sub(token_b_amount);

    liquidity_params.liquidity_amount = 
        liquidity_params.token_a_amount + liquidity_params.token_b_amount;

    let serialized_data = 
        borsh::to_vec(liquidity_params).map_err(|_| ProgramError::Custom(503))?;
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

    let token_b_amount = calculate_swap_amount(
        liquidity_params.token_a_amount,
        liquidity_params.token_b_amount,
        token_a_amount,
    );

    if token_b_amount < min_token_b_amount {
        return Err(ProgramError::Custom(503));
    }

    liquidity_params.token_a_amount += token_a_amount;
    liquidity_params.token_b_amount -= token_b_amount;
    liquidity_params.liquidity_amount = 
        liquidity_params.token_a_amount + liquidity_params.token_b_amount;

    let serialized_data = 
        borsh::to_vec(liquidity_params).map_err(|_| ProgramError::Custom(504))?;
    liquidity_data.copy_from_slice(&serialized_data);

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

pub fn place_limit_order(
    order_account: &AccountInfo,
    owner: &Pubkey,
    token_pair: (Pubkey, Pubkey),
    amount: u64,
    price: u64,
    order_type: OrderType,
) -> Result<(), ProgramError> {
    let mut order_data = order_account.data.borrow_mut();
    let order = LimitOrder {
        owner: *owner,
        token_pair,
        amount,
        price,
        order_type,
        status: OrderStatus::Open,
    };
    let serialized_order = borsh::to_vec(&order).map_err(|_| ProgramError::Custom(510))?;
    order_data[..serialized_order.len()].copy_from_slice(&serialized_order);
    Ok(())
}

pub fn add_liquidity_to_vault(
    vault: &mut Vault,
    token_a_amount: u64,
    token_b_amount: u64,
) -> Result<(), ProgramError> {
    vault.token_a_amount += token_a_amount;
    vault.token_b_amount += token_b_amount;
    Ok(())
}

pub fn remove_liquidity_from_vault(
    vault: &mut Vault,
    token_a_amount: u64,
    token_b_amount: u64,
) -> Result<(), ProgramError> {
    if vault.token_a_amount < token_a_amount || vault.token_b_amount < token_b_amount {
        return Err(ProgramError::Custom(507)); // Insufficient liquidity
    }
    vault.token_a_amount -= token_a_amount;
    vault.token_b_amount -= token_b_amount;
    Ok(())
}

pub fn swap_tokens_in_vault(
    vault: &mut Vault,
    input_amount: u64,
    min_output_amount: u64,
    is_token_a_to_b: bool,
) -> Result<u64, ProgramError> {
    let output_amount = if is_token_a_to_b {
        calculate_swap_amount(vault.token_a_amount, vault.token_b_amount, input_amount)
    } else {
        calculate_swap_amount(vault.token_b_amount, vault.token_a_amount, input_amount)
    };

    if output_amount < min_output_amount {
        return Err(ProgramError::Custom(508)); // Slippage error
    }

    if is_token_a_to_b {
        vault.token_a_amount += input_amount;
        vault.token_b_amount -= output_amount;
    } else {
        vault.token_b_amount += input_amount;
        vault.token_a_amount -= output_amount;
    }

    Ok(output_amount)
}

pub fn contribute_liquidity(
    liquidity_params: &mut LiquidityParams,
    token_a_contrib: u64,
    token_b_contrib: u64,
) -> Result<(), ProgramError> {
    liquidity_params.token_a_amount += token_a_contrib;
    liquidity_params.token_b_amount += token_b_contrib;
    liquidity_params.liquidity_amount += token_a_contrib + token_b_contrib;
    Ok(())
}

pub fn withdraw_liquidity(
    liquidity_params: &mut LiquidityParams,
    token_a_withdraw: u64,
    token_b_withdraw: u64,
) -> Result<(), ProgramError> {
    if liquidity_params.token_a_amount < token_a_withdraw || liquidity_params.token_b_amount < token_b_withdraw {
        return Err(ProgramError::Custom(507)); // Insufficient liquidity
    }
    liquidity_params.token_a_amount -= token_a_withdraw;
    liquidity_params.token_b_amount -= token_b_withdraw;
    liquidity_params.liquidity_amount -= token_a_withdraw + token_b_withdraw;
    Ok(())
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct LimitOrder {
    pub owner: Pubkey,
    pub token_pair: (Pubkey, Pubkey),
    pub amount: u64,
    pub price: u64,
    pub order_type: OrderType,
    pub status: OrderStatus,
}



#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum OrderStatus {
    Open,
    Executed,
    Cancelled,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum OrderType {
    Buy,
    Sell,
}

pub fn execute_limit_orders(
    orders: &mut [LimitOrder],
    current_market_prices: &[(Pubkey, Pubkey, u64)],
) -> Result<(), ProgramError> {
    for order in orders.iter_mut() {
        if let Some(price) = current_market_prices.iter().find(|(token_a, token_b, _)| *token_a == order.token_pair.0 && *token_b == order.token_pair.1).map(|(_, _, price)| *price) {
            if (order.order_type == OrderType::Buy && price <= order.price) || (order.order_type == OrderType::Sell && price >= order.price) {
                order.status = OrderStatus::Executed;
            }
        }
    }
    Ok(())
}

pub fn update_yield(liquidity_params: &mut LiquidityParams, current_time: u64) -> Result<(), ProgramError> {
    let time_elapsed = current_time - liquidity_params.last_yield_update_time;
    let yield_rate = 0.05; //  yield rate: 5% per year

    //  yield calculation: yield = principal * rate * time
    let additional_yield = (liquidity_params.liquidity_amount as f64 * yield_rate * (time_elapsed as f64 / 31536000.0)) as u64; // 31536000 = seconds in a year

    liquidity_params.yield_accumulated += additional_yield;
    liquidity_params.last_yield_update_time = current_time;

    Ok(())
}
