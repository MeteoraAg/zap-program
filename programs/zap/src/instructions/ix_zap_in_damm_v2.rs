use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use damm_v2::types::{AddLiquidityParameters, SwapParameters};
use damm_v2_program::{
    activation_handler::ActivationHandler,
    constants::seeds::POOL_AUTHORITY_PREFIX,
    params::swap::TradeDirection,
    safe_math::SafeMath,
    state::{fee::FeeMode, ModifyLiquidityResult, Pool, Position},
    token::{
        calculate_transfer_fee_excluded_amount, calculate_transfer_fee_included_amount,
        TransferFeeExcludedAmount, TransferFeeIncludedAmount,
    },
    u128x128_math::{mul_div_u256, Rounding},
    AddLiquidityCtx, PoolError, SwapCtx,
};
use ruint::aliases::U256;

use crate::{constants::amm_program_id::DAMM_V2, error::ZapError};

///
/// To add liquidity to DAMM V2, the token amounts are calculated as follows:
///
/// `a = ΔL * (1/√P - 1/√P_max)`
/// `b = ΔL * (√P - √P_min)`
///
/// To maintain generality, we support two distinct cases: adding {a, 0} and adding {0, b}.
/// For imbalanced additions of the form {a, b}, we sequentially process {a, 0} followed by {0, b}.
///
#[event_cpi]
#[derive(Accounts)]
pub struct ZapInDammV2Ctx<'info> {
    /// CHECK: Pool authority
    #[account(
        seeds = [POOL_AUTHORITY_PREFIX.as_ref()],
        bump,
        seeds::program = damm_v2_program
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// Pool
    #[account(
        mut,
        has_one = token_a_vault,
        has_one = token_b_vault,
        has_one = token_a_mint,
        has_one = token_b_mint
    )]
    pub pool: AccountLoader<'info, Pool>,

    #[account(mut, has_one = pool)]
    pub position: AccountLoader<'info, Position>,

    /// The user token a account
    #[account(mut)]
    pub token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The user token b account
    #[account(mut)]
    pub token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for input token
    #[account(
        mut,
        token::token_program = token_a_program,
        token::mint = token_a_mint
    )]
    pub token_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for output token
    #[account(
        mut,
        token::token_program = token_b_program,
        token::mint = token_b_mint
    )]
    pub token_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token a
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token b
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The token account for nft
    #[account(
        constraint = position_nft_account.mint == position.load()?.nft_mint,
        constraint = position_nft_account.amount == 1,
        token::authority = owner
    )]
    pub position_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Owner of position
    pub owner: Signer<'info>,

    /// Token a program
    pub token_a_program: Interface<'info, TokenInterface>,

    /// Token b program
    pub token_b_program: Interface<'info, TokenInterface>,

    /// CHECK: DAMM V2 event authority
    #[account(
        seeds = [b"__event_authority"],
        bump,
        seeds::program = damm_v2_program,
    )]
    pub damm_v2_event_authority: AccountInfo<'info>,

    /// CHECK: DAMM V2 program
    #[account(address = DAMM_V2 @ ZapError::InvalidAmmProgramId)]
    pub damm_v2_program: UncheckedAccount<'info>,

    /// Referral token account
    #[account(mut)]
    pub referral_token_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
}

impl<'info> ZapInDammV2Ctx<'info> {
    pub fn get_swap_ix_data(&self, params: SwapParameters) -> Result<Vec<u8>> {
        let mut data = vec![];
        data.extend_from_slice(damm_v2::client::args::Swap::DISCRIMINATOR);
        data.extend_from_slice(&params.try_to_vec()?);
        Ok(data)
    }

    pub fn get_add_liquidity_ix_data(&self, params: AddLiquidityParameters) -> Result<Vec<u8>> {
        let mut data = vec![];
        data.extend_from_slice(damm_v2::client::args::AddLiquidity::DISCRIMINATOR);
        data.extend_from_slice(&params.try_to_vec()?);
        Ok(data)
    }

    pub fn sort_accounts(
        &self,
        trade_direction: TradeDirection,
    ) -> (
        &Box<InterfaceAccount<'info, Mint>>,
        &Box<InterfaceAccount<'info, Mint>>,
        &Box<InterfaceAccount<'info, TokenAccount>>,
        &Box<InterfaceAccount<'info, TokenAccount>>,
        &Interface<'info, TokenInterface>,
        &Interface<'info, TokenInterface>,
    ) {
        match trade_direction {
            TradeDirection::AtoB => (
                &self.token_a_mint,
                &self.token_b_mint,
                &self.token_a_vault,
                &self.token_b_vault,
                &self.token_a_program,
                &self.token_b_program,
            ),
            TradeDirection::BtoA => (
                &self.token_b_mint,
                &self.token_a_mint,
                &self.token_b_vault,
                &self.token_a_vault,
                &self.token_b_program,
                &self.token_a_program,
            ),
        }
    }

