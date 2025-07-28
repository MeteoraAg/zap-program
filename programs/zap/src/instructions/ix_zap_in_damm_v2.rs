use anchor_lang::{prelude::*, solana_program::log::sol_log_compute_units};
use anchor_spl::token_interface::Mint;
use damm_v2::types::{AddLiquidityParameters, SwapParameters};
use damm_v2_program::{
    activation_handler::ActivationHandler,
    curve::RESOLUTION,
    params::swap::TradeDirection,
    safe_math::SafeMath,
    state::{fee::FeeMode, ModifyLiquidityResult, Pool},
    token::{
        calculate_transfer_fee_excluded_amount, calculate_transfer_fee_included_amount,
        TransferFeeExcludedAmount, TransferFeeIncludedAmount,
    },
    u128x128_math::{mul_div_u256, Rounding},
    PoolError,
};
use num::ToPrimitive;
use ruint::aliases::U256;

use crate::{constants::amm_program_id::DAMM_V2, error::ZapError};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ZapInDammV2Parameters {
    pub a: u64,
    pub b: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ZapInDammV2Result {
    pub liquidity_delta: u128,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub token_a_remaining_amount: u64,
    pub token_b_remaining_amount: u64,
    pub token_swapped_amount: u64,
    pub token_returned_amount: u64,
}

#[derive(Accounts)]
pub struct ZapInDammV2Ctx<'info> {
    /// CHECK: Pool authority
    pub pool_authority: UncheckedAccount<'info>,
    /// CHECK: Pool
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,
    /// CHECK: Pool authority
    #[account(mut)]
    pub position: UncheckedAccount<'info>,
    /// CHECK: The user token a account
    #[account(mut)]
    pub token_a_account: UncheckedAccount<'info>,
    /// CHECK: The user token b account
    #[account(mut)]
    pub token_b_account: UncheckedAccount<'info>,
    /// CHECK: The vault token account for token a
    #[account(mut)]
    pub token_a_vault: UncheckedAccount<'info>,
    /// CHECK: The vault token account for token b
    #[account(mut)]
    pub token_b_vault: UncheckedAccount<'info>,
    /// The mint of token a
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,
    /// The mint of token b
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: The token account for nft
    pub position_nft_account: UncheckedAccount<'info>,
    /// Owner of position
    pub owner: Signer<'info>,
    /// CHECK: Token a program
    pub token_a_program: UncheckedAccount<'info>,
    /// CHECK: Token b program
    pub token_b_program: UncheckedAccount<'info>,
    /// CHECK: DAMM V2 event authority
    pub damm_v2_event_authority: UncheckedAccount<'info>,
    /// CHECK: DAMM V2 program
    #[account(address = DAMM_V2 @ ZapError::InvalidAmmProgramId)]
    pub damm_v2_program: AccountInfo<'info>,
    /// CHECK: Referral token account
    #[account(mut)]
    pub referral_token_account: Option<UncheckedAccount<'info>>,
}

