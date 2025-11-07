use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface};
use damm_v2::{
    activation_handler::ActivationHandler, params::swap::TradeDirection, state::Pool,
    AddLiquidityParameters, SwapMode, SwapParameters2,
};

use crate::{
    damm_v2_ultils::{
        calculate_swap_amount, get_liquidity_from_amounts_and_trade_direction, get_price_change_bps,
    },
    error::ZapError,
};

#[derive(Accounts)]
pub struct ZapInDammv2Ctx<'info> {
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,

    /// CHECK: pool_authority, will be checked when we call function in damm v2
    pub pool_authority: UncheckedAccount<'info>,

    /// CHECK: position, will be checked when we call function in damm v2
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// The user token a account
    /// TODO we could change it to unchecked account
    #[account(mut)]
    pub token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The user token b account
    /// TODO we could change it to unchecked account
    #[account(mut)]
    pub token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: token_a_vault, will be checked when we call function in damm v2
    #[account(mut)]
    pub token_a_vault: UncheckedAccount<'info>,

    /// CHECK: token_b_vault, will be checked when we call function in damm v2
    #[account(mut)]
    pub token_b_vault: UncheckedAccount<'info>,

    /// CHECK: The mint of token a
    pub token_a_mint: UncheckedAccount<'info>,

    /// CHECK: The mint of token b
    pub token_b_mint: UncheckedAccount<'info>,

    /// CHECK: position_nft_account, will be checked when we call function in damm v2
    pub position_nft_account: UncheckedAccount<'info>,

    /// owner of position
    pub owner: Signer<'info>,

    /// Token a program
    pub token_a_program: Interface<'info, TokenInterface>,

    /// Token b program
    pub token_b_program: Interface<'info, TokenInterface>,

    pub damm_program: Program<'info, damm_v2::program::CpAmm>,

    /// CHECK: damm event authority, will be check in damm v2 functions
    pub damm_event_authority: UncheckedAccount<'info>,
}

impl<'info> ZapInDammv2Ctx<'info> {
    fn swap(&self, amount: u64, trade_direction: TradeDirection) -> Result<()> {
        let (input_token_account, output_token_account) = if trade_direction == TradeDirection::AtoB
        {
            (
                self.token_a_account.to_account_info(),
                self.token_b_account.to_account_info(),
            )
        } else {
            (
                self.token_b_account.to_account_info(),
                self.token_a_account.to_account_info(),
            )
        };
        damm_v2::cpi::swap2(
            CpiContext::new(
                self.damm_program.to_account_info(),
                damm_v2::cpi::accounts::SwapCtx {
                    pool_authority: self.pool_authority.to_account_info(),
                    input_token_account,
                    output_token_account,
                    pool: self.pool.to_account_info(),
                    token_a_vault: self.token_a_vault.to_account_info(),
                    token_b_vault: self.token_b_vault.to_account_info(),
                    token_a_mint: self.token_a_mint.to_account_info(),
                    token_b_mint: self.token_b_mint.to_account_info(),
                    token_a_program: self.token_a_program.to_account_info(),
                    token_b_program: self.token_b_program.to_account_info(),
                    event_authority: self.damm_event_authority.to_account_info(),
                    program: self.damm_program.to_account_info(),
                    payer: self.owner.to_account_info(),
                    referral_token_account: None, // TODO check whether it should be some(damm_program)
                },
            ),
            SwapParameters2 {
                amount_0: amount,
                amount_1: 0, // TODO do we need to care for slippage rate
                swap_mode: SwapMode::ExactIn.into(),
            },
        )?;
        Ok(())
    }

    fn add_liquidity(&self, liquidity: u128) -> Result<()> {
        damm_v2::cpi::add_liquidity(
            CpiContext::new(
                self.damm_program.to_account_info(),
                damm_v2::cpi::accounts::AddLiquidityCtx {
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
                    event_authority: self.damm_event_authority.to_account_info(),
                    program: self.damm_program.to_account_info(),
                },
            ),
            AddLiquidityParameters {
                liquidity_delta: liquidity,
                token_a_amount_threshold: u64::MAX, // TODO should we take care for that
                token_b_amount_threshold: u64::MAX,
            },
        )?;
        Ok(())
    }
}

pub fn handle_zap_in_damm_v2(
    ctx: Context<ZapInDammv2Ctx>,
    max_sqrt_price_change_bps: u32,
) -> Result<()> {
    // 1. we add liquidity firstly, so later if we need swap, user could get some fees back
    let pool = ctx.accounts.pool.load()?;
    let pre_sqrt_price = pool.sqrt_price;
    let token_a_amount = ctx.accounts.token_a_account.amount;
    let token_b_amount = ctx.accounts.token_b_account.amount;
    let (liquidity, trade_direction) = get_liquidity_from_amounts_and_trade_direction(
        token_a_amount,
        token_b_amount,
        pool.sqrt_price,
        pool.sqrt_min_price,
        pool.sqrt_max_price,
    )?;

    if liquidity > 0 {
        drop(pool);
        ctx.accounts.add_liquidity(liquidity)?;
    }

    // 2. We check if user is still having some balance left, we will swap before they could add remanining liquidity
    let remaining_amount = if trade_direction == TradeDirection::AtoB {
        ctx.accounts.token_a_account.reload()?;
        ctx.accounts.token_a_account.amount
    } else {
        ctx.accounts.token_b_account.reload()?;
        ctx.accounts.token_b_account.amount
    };
    if remaining_amount > 0 {
        let pool = ctx.accounts.pool.load()?;
        let current_point = ActivationHandler::get_current_point(pool.activation_type)?;
        let swap_amount =
            calculate_swap_amount(&pool, remaining_amount, trade_direction, current_point)?;
        if swap_amount > 0 {
            drop(pool);
            ctx.accounts.swap(swap_amount, trade_direction)?;
        }
    }

    // validate pool price after swap
    let pool = ctx.accounts.pool.load()?;
    let post_sqrt_price = pool.sqrt_price;
    // validate price change
    let sqrt_price_change_bps = get_price_change_bps(pre_sqrt_price, post_sqrt_price)?;
    require!(
        sqrt_price_change_bps <= max_sqrt_price_change_bps,
        ZapError::ExceededSlippage
    );

    // 3. Do final add liquidity
    // reload balance
    ctx.accounts.token_a_account.reload()?;
    ctx.accounts.token_b_account.reload()?;

    let token_a_amount = ctx.accounts.token_a_account.amount;
    let token_b_amount = ctx.accounts.token_b_account.amount;

    let (liquidity, _trade_direction) = get_liquidity_from_amounts_and_trade_direction(
        token_a_amount,
        token_b_amount,
        pool.sqrt_price,
        pool.sqrt_min_price,
        pool.sqrt_max_price,
    )?;

    if liquidity > 0 {
        drop(pool);
        ctx.accounts.add_liquidity(liquidity)?;
    }

    // TODO emit event?

    Ok(())
}