    ///
    /// Simulate an atomic swap on DAMM V2
    ///
    pub fn simulate_swap(&self, amount_in: u64, trade_direction: TradeDirection) -> Result<u64> {
        // Parse accounts
        let (token_in_mint, token_out_mint, ..) = self.sort_accounts(trade_direction);
        let pool = self.pool.load()?;
        // Transfer-in fee (Token Extension)
        let TransferFeeExcludedAmount {
            amount: transfer_fee_excluded_amount_in,
            ..
        } = calculate_transfer_fee_excluded_amount(&token_in_mint, amount_in)?;
        require!(transfer_fee_excluded_amount_in > 0, PoolError::AmountIsZero);
        // Swap fee
        let has_referral = self.referral_token_account.is_some();
        let fee_mode =
            &FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, has_referral)?;
        let current_point = ActivationHandler::get_current_point(pool.activation_type)?;
        // Swap (skip the pre-swap update cause it doesn't immediately affect to the result)
        let swap_result = pool.get_swap_result(
            transfer_fee_excluded_amount_in,
            fee_mode,
            trade_direction,
            current_point,
        )?;
        // Transfer-out fee (Token Extension)
        let TransferFeeExcludedAmount {
            amount: transfer_fee_excluded_amount_out,
            ..
        } = calculate_transfer_fee_excluded_amount(&token_out_mint, swap_result.output_amount)?;
        // TODO: how to avoid slippage rate?
        Ok(transfer_fee_excluded_amount_out)
    }
}