impl<'info> ZapInDammV2Ctx<'info> {
    ///
    /// Simulate an swap on DAMM V2.
    ///
    /// # Arguments
    ///
    /// * `amount_in` - The input amount.
    /// * `local_pool` - The pool state. If `local_pool` is `None`, the function will borrow one from the current context.
    ///
    /// # Returns
    ///
    /// The token amounts with fees included
    ///
    pub fn simulate_swap(
        &self,
        amount_in: u64,
        trade_direction: TradeDirection,
        local_pool: Option<&mut Pool>,
    ) -> Result<u64> {
        let mut borrowed_pool = self.pool.load()?.clone();
        let pool = local_pool.unwrap_or(&mut borrowed_pool);
        // Parse accounts
        let (token_in_mint, token_out_mint) = if trade_direction == TradeDirection::AtoB {
            (&self.token_a_mint, &self.token_b_mint)
        } else {
            (&self.token_a_mint, &self.token_b_mint)
        };
        // Transfer-in fee (Token Extension)
        let TransferFeeExcludedAmount {
            amount: transfer_fee_excluded_amount_in,
            ..
        } = calculate_transfer_fee_excluded_amount(token_in_mint, amount_in)?;
        require!(transfer_fee_excluded_amount_in > 0, PoolError::AmountIsZero);
        // Swap fee
        let has_referral = self.referral_token_account.is_some();
        let fee_mode =
            &FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, has_referral)?;
        let current_timestamp = Clock::get()?
            .unix_timestamp
            .to_u64()
            .ok_or(PoolError::MathOverflow)?;
        let current_point = ActivationHandler::get_current_point(pool.activation_type)?;
        // Update for dynamic fee references
        pool.update_pre_swap(current_timestamp)?;
        // Swap (We can skip the pre-swap update cause it doesn't immediately affect to the result)
        let swap_result = pool.get_swap_result(
            transfer_fee_excluded_amount_in,
            fee_mode,
            trade_direction,
            current_point,
        )?;
        // Apply the swap result
        pool.apply_swap_result(&swap_result, fee_mode, current_timestamp)?;
        // Transfer-out fee (Token Extension)
        let TransferFeeExcludedAmount {
            amount: transfer_fee_excluded_amount_out,
            ..
        } = calculate_transfer_fee_excluded_amount(token_out_mint, swap_result.output_amount)?;
        Ok(transfer_fee_excluded_amount_out)
    }

    ///
    /// Invoke the swap on DAMM V2.
    ///
    /// # Arguments
    ///
    /// * `amount_in` - The input amount.
    /// * `minimum_amount_out` - The minimum output amount.
    /// * `trade_direction` - The trade direction.
    ///
    pub fn swap(
        &self,
        amount_in: u64,
        minimum_amount_out: u64,
        trade_direction: TradeDirection,
    ) -> Result<()> {
        let (token_in_account, token_out_account) = if trade_direction == TradeDirection::AtoB {
            (&self.token_a_account, &self.token_b_account)
        } else {
            (&self.token_b_account, &self.token_a_account)
        };
        damm_v2::cpi::swap(
            CpiContext::new(
                self.damm_v2_program.to_account_info(),
                damm_v2::cpi::accounts::Swap {
                    pool_authority: self.pool_authority.to_account_info(),
                    pool: self.pool.to_account_info(),
                    input_token_account: token_in_account.to_account_info(),
                    output_token_account: token_out_account.to_account_info(),
                    token_a_vault: self.token_a_vault.to_account_info(),
                    token_b_vault: self.token_b_vault.to_account_info(),
                    token_a_mint: self.token_a_mint.to_account_info(),
                    token_b_mint: self.token_b_mint.to_account_info(),
                    payer: self.owner.to_account_info(),
                    token_a_program: self.token_a_program.to_account_info(),
                    token_b_program: self.token_b_program.to_account_info(),
                    referral_token_account: self
                        .referral_token_account
                        .as_ref()
                        .map(|acc| acc.to_account_info()),
                    event_authority: self.damm_v2_event_authority.to_account_info(),
                    program: self.damm_v2_program.to_account_info(),
                },
            ),
            SwapParameters {
                amount_in,
                minimum_amount_out,
            },
        )
    }

    ///
    /// Simulate an add_liquidity on DAMM V2.
    ///
    /// # Arguments
    ///
    /// * `liquidity_delta` - The liquidity delta.
    /// * `local_pool` - The pool state. If `local_pool` is `None`, the function will borrow one from the current context.
    ///
    /// # Returns
    ///
    /// The required amounts of token A & B
    ///
    pub fn simulate_add_liquidity(
        &self,
        liquidity_delta: u128,
        local_pool: Option<&Pool>,
    ) -> Result<(u64, u64)> {
        let borrowed_pool = self.pool.load()?.clone();
        let pool = local_pool.unwrap_or(&borrowed_pool);

        let (token_a_amount, token_b_amount) =
            derive_inputs_based_on_liquidity_delta(liquidity_delta, &pool)?;
        let TransferFeeIncludedAmount {
            amount: transfer_fee_included_token_a_amount,
            ..
        } = calculate_transfer_fee_included_amount(&self.token_a_mint, token_a_amount)?;
        let TransferFeeIncludedAmount {
            amount: transfer_fee_included_token_b_amount,
            ..
        } = calculate_transfer_fee_included_amount(&self.token_b_mint, token_b_amount)?;

        Ok((
            transfer_fee_included_token_a_amount,
            transfer_fee_included_token_b_amount,
        ))
    }

    ///
    /// Invoke the add_liqudity on DAMM V2.
    ///
    /// # Arguments
    ///
    /// * `liquidity_delta` - The liquidity delta.
    /// * `token_a_amount_threshold` - The maximum input amount of token A.
    /// * `token_b_amount_threshold` - The maximum input amount of token B.
    ///
    pub fn add_liqudity(
        &self,
        liquidity_delta: u128,
        token_a_amount_threshold: u64,
        token_b_amount_threshold: u64,
    ) -> Result<()> {
        damm_v2::cpi::add_liquidity(
            CpiContext::new(
                self.damm_v2_program.to_account_info(),
                damm_v2::cpi::accounts::AddLiquidity {
                    pool: self.pool.to_account_info(),
                    position: self.position.to_account_info(),
                    token_a_account: self.token_a_account.to_account_info(),
                    token_b_account: self.token_b_account.to_account_info(),
                    token_a_vault: self.token_a_vault.to_account_info(),
                    token_b_vault: self.token_b_vault.to_account_info(),
                    token_a_mint: self.token_a_mint.to_account_info(),
                    token_b_mint: self.token_b_mint.to_account_info(),
                    position_nft_account: self.position_nft_account.to_account_info(),
                    owner: self.owner.to_account_info(),
                    token_a_program: self.token_a_program.to_account_info(),
                    token_b_program: self.token_b_program.to_account_info(),
                    event_authority: self.damm_v2_event_authority.to_account_info(),
                    program: self.damm_v2_program.to_account_info(),
                },
            ),
            AddLiquidityParameters {
                liquidity_delta,
                token_a_amount_threshold,
                token_b_amount_threshold,
            },
        )
    }

    ///
    /// Derive the liquidity delta based on the amount of token A.
    ///
    /// # Arguments
    ///
    /// * `a` - The desired amount of token A.
    /// * `local_pool` - The pool state. If `local_pool` is `None`, the function will borrow one from the current context.
    ///
    /// # Returns
    ///
    /// The liquidity delta
    ///
    pub fn derive_liquidity_delta_based_on_a_included_fees(
        &self,
        a: u64,
        local_pool: Option<&Pool>,
    ) -> Result<u128> {
        let borrowed_pool = self.pool.load()?.clone();
        let pool = local_pool.unwrap_or(&borrowed_pool);
        let TransferFeeExcludedAmount {
            amount: transfer_fee_excluded_token_a_amount,
            ..
        } = calculate_transfer_fee_excluded_amount(&self.token_a_mint, a)?;
        let liquidity_delta =
            derive_liquidity_delta_based_on_a(transfer_fee_excluded_token_a_amount, pool)?;

        Ok(liquidity_delta)
    }

    ///
    /// Derive the liquidity delta based on the amount of token B.
    ///
    /// # Arguments
    ///
    /// * `b` - The desired amount of token B.
    /// * `local_pool` - The pool state. If `local_pool` is `None`, the function will borrow one from the current context.
    ///
    /// # Returns
    ///
    /// The liquidity delta
    ///
    pub fn derive_liquidity_delta_based_on_b_included_fees(
        &self,
        b: u64,
        local_pool: Option<&Pool>,
    ) -> Result<u128> {
        let borrowed_pool = self.pool.load()?.clone();
        let pool = local_pool.unwrap_or(&borrowed_pool);
        let TransferFeeExcludedAmount {
            amount: transfer_fee_excluded_token_b_amount,
            ..
        } = calculate_transfer_fee_excluded_amount(&self.token_b_mint, b)?;
        let liquidity_delta =
            derive_liquidity_delta_based_on_b(transfer_fee_excluded_token_b_amount, &pool)?;

        Ok(liquidity_delta)
    }
}

