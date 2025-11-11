use crate::{
    error::ZapError, StrategyType, UnparsedAddLiquidityParams, UserLedger, ZapInRebalancingParams,
};
use anchor_lang::prelude::*;
use anchor_spl::{token::accessor, token_interface::Mint};
use damm_v2::{safe_math::SafeMath, token::calculate_transfer_fee_excluded_amount};
use dlmm::{
    accounts::LbPair,
    types::{AddLiquidityParams, RebalanceLiquidityParams, RemainingAccountsInfo},
};

#[derive(Accounts)]
pub struct ZapInDlmmForUnintializedPositionCtx<'info> {
    #[account(mut, has_one = owner)]
    pub ledger: AccountLoader<'info, UserLedger>,

    /// lb pair
    #[account(mut)]
    pub lb_pair: AccountLoader<'info, LbPair>,

    /// user position
    /// Check it is different from owner to advoid user to pass owner address wrongly
    #[account(mut, constraint = position.key.ne(owner.key))]
    pub position: Signer<'info>,

    /// CHECK: will be validated in dlmm program
    #[account(mut)]
    pub bin_array_bitmap_extension: UncheckedAccount<'info>,

    /// CHECK: will be validated in dlmm program
    #[account(mut)]
    pub user_token_x: UncheckedAccount<'info>,

    /// CHECK: will be validated in dlmm program
    #[account(mut)]
    pub user_token_y: UncheckedAccount<'info>,

    /// CHECK: will be validated in dlmm program
    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    /// CHECK: will be validated in dlmm program
    #[account(mut)]
    pub reserve_y: UncheckedAccount<'info>,

    pub token_x_mint: InterfaceAccount<'info, Mint>,
    pub token_y_mint: InterfaceAccount<'info, Mint>,

    pub dlmm_program: Program<'info, dlmm::program::LbClmm>,

    /// owner of position
    pub owner: Signer<'info>,

    #[account(mut)]
    pub rent_payer: Signer<'info>,

    /// CHECK: will be validated in dlmm program
    pub token_x_program: UncheckedAccount<'info>,

    /// CHECK: will be validated in dlmm program
    pub token_y_program: UncheckedAccount<'info>,

    /// CHECK: will be validated in dlmm program
    pub memo_program: UncheckedAccount<'info>,

    /// CHECK: will be validated in dlmm program
    pub system_program: UncheckedAccount<'info>,

    /// CHECK: will be validated in dlmm program
    pub dlmm_event_authority: UncheckedAccount<'info>,
}

impl<'info> ZapInDlmmForUnintializedPositionCtx<'info> {
    fn initialize_position(&self, lower_bin_id: i32, width: i32) -> Result<()> {
        dlmm::cpi::initialize_position2(
            CpiContext::new(
                self.dlmm_program.to_account_info(),
                dlmm::cpi::accounts::InitializePosition2 {
                    payer: self.rent_payer.to_account_info(),
                    position: self.position.to_account_info(),
                    lb_pair: self.lb_pair.to_account_info(),
                    owner: self.owner.to_account_info(),
                    program: self.dlmm_program.to_account_info(),
                    event_authority: self.dlmm_event_authority.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
            ),
            lower_bin_id,
            width,
        )?;
        Ok(())
    }
}

pub fn handle_zap_in_dlmm_for_uninitialized_position<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, ZapInDlmmForUnintializedPositionCtx<'info>>,
    bin_delta: u8,
    active_id: i32,
    max_active_bin_slippage: u16,
    favor_x_in_active_id: bool,
    strategy: StrategyType,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let mut ledger = ctx.accounts.ledger.load_mut()?;
    let max_deposit_x_amount = ledger.amount_a;
    let max_deposit_y_amount = ledger.amount_b;

    let token_x_account_ai = ctx.accounts.user_token_x.to_account_info();
    let token_y_account_ai = ctx.accounts.user_token_y.to_account_info();
    let pre_user_amount_x = accessor::amount(&token_x_account_ai)?;
    let pre_user_amount_y = accessor::amount(&token_y_account_ai)?;

    let lb_pair = ctx.accounts.lb_pair.load()?;

    // create position wth bin_delta in left side, and bin_delta in right side
    let bin_delta: i32 = bin_delta.into();
    let lower_bin_id = lb_pair.active_id.safe_sub(bin_delta)?;
    let width = bin_delta.safe_mul(2)?.safe_add(1)?;

