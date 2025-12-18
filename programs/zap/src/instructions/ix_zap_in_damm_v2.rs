use anchor_lang::prelude::*;
use anchor_spl::{token::accessor, token_interface::Mint};
use damm_v2::{
    activation_handler::ActivationHandler, params::swap::TradeDirection, state::Pool,
    AddLiquidityParameters, SwapMode, SwapParameters2,
};

use crate::{
    damm_v2_utils::{calculate_swap_amount, get_price_change_bps},
    error::ZapError,
    new_transfer_fee_calculator, UserLedger,
};

#[derive(Accounts)]
pub struct ZapInDammv2Ctx<'info> {
    #[account(mut, has_one = owner)]
    pub ledger: AccountLoader<'info, UserLedger>,

    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,

    /// CHECK: pool_authority, will be checked when we call function in damm v2
    pub pool_authority: UncheckedAccount<'info>,

    /// CHECK: position, will be checked when we call function in damm v2
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// CHECK: The user token a account
    #[account(mut)]
    pub token_a_account: UncheckedAccount<'info>,

    /// CHECK: The user token b account
    #[account(mut)]
    pub token_b_account: UncheckedAccount<'info>,

    /// CHECK: token_a_vault, will be checked when we call function in damm v2
    #[account(mut)]
    pub token_a_vault: UncheckedAccount<'info>,

    /// CHECK: token_b_vault, will be checked when we call function in damm v2
    #[account(mut)]
    pub token_b_vault: UncheckedAccount<'info>,

    /// CHECK: The mint of token a
    pub token_a_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: The mint of token b
    pub token_b_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: position_nft_account, will be checked when we call function in damm v2
    pub position_nft_account: UncheckedAccount<'info>,

    /// owner of position
    pub owner: Signer<'info>,

    /// CHECK: Token a program
    pub token_a_program: UncheckedAccount<'info>,

    /// CHECK: Token b program
    pub token_b_program: UncheckedAccount<'info>,

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
                amount_1: 0,
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
                token_a_amount_threshold: u64::MAX,
                token_b_amount_threshold: u64::MAX,
            },
        )?;
        Ok(())
    }
}

pub fn handle_zap_in_damm_v2(
    ctx: Context<ZapInDammv2Ctx>,
    pre_sqrt_price: u128,           // sqrt price user observe in local
    max_sqrt_price_change_bps: u32, // max sqrt price change after swap
) -> Result<()> {
    let mut ledger = ctx.accounts.ledger.load_mut()?;
    let max_deposit_a_amount = ledger.amount_a;
    let max_deposit_b_amount = ledger.amount_b;
    // 1. we add liquidity firstly, so later if we need swap, user could get some fees back
    let pool = ctx.accounts.pool.load()?;
    let token_a_account_ai = ctx.accounts.token_a_account.to_account_info();
    let token_b_account_ai = ctx.accounts.token_b_account.to_account_info();

    let token_a_transfer_fee_calculator = new_transfer_fee_calculator(&ctx.accounts.token_a_mint)?;
    let token_b_transfer_fee_calculator = new_transfer_fee_calculator(&ctx.accounts.token_b_mint)?;

    let user_amount_a_1 = accessor::amount(&token_a_account_ai)?;
    let user_amount_b_1 = accessor::amount(&token_b_account_ai)?;

    let (liquidity, trade_direction) = ledger.get_liquidity_from_amounts_and_trade_direction(
        &token_a_transfer_fee_calculator,
        &token_b_transfer_fee_calculator,
        pool.sqrt_price,
        pool.sqrt_min_price,
        pool.sqrt_max_price,
    )?;

    drop(pool);

    if liquidity > 0 {
        ctx.accounts.add_liquidity(liquidity)?;
    }

    // 2. We check if user is still having some balance left, we will swap before they could add remaining liquidity
    let user_amount_a_2 = accessor::amount(&token_a_account_ai)?;
    let user_amount_b_2 = accessor::amount(&token_b_account_ai)?;

    ledger.update_ledger_balances(
        user_amount_a_1,
        user_amount_a_2,
        user_amount_b_1,
        user_amount_b_2,
    )?;

    let remaining_amount = if trade_direction == TradeDirection::AtoB {
        ledger.amount_a
    } else {
        ledger.amount_b
    };

    if remaining_amount > 0 {
        let pool = ctx.accounts.pool.load()?;
        let current_point = ActivationHandler::get_current_point(pool.activation_type)?;
        let swap_result = calculate_swap_amount(
            &pool,
            &token_a_transfer_fee_calculator,
            &token_b_transfer_fee_calculator,
            remaining_amount,
            trade_direction,
            current_point,
            pool.pool_fees.init_sqrt_price,
            pool.sqrt_price,
        );
        match swap_result {
            Ok((swap_in_amount, swap_out_amount)) => {
                if swap_in_amount == 0 || swap_out_amount == 0 {
                    msg!(
                        "max_deposit_amounts: {} {}, remaining_amounts: {} {}, swap_amounts: {} {}",
                        max_deposit_a_amount,
                        max_deposit_b_amount,
                        ledger.amount_a,
                        ledger.amount_b,
                        swap_in_amount,
                        swap_out_amount
                    );
                    return Ok(()); // no need to swap, just return
                }
                drop(pool);
                ctx.accounts.swap(swap_in_amount, trade_direction)?;
            }
            Err(err) => {
                // if calculation fail, we just skip swap and add liquidity with remaining amount
                msg!("Calculate swap amount error: {:?}", err);
                msg!(
                    "max_deposit_amounts: {} {}, remaining_amounts: {} {}",
                    max_deposit_a_amount,
                    max_deposit_b_amount,
                    ledger.amount_a,
                    ledger.amount_b
                );
                return Ok(());
            }
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
    let user_amount_a_3 = accessor::amount(&token_a_account_ai)?;
    let user_amount_b_3 = accessor::amount(&token_b_account_ai)?;

    ledger.update_ledger_balances(
        user_amount_a_2,
        user_amount_a_3,
        user_amount_b_2,
        user_amount_b_3,
    )?;

    let (liquidity, _trade_direction) = ledger.get_liquidity_from_amounts_and_trade_direction(
        &token_a_transfer_fee_calculator,
        &token_b_transfer_fee_calculator,
        pool.sqrt_price,
        pool.sqrt_min_price,
        pool.sqrt_max_price,
    )?;

    if liquidity > 0 {
        drop(pool);
        ctx.accounts.add_liquidity(liquidity)?;
    }

    let user_amount_a_4 = accessor::amount(&token_a_account_ai)?;
    let user_amount_b_4 = accessor::amount(&token_b_account_ai)?;

    ledger.update_ledger_balances(
        user_amount_a_3,
        user_amount_a_4,
        user_amount_b_3,
        user_amount_b_4,
    )?;

    // log will be truncated, shouldn't rely on that
    msg!(
        "max_deposit_amounts: {} {}, remaining_amounts: {} {}",
        max_deposit_a_amount,
        max_deposit_b_amount,
        ledger.amount_a,
        ledger.amount_b
    );

    Ok(())
}