///
/// To add liquidity to DAMM V2, the token amounts are calculated as follows:
///
/// `a = ΔL * (1/√P - 1/√P_max)`
/// `b = ΔL * (√P - √P_min)`
///
/// To maintain generality, we support two distinct cases: adding {a, 0} and adding {0, b}.
/// For imbalanced additions of the form {a, b}, we sequentially process {a, 0} followed by {0, b}.
///
pub fn handle_zap_in_damm_v2(
    ctx: Context<ZapInDammV2Ctx>,
    ZapInDammV2Parameters { a, b }: ZapInDammV2Parameters,
) -> Result<Vec<ZapInDammV2Result>> {
    require!(a != 0 || b != 0, ZapError::AmountIsZero);
    if a == 0 {
        return Ok(vec![handle_zap_on_b_in_damm_v2(ctx, b)?]);
    }
    if b == 0 {
        return Ok(vec![handle_zap_on_a_in_damm_v2(ctx, a)?]);
    }

    let mut result: Vec<ZapInDammV2Result> = vec![];
    let liquidity_delta_based_on_a = ctx
        .accounts
        .derive_liquidity_delta_based_on_a_included_fees(a, None)?;
    let liquidity_delta_based_on_b = ctx
        .accounts
        .derive_liquidity_delta_based_on_b_included_fees(b, None)?;
    // liqidity delta (the cut-off)
    let liquidity_delta = liquidity_delta_based_on_a.min(liquidity_delta_based_on_b);
    require!(liquidity_delta > 0, ZapError::AmountIsZero);
    // Add liqidity
    let (token_a_amount_threshold, token_b_amount_threshold) =
        ctx.accounts.simulate_add_liquidity(liquidity_delta, None)?;
    ctx.accounts.add_liqudity(
        liquidity_delta,
        token_a_amount_threshold,
        token_b_amount_threshold,
    )?;
    // Remaining
    let token_a_remaining_amount = a.safe_sub(token_a_amount_threshold)?;
    let token_b_remaining_amount = b.safe_sub(token_b_amount_threshold)?;
    result.push(ZapInDammV2Result {
        liquidity_delta,
        token_a_amount: token_a_amount_threshold,
        token_b_amount: token_b_amount_threshold,
        token_a_remaining_amount,
        token_b_remaining_amount,
        token_returned_amount: 0,
        token_swapped_amount: 0,
    });

    if token_a_remaining_amount == 0 && token_b_remaining_amount == 0 {
        return Ok(result);
    } else if token_a_remaining_amount == 0 {
        let sub = handle_zap_on_b_in_damm_v2(ctx, token_b_remaining_amount)?;
        result.push(sub);
        return Ok(result);
    } else if token_b_remaining_amount == 0 {
        let sub = handle_zap_on_a_in_damm_v2(ctx, token_a_remaining_amount)?;
        result.push(sub);
        return Ok(result);
    } else {
        err!(ZapError::CannotConvergeToOptimalValue)
    }
}

