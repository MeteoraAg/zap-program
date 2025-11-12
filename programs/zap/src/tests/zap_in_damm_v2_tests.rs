use std::fs;

use anchor_spl::token_2022::spl_token_2022::extension::transfer_fee::TransferFee;
use damm_v2::{params::swap::TradeDirection, state::Pool};

use crate::{calculate_swap_amount, TransferFeeCalculator};

pub const SOL_USDC_CL_ADDRESS: &str = "8Pm2kZpnxD3hoMmt4bjStX2Pw2Z9abpbHzZxMPqxPmie";

fn get_pool_account(pool_address: &str) -> Pool {
    let path = format!("../../fixtures/{}.bin", pool_address);
    let account_data = fs::read(&path).expect("Failed to read account data");

    let mut data_without_discriminator = account_data[8..].to_vec();
    let &pool: &Pool = bytemuck::from_bytes(&mut data_without_discriminator);

    pool
}

#[test]
fn test_calculate_swap_result() {
    let pool = get_pool_account(SOL_USDC_CL_ADDRESS);

    let trade_direction = TradeDirection::AtoB;
    let remaining_amount = 1_000_000_000; //1 sol
    let current_point = 1762837786;
    let transfer_fee_calculator = TransferFeeCalculator {
        epoch_transfer_fee: TransferFee::default(),
        no_transfer_fee_extension: true,
    };
    let swap_amount = calculate_swap_amount(
        &pool,
        &transfer_fee_calculator,
        &transfer_fee_calculator,
        remaining_amount,
        trade_direction,
        current_point,
    )
    .unwrap();

    // test swap and add liquidity in meteora website
    // https://app.meteora.ag/dammv2/8Pm2kZpnxD3hoMmt4bjStX2Pw2Z9abpbHzZxMPqxPmie?referrer=home
    println!("swap_amount: {}", swap_amount);
}