    // check the position is not initialized yet
    require!(
        ctx.accounts.position.owner.eq(&Pubkey::default()) && ctx.accounts.position.data_is_empty(),
        ZapError::InvalidPosition
    );

    // initialize position
    drop(lb_pair);
    ctx.accounts.initialize_position(lower_bin_id, width)?;

    // rebalancing
    // TODO refactor to save more code with endpoint zap in dlmm for initialized position
    let lb_pair = ctx.accounts.lb_pair.load()?;
    let lb_pair_active_id = lb_pair.active_id;
    let min_delta_id = lower_bin_id.safe_sub(lb_pair_active_id)?;
    let upper_bin_id = lower_bin_id.safe_add(width)?.safe_sub(1)?;
    let max_delta_id = upper_bin_id.safe_sub(lb_pair_active_id)?;

    let amount_x =
        calculate_transfer_fee_excluded_amount(&ctx.accounts.token_x_mint, max_deposit_x_amount)?
            .amount;
    let amount_y =
        calculate_transfer_fee_excluded_amount(&ctx.accounts.token_y_mint, max_deposit_y_amount)?
            .amount;

    let params = ZapInRebalancingParams {
        amount_x,
        amount_y,
        active_id: lb_pair_active_id,
        bin_step: lb_pair.bin_step,
        min_delta_id,
        max_delta_id,
        favor_x_in_active_id,
        strategy,
    };

    let UnparsedAddLiquidityParams {
        x0,
        y0,
        delta_x,
        delta_y,
        bit_flag,
    } = params.get_rebalancing_params()?;

    let params = RebalanceLiquidityParams {
        active_id,
        max_active_bin_slippage,
        should_claim_fee: false,
        should_claim_reward: false,
        min_withdraw_x_amount: 0,
        max_deposit_x_amount,
        min_withdraw_y_amount: 0,
        max_deposit_y_amount,
        shrink_mode: 3, // we dont allow to shrink in both side
        padding: [0; 31],
        removes: vec![],
        adds: vec![AddLiquidityParams {
            min_delta_id,
            max_delta_id,
            x0,
            y0,
            delta_x,
            delta_y,
            favor_x_in_active_id,
            bit_flag,
            ..Default::default()
        }],
    };

    drop(lb_pair);

    dlmm::cpi::rebalance_liquidity(
        CpiContext::new(
            ctx.accounts.dlmm_program.to_account_info(),
            dlmm::cpi::accounts::RebalanceLiquidity {
                position: ctx.accounts.position.to_account_info(),
                lb_pair: ctx.accounts.lb_pair.to_account_info(),
                // TODO need to check bin array bitmap extension
                bin_array_bitmap_extension: Some(
                    ctx.accounts.bin_array_bitmap_extension.to_account_info(),
                ),
                owner: ctx.accounts.owner.to_account_info(),
                user_token_x: ctx.accounts.user_token_x.to_account_info(),
                user_token_y: ctx.accounts.user_token_y.to_account_info(),
                reserve_x: ctx.accounts.reserve_x.to_account_info(),
                reserve_y: ctx.accounts.reserve_y.to_account_info(),
                token_x_mint: ctx.accounts.token_x_mint.to_account_info(),
                token_y_mint: ctx.accounts.token_y_mint.to_account_info(),
                rent_payer: ctx.accounts.rent_payer.to_account_info(),
                token_x_program: ctx.accounts.token_x_program.to_account_info(),
                token_y_program: ctx.accounts.token_y_program.to_account_info(),
                memo_program: ctx.accounts.memo_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                program: ctx.accounts.dlmm_program.to_account_info(),
                event_authority: ctx.accounts.dlmm_event_authority.to_account_info(),
            },
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
        params,
        remaining_accounts_info,
    )?;

    let post_user_amount_x = accessor::amount(&token_x_account_ai)?;
    let post_user_amount_y = accessor::amount(&token_y_account_ai)?;

    ledger.update_ledger_balances(
        pre_user_amount_x,
        post_user_amount_x,
        pre_user_amount_y,
        post_user_amount_y,
    )?;

    // log will be truncated, shouldn't rely on that
    msg!(
        "max_deposit_amounts: {} {}, remaining_amounts: {} {}",
        max_deposit_x_amount,
        max_deposit_y_amount,
        ledger.amount_a,
        ledger.amount_b
    );

    Ok(())
}