///
/// Handle zap-in on the side of token A only. We will execute a binary search on `a` to find the solutions for `liquidity_delta`.
///
/// `amount_in = a + Δa`
///
/// `ΔL = a * √P * √P_max / (√P_max - √P)`
///
pub fn handle_zap_on_a_in_damm_v2(
    ctx: Context<ZapInDammV2Ctx>,
    amount_in: u64,
) -> Result<ZapInDammV2Result> {
    require!(amount_in > 0, ZapError::AmountIsZero);
    let trade_direction = TradeDirection::AtoB;

    let mut min_a: u64 = 0;
    let mut max_a: u64 = amount_in;

    let mut a = min_a.safe_add(max_a)?.safe_div(2)?;
    let mut liquidity_delta: u128;
    let mut b: u64;
    let mut confused_flag: i8 = 2;
    loop {
        let pool_data = ctx.accounts.pool.load()?;
        let mut pool: Pool = pool_data.clone();
        // Confused flag is to detect when `a` jumps between max and min with max-min=1 and cannot reach any stop condition
        if max_a.safe_sub(min_a)? <= 1 {
            confused_flag -= 1;
        }
        // Assume the number of swapped tokens
        b = ctx
            .accounts
            .simulate_swap(amount_in.safe_sub(a)?, trade_direction, Some(&mut pool))?;
        // Assume liquidity delta
        liquidity_delta = ctx
            .accounts
            .derive_liquidity_delta_based_on_b_included_fees(b, Some(&pool))?;
        // Compute the token amounts based on the assumed liquidity delta
        let (token_a_amount, _) = derive_inputs_based_on_liquidity_delta(liquidity_delta, &pool)?;
        let TransferFeeIncludedAmount { amount: a_, .. } =
            calculate_transfer_fee_included_amount(&ctx.accounts.token_a_mint, token_a_amount)?;
        // Converge the mid point of liquidity delta
        if a_ > a {
            if confused_flag <= 0 {
                break;
            }
            // If a' > a, min_a = a, max_a = a'
            min_a = a;
            max_a = a_;
            a = min_a.safe_add(max_a)?.safe_add(1)?.safe_div(2)?; // Adding 1 to round up
        } else if a_ < a {
            // If a' < a, min_a = a', max_a = a
            min_a = a_;
            max_a = a;
            a = min_a.safe_add(max_a)?.safe_div(2)?;
        } else {
            // If a' = a, stop
            break;
        }
    }
    if liquidity_delta == 0 {
        return Ok(ZapInDammV2Result {
            liquidity_delta: 0,
            token_a_amount: amount_in,
            token_b_amount: 0,
            token_a_remaining_amount: amount_in,
            token_b_remaining_amount: 0,
            token_swapped_amount: 0,
            token_returned_amount: 0,
        });
    }

    // Swap
    sol_log_compute_units();
    let token_a_swapped_amount = amount_in.safe_sub(a)?;
    ctx.accounts
        .swap(token_a_swapped_amount, b, trade_direction)?;
    sol_log_compute_units();

    // Add liqidity
    sol_log_compute_units();
    let (token_a_amount_threshold, token_b_amount_threshold) =
        ctx.accounts.simulate_add_liquidity(liquidity_delta, None)?;
    ctx.accounts.add_liqudity(
        liquidity_delta,
        token_a_amount_threshold,
        token_b_amount_threshold,
    )?;
    sol_log_compute_units();

    Ok(ZapInDammV2Result {
        liquidity_delta,
        token_a_amount: amount_in,
        token_b_amount: 0,
        token_a_remaining_amount: a.safe_sub(token_a_amount_threshold)?,
        token_b_remaining_amount: b.safe_sub(token_b_amount_threshold)?,
        token_swapped_amount: token_a_swapped_amount,
        token_returned_amount: b,
    })
}