///
/// Handle zap-in on the side of token A only. We will execute a binary search on ΔL to find solutions for token_a_amount and token_b_amount.
///
/// `ΔL_min = 0`
///
/// `ΔL_max = a * √P * √P_max / ( √P_max - √P )`
///
pub fn handle_zap_on_a_in_damm_v2(
    ctx: Context<ZapInDammV2Ctx>,
    a: u64,
) -> Result<(u128, u64, u64)> {
    let trade_direction = TradeDirection::AtoB;

    let mut min_liquidity_delta: u128 = 0;
    let mut max_liquidity_delta: u128 = {
        let pool = ctx.accounts.pool.load()?;
        mul_div_u256(
            U256::from(a),
            U256::from(pool.sqrt_price).safe_mul(U256::from(pool.sqrt_max_price))?,
            U256::from(pool.sqrt_max_price - pool.sqrt_price),
            Rounding::Up,
        )
        .ok_or_else(|| ZapError::MathOverflow)?
        .try_into()
        .map_err(|_| ZapError::MathOverflow)?
    };

    let mut liquidity_delta = min_liquidity_delta
        .safe_add(max_liquidity_delta)?
        .safe_div(2)?;
    let mut token_a_swapped_amount: u64;
    let mut returned_b_amount: u64;
    let mut confused_flag: i8 = 2;
    loop {
        // Confused flag is to detect when liquidity_delta jumps between max and min with max-min=1 and cannot reach any stop condition
        if max_liquidity_delta.safe_sub(min_liquidity_delta)? <= 1 {
            confused_flag -= 1;
        }
        // Compute the token amounts based on the assumed liquidity delta
        let pool = ctx.accounts.pool.load()?;
        let ModifyLiquidityResult {
            token_a_amount,
            token_b_amount,
        } = pool.get_amounts_for_modify_liquidity(liquidity_delta, Rounding::Up)?;
        let TransferFeeIncludedAmount {
            amount: transfer_fee_included_token_a_amount,
            ..
        } = calculate_transfer_fee_included_amount(&ctx.accounts.token_a_mint, token_a_amount)?;
        let TransferFeeIncludedAmount {
            amount: transfer_fee_included_token_b_amount,
            ..
        } = calculate_transfer_fee_included_amount(&ctx.accounts.token_b_mint, token_b_amount)?;
        // Compare to the actual swap
        token_a_swapped_amount = a.safe_sub(transfer_fee_included_token_a_amount)?;
        returned_b_amount = ctx
            .accounts
            .simulate_swap(token_a_swapped_amount, trade_direction)?;
        // Converge the mid point of liquidity delta
        if returned_b_amount > transfer_fee_included_token_b_amount {
            if confused_flag <= 0 {
                break;
            }
            // If returned_b_amount > transfer_fee_included_token_b_amount, raise liquidity_delta
            min_liquidity_delta = liquidity_delta;
            liquidity_delta = min_liquidity_delta
                .safe_add(max_liquidity_delta)?
                .safe_add(1)? // To round up
                .safe_div(2)?;
        } else if returned_b_amount < transfer_fee_included_token_b_amount {
            // If returned_b_amount < transfer_fee_included_token_b_amount, lower liquidity_delta
            max_liquidity_delta = liquidity_delta;
            liquidity_delta = min_liquidity_delta
                .safe_add(max_liquidity_delta)?
                .safe_div(2)?;
        } else {
            // If returned_b_amount = transfer_fee_included_token_b_amount, stop liquidity_delta
            break;
        }
    }

    // Swap
    let swap_ctx = CpiContext::new(
        ctx.accounts.damm_v2_program.to_account_info(),
        SwapCtx {
            pool_authority: ctx.accounts.pool_authority.clone(),
            pool: ctx.accounts.pool.clone(),
            input_token_account: ctx.accounts.token_a_account.clone(),
            output_token_account: ctx.accounts.token_b_account.clone(),
            token_a_vault: ctx.accounts.token_a_vault.clone(),
            token_b_vault: ctx.accounts.token_b_vault.clone(),
            token_a_mint: ctx.accounts.token_a_mint.clone(),
            token_b_mint: ctx.accounts.token_b_mint.clone(),
            payer: ctx.accounts.owner.clone(),
            token_a_program: ctx.accounts.token_a_program.clone(),
            token_b_program: ctx.accounts.token_b_program.clone(),
            referral_token_account: ctx.accounts.referral_token_account.clone(),
            event_authority: ctx.accounts.damm_v2_event_authority.to_account_info(),
            program: ctx.accounts.damm_v2_program.to_account_info(),
        },
    );
    let swap_ix = Instruction {
        program_id: ctx.accounts.damm_v2_program.key(),
        accounts: swap_ctx.to_account_metas(None),
        data: ctx.accounts.get_swap_ix_data(SwapParameters {
            amount_in: token_a_swapped_amount,
            minimum_amount_out: returned_b_amount,
        })?,
    };
    invoke(&swap_ix, &swap_ctx.to_account_infos())?;

    // Add liqidity
    let add_liquidity_ctx = CpiContext::new(
        ctx.accounts.damm_v2_program.to_account_info(),
        AddLiquidityCtx {
            pool: ctx.accounts.pool.clone(),
            position: ctx.accounts.position.clone(),
            token_a_account: ctx.accounts.token_a_account.clone(),
            token_b_account: ctx.accounts.token_b_account.clone(),
            token_a_vault: ctx.accounts.token_a_vault.clone(),
            token_b_vault: ctx.accounts.token_b_vault.clone(),
            token_a_mint: ctx.accounts.token_a_mint.clone(),
            token_b_mint: ctx.accounts.token_b_mint.clone(),
            position_nft_account: ctx.accounts.position_nft_account.clone(),
            owner: ctx.accounts.owner.clone(),
            token_a_program: ctx.accounts.token_a_program.clone(),
            token_b_program: ctx.accounts.token_b_program.clone(),
            event_authority: ctx.accounts.damm_v2_event_authority.to_account_info(),
            program: ctx.accounts.damm_v2_program.to_account_info(),
        },
    );

    // Debug
    let (token_a_amount_threshold, token_b_amount_threshold) = {
        let pool = ctx.accounts.pool.load()?;
        let ModifyLiquidityResult {
            token_a_amount,
            token_b_amount,
        } = pool.get_amounts_for_modify_liquidity(liquidity_delta, Rounding::Up)?;
        let TransferFeeIncludedAmount {
            amount: transfer_fee_included_token_a_amount,
            ..
        } = calculate_transfer_fee_included_amount(&ctx.accounts.token_a_mint, token_a_amount)?;
        let TransferFeeIncludedAmount {
            amount: transfer_fee_included_token_b_amount,
            ..
        } = calculate_transfer_fee_included_amount(&ctx.accounts.token_b_mint, token_b_amount)?;
        (
            transfer_fee_included_token_a_amount,
            transfer_fee_included_token_b_amount,
        )
    };
    msg!(
        "a {} b {}",
        token_a_amount_threshold,
        token_b_amount_threshold
    );
    let add_liquidity_ix = Instruction {
        program_id: ctx.accounts.damm_v2_program.key(),
        accounts: add_liquidity_ctx.to_account_metas(None),
        data: ctx
            .accounts
            .get_add_liquidity_ix_data(AddLiquidityParameters {
                liquidity_delta,
                token_a_amount_threshold,
                token_b_amount_threshold,
            })?,
    };
    invoke(&add_liquidity_ix, &add_liquidity_ctx.to_account_infos())?;

    Ok((liquidity_delta, token_a_swapped_amount, returned_b_amount))
}

///
/// Handle zap-in on the side of token B only. We will execute a binary search on ΔL to find solutions for token_a_amount and token_b_amount.
///
/// `ΔL_min = 0`
///
/// `ΔL_max = b / ( √P - √P_min )`
///
pub fn handle_zap_on_b_in_damm_v2(_ctx: Context<ZapInDammV2Ctx>, _b: u64) -> Result<()> {
    // let trade_direction = TradeDirection::BtoA;

    Ok(())
}
