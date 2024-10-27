pub mod curve;
pub mod instruction;
pub mod operations;
pub mod reward;
pub mod state;
pub mod vault_operations;
pub mod trading_routes;
pub mod limit_order;
pub mod dca;
use trading_routes::TradingRoute;
use limit_order::LimitOrderBook;

pub use curve::*;
pub use instruction::LiquidityInstruction;
pub use operations::*;
pub use reward::RewardParams;
pub use state::LiquidityParams;

use arch_program::{
    account::AccountInfo,
    entrypoint,
    program::{next_account_info, set_return_data},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    if accounts.len() != 1 {
        return Err(ProgramError::Custom(501));
    }

    let account_iter = &mut accounts.iter();
    let liquidity_account = next_account_info(account_iter)?;

    // Deserialize the instruction data
    let instruction = LiquidityInstruction::deserialize(&mut &instruction_data[..])
        .map_err(|_| ProgramError::Custom(502))?;

    let liquidity_data = liquidity_account
        .data
        .try_borrow()
        .map_err(|_| ProgramError::Custom(502))?;
    let mut liquidity_params = LiquidityParams::deserialize(&mut &liquidity_data[..])
        .map_err(|_| ProgramError::Custom(502))?;

    match instruction {
        LiquidityInstruction::AddLiquidity {
            token_a_amount,
            token_b_amount,
        } => {
            add_liquidity(
                &liquidity_account,
                &mut liquidity_params,
                token_a_amount,
                token_b_amount,
            )?;
        }
        LiquidityInstruction::RemoveLiquidity {
            token_a_amount,
            token_b_amount,
        } => {
            remove_liquidity(
                &liquidity_account,
                &mut liquidity_params,
                token_a_amount,
                token_b_amount,
            )?;
        }
        LiquidityInstruction::GetLiquidityAmount => {
            let liquidity_amount = liquidity_params.get_liquidity_amount();
            let serialized_data =
                borsh::to_vec(&liquidity_amount).map_err(|_| ProgramError::Custom(505))?;
            set_return_data(&serialized_data);
        }
        LiquidityInstruction::SwapTokens {
            token_a_amount,
            min_token_b_amount,
        } => {
            swap_tokens(
                &liquidity_account,
                &mut liquidity_params,
                token_a_amount,
                min_token_b_amount,
            )?;
        }
        LiquidityInstruction::StakeTokens { stake_amount } => {
            let accounts_iter = &mut accounts.iter();
            let staking_account = next_account_info(accounts_iter)?;

            let mut reward_params = RewardParams::try_from_slice(&staking_account.data.borrow())
                .map_err(|_| ProgramError::InvalidAccountData)?;

            stake_tokens(staking_account, &mut reward_params, stake_amount)?;

            reward_params
                .serialize(&mut &mut staking_account.data.borrow_mut()[..])
                .map_err(|_| ProgramError::InvalidAccountData)?;
        }
        LiquidityInstruction::UnstakeTokens { unstake_amount } => {
            let accounts_iter = &mut accounts.iter();
            let staking_account = next_account_info(accounts_iter)?;

            let mut reward_params = RewardParams::try_from_slice(&staking_account.data.borrow())
                .map_err(|_| ProgramError::InvalidAccountData)?;

            unstake_tokens(staking_account, &mut reward_params, unstake_amount)?;

            reward_params
                .serialize(&mut &mut staking_account.data.borrow_mut()[..])
                .map_err(|_| ProgramError::InvalidAccountData)?;
        }
        LiquidityInstruction::ClaimRewards => {
            let accounts_iter = &mut accounts.iter();
            let staking_account = next_account_info(accounts_iter)?;

            let mut reward_params = RewardParams::try_from_slice(&staking_account.data.borrow())
                .map_err(|_| ProgramError::InvalidAccountData)?;

            let reward_amount = claim_rewards(staking_account, &mut reward_params)?;

            reward_params
                .serialize(&mut &mut staking_account.data.borrow_mut()[..])
                .map_err(|_| ProgramError::InvalidAccountData)?;
        }
        LiquidityInstruction::OptimizedTradingRoute => {
            let accounts_iter = &mut accounts.iter();
            let staking_account = next_account_info(accounts_iter)?;

            let mut reward_params = RewardParams::try_from_slice(&staking_account.data.borrow())
                .map_err(|_| ProgramError::InvalidAccountData)?;

            let trading_route = TradingRoute::new(&staking_account.data.borrow());
            let optimized_route = trading_route.calculate_best_route(100, Pubkey::new_unique())?;

            reward_params
                .serialize(&mut &mut staking_account.data.borrow_mut()[..])
                .map_err(|_| ProgramError::InvalidAccountData)?;
        }
        LiquidityInstruction::ProcessLimitOrders => {
            let mut order_book = LimitOrderBook::new(); // This would realistically be loaded from state
            order_book.process_orders(&current_market_prices); // Assume current_market_prices is fetched or calculated
            // Save updated order book state
            let serialized_order_book = order_book.serialize().map_err(|_| ProgramError::Custom(506))?;
            let order_book_account = next_account_info(accounts_iter)?;
            order_book_account.data.borrow_mut().copy_from_slice(&serialized_order_book);
        }
        _ => {}
    }

    Ok(())
}