///
/// Handle zap-in on the side of token B only. We will execute a binary search on `b` to find the solutions for `liquidity_delta`.
///
/// `amount_in = b + Δb`
///
/// `ΔL = b / (√P - √P_min)`
///
pub fn handle_zap_on_b_in_damm_v2(
    ctx: Context<ZapInDammV2Ctx>,
    amount_in: u64,
) -> Result<ZapInDammV2Result> {
    require!(amount_in > 0, ZapError::AmountIsZero);
    let trade_direction = TradeDirection::BtoA;

    let mut min_b: u64 = 0;
    let mut max_b: u64 = amount_in;

    let mut b = min_b.safe_add(max_b)?.safe_div(2)?;
    let mut liquidity_delta: u128;
    let mut a: u64;
    let mut confused_flag: i8 = 2;
    loop {
        let pool_data = ctx.accounts.pool.load()?;
        let mut pool: Pool = pool_data.clone();
        // Confused flag is to detect when `b` jumps between max and min with max-min=1 and cannot reach any stop condition
        if max_b.safe_sub(min_b)? <= 1 {
            confused_flag -= 1;
        }
        // Assume the number of swapped tokens
        a = ctx
            .accounts
            .simulate_swap(amount_in.safe_sub(b)?, trade_direction, Some(&mut pool))?;
        // Assume liquidity delta
        liquidity_delta = ctx
            .accounts
            .derive_liquidity_delta_based_on_b_included_fees(b, Some(&pool))?;
        // Compute the token amounts based on the assumed liquidity delta
        let (_, token_b_amount) = derive_inputs_based_on_liquidity_delta(liquidity_delta, &pool)?;
        let TransferFeeIncludedAmount { amount: b_, .. } =
            calculate_transfer_fee_included_amount(&ctx.accounts.token_b_mint, token_b_amount)?;
        // Converge the mid point of liquidity delta
        if b_ > b {
            if confused_flag <= 0 {
                break;
            }
            // If b' > b, min_a = a, max_a = a'
            min_b = b;
            max_b = b_;
            b = min_b.safe_add(max_b)?.safe_add(1)?.safe_div(2)?; // Adding 1 to round up
        } else if b_ < b {
            // If b' < b, min_b = b', max_b = b
            min_b = b_;
            max_b = b;
            b = min_b.safe_add(max_b)?.safe_div(2)?;
        } else {
            // If b' = b, stop
            break;
        }
    }
    if liquidity_delta == 0 {
        return Ok(ZapInDammV2Result {
            liquidity_delta: 0,
            token_a_amount: 0,
            token_b_amount: amount_in,
            token_a_remaining_amount: 0,
            token_b_remaining_amount: amount_in,
            token_swapped_amount: 0,
            token_returned_amount: 0,
        });
    }

    // Swap
    let token_b_swapped_amount = amount_in.safe_sub(b)?;
    ctx.accounts
        .swap(token_b_swapped_amount, a, trade_direction)?;

    // Add liqidity
    let (token_a_amount_threshold, token_b_amount_threshold) =
        ctx.accounts.simulate_add_liquidity(liquidity_delta, None)?;
    ctx.accounts.add_liqudity(
        liquidity_delta,
        token_a_amount_threshold,
        token_b_amount_threshold,
    )?;

    Ok(ZapInDammV2Result {
        liquidity_delta,
        token_a_amount: 0,
        token_b_amount: amount_in,
        token_a_remaining_amount: a.safe_sub(token_a_amount_threshold)?,
        token_b_remaining_amount: b.safe_sub(token_b_amount_threshold)?,
        token_swapped_amount: token_b_swapped_amount,
        token_returned_amount: a,
    })
}

///
/// Utils
///
/// We isolate these functions so that:
///
/// 1. Rounding up/down are tightly binded into the convegence loop and results, then unintended changes will cause very unexpected output. These encapsulated functions are to avoid mistakes.
/// 2. Posibility for unit tests.
///

pub fn derive_inputs_based_on_liquidity_delta(
    liquidity_delta: u128,
    pool: &Pool,
) -> Result<(u64, u64)> {
    let ModifyLiquidityResult {
        token_a_amount,
        token_b_amount,
    } = pool.get_amounts_for_modify_liquidity(liquidity_delta, Rounding::Up)?;

    Ok((token_a_amount, token_b_amount))
}

pub fn derive_liquidity_delta_based_on_a(a: u64, pool: &Pool) -> Result<u128> {
    let liquidity_delta: u128 = mul_div_u256(
        U256::from(a),
        U256::from(pool.sqrt_price).safe_mul(U256::from(pool.sqrt_max_price))?,
        U256::from(pool.sqrt_max_price - pool.sqrt_price),
        Rounding::Down,
    )
    .ok_or(ZapError::MathOverflow)?
    .try_into()
    .map_err(|_| ZapError::MathOverflow)?;

    Ok(liquidity_delta)
}

pub fn derive_liquidity_delta_based_on_b(b: u64, pool: &Pool) -> Result<u128> {
    let liquidity_delta: u128 = mul_div_u256(
        U256::from(b),
        U256::from(1).safe_shl(RESOLUTION as usize * 2)?,
        U256::from(pool.sqrt_price - pool.sqrt_min_price),
        Rounding::Down,
    )
    .ok_or(ZapError::MathOverflow)?
    .try_into()
    .map_err(|_| ZapError::MathOverflow)?;

    Ok(liquidity_delta)
}

///
/// Unit tests
///

#[cfg(test)]
mod tests {
    use super::*;
    use damm_v2_program::state::{
        fee::{BaseFeeStruct, PoolFeesStruct},
        Pool,
    };
    use integer_sqrt::IntegerSquareRoot;
    use proptest::prelude::*;

    const RES: u128 = 1u128 << RESOLUTION;
    const MIN_SUPPLY: u64 = 1000000000;
    const MAX_SUPPLY: u64 = 1000000000000000000;

    prop_compose! {
        fn custom_strategy()
            (x in MIN_SUPPLY..MAX_SUPPLY, y in 2u64..1000)
            (a in 1u64..x, b in Just(y), c in x..MAX_SUPPLY, d in x..MAX_SUPPLY) -> (u64, u64, u128, u128) {
                (a, b, c as u128, d as u128)
            }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn test_liquidity_delta_shifting_on_a((a, gamma, liquidity_a, liquidity_b) in custom_strategy()) {
            // L = √AB
            let liquidity = (liquidity_a * liquidity_b).integer_sqrt() * RES;
            // √P = √B/A
            let sqrt_price = liquidity_b.integer_sqrt() * RES / liquidity_a.integer_sqrt();
            // √P_min = √P / gamma
            let sqrt_min_price = sqrt_price / gamma as u128;
            // √P_min = √P * gamma
            let sqrt_max_price = sqrt_price * gamma as u128;
            let pool = Pool {
                pool_fees: PoolFeesStruct {
                    base_fee: BaseFeeStruct {
                        cliff_fee_numerator: 2_500_000,
                        number_of_period: 0,
                        reduction_factor: 0,
                        period_frequency: 0,
                        fee_scheduler_mode: 0,
                        ..Default::default()
                    },
                    protocol_fee_percent: 20,
                    partner_fee_percent: 0,
                    referral_fee_percent: 0,
                    ..Default::default()
                },
                sqrt_min_price,
                sqrt_max_price,
                liquidity,
                sqrt_price,
                activation_type: 0,
                collect_fee_mode: 0,
                ..Default::default()
            };

            let liquidity_delta = derive_liquidity_delta_based_on_a(a, &pool).unwrap();
            if let Ok((a_, _)) = derive_inputs_based_on_liquidity_delta(liquidity_delta, &pool) {
                assert_eq!(a, a_);
            }
        }

        #[test]
        fn test_liquidity_delta_shifting_on_b((b, gamma, liquidity_a, liquidity_b) in custom_strategy()) {
            // L = √AB
            let liquidity = (liquidity_a * liquidity_b).integer_sqrt() * RES;
            // √P = √B/A
            let sqrt_price = liquidity_b.integer_sqrt() * RES / liquidity_a.integer_sqrt();
            // √P_min = √P / gamma
            let sqrt_min_price = sqrt_price / gamma as u128;
            // √P_min = √P * gamma
            let sqrt_max_price = sqrt_price * gamma as u128;
            let pool = Pool {
                pool_fees: PoolFeesStruct {
                    base_fee: BaseFeeStruct {
                        cliff_fee_numerator: 2_500_000,
                        number_of_period: 0,
                        reduction_factor: 0,
                        period_frequency: 0,
                        fee_scheduler_mode: 0,
                        ..Default::default()
                    },
                    protocol_fee_percent: 20,
                    partner_fee_percent: 0,
                    referral_fee_percent: 0,
                    ..Default::default()
                },
                sqrt_min_price,
                sqrt_max_price,
                liquidity,
                sqrt_price,
                activation_type: 0,
                collect_fee_mode: 0,
                ..Default::default()
            };

            let liquidity_delta = derive_liquidity_delta_based_on_b(b, &pool).unwrap();
            if let Ok((_, b_)) = derive_inputs_based_on_liquidity_delta(liquidity_delta, &pool) {
                assert_eq!(b, b_);
            }
        }
    }
}
