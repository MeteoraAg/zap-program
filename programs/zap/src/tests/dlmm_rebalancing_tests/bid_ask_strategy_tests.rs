use crate::{
    tests::dlmm_rebalancing_tests::utils::{
        assert_diff_amount, build_add_liquidity_params, get_bin_add_liquidity,
        get_liquidity_distribution, get_total_amount, AmountInBin,
    },
    StrategyType,
};

const STRATEGY: StrategyType = StrategyType::BidAsk;

#[test]
fn test_strategy_only_ask_side_single_bin() {
    let active_id = 100;
    let bin_step = 100;
    let total_amount_x = 100_000_000;
    let min_delta_id = 100;
    let max_delta_id = 100;
    let favor_x_in_active_id = true;

    let params = build_add_liquidity_params(
        total_amount_x,
        0,
        active_id,
        bin_step,
        min_delta_id,
        max_delta_id,
        favor_x_in_active_id,
        STRATEGY,
    );

    let amount_in_bins = get_bin_add_liquidity(&params, active_id, bin_step).unwrap();
    let amount_in_bin = &amount_in_bins[0];

    let diff = total_amount_x - amount_in_bin.amount_x;
    assert_eq!(diff, 12);
}

#[test]
fn test_strategy_only_bid_side_favour_x() {
    let active_id = 100;
    let bin_step = 10;
    let total_amount_y = 100_000_000;
    let min_delta_id = -100;
    let max_delta_id = -1;
    let favor_x_in_active_id = true;

    let params = build_add_liquidity_params(
        0,
        total_amount_y,
        active_id,
        bin_step,
        min_delta_id,
        max_delta_id,
        favor_x_in_active_id,
        STRATEGY,
    );

    let amount_in_bins = get_bin_add_liquidity(&params, active_id, bin_step).unwrap();

    let expected_min_bin_id = active_id.checked_add(min_delta_id).unwrap();
    let expected_max_bin_id = active_id.checked_add(max_delta_id).unwrap();

    let generated_min_bin_id = amount_in_bins[0].bin_id;
    let generated_max_bin_id = amount_in_bins[amount_in_bins.len() - 1].bin_id;

    assert_eq!(generated_min_bin_id, expected_min_bin_id);
    assert_eq!(generated_max_bin_id, expected_max_bin_id);
    // println!("amount_in_bins {:#?}", amount_in_bins);

    for &AmountInBin {
        bin_id,
        amount_x,
        amount_y,
    } in amount_in_bins.iter()
    {
        if bin_id < active_id {
            assert_eq!(amount_x, 0);
        }
        if bin_id == active_id {
            assert!(amount_y == 0);
        }
        if bin_id > active_id {
            assert!(amount_x > 0);
            assert!(amount_y == 0);
        }
    }

    let liquidity_distributions = get_liquidity_distribution(&amount_in_bins, bin_step);
    println!("{:?}", liquidity_distributions);

    let (amount_x, amount_y) = get_total_amount(&amount_in_bins);
    assert_eq!(amount_x, 0);
    println!("amount_y {}", amount_y);
    assert_eq!(amount_y <= total_amount_y, true);
    assert_diff_amount(total_amount_y, amount_y, 10); // less than 10bps
}

#[test]
fn test_strategy_only_bid_side_favor_y() {
    let active_id = 100;
    let bin_step = 10;
    let total_amount_y = 100_000_000;
    let min_delta_id = -100;
    let max_delta_id = 0;
    let favor_x_in_active_id = false;

    let params = build_add_liquidity_params(
        0,
        total_amount_y,
        active_id,
        bin_step,
        min_delta_id,
        max_delta_id,
        favor_x_in_active_id,
        STRATEGY,
    );

    let amount_in_bins = get_bin_add_liquidity(&params, active_id, bin_step).unwrap();

    let expected_min_bin_id = active_id.checked_add(min_delta_id).unwrap();
    let expected_max_bin_id = active_id.checked_add(max_delta_id).unwrap();

    let generated_min_bin_id = amount_in_bins[0].bin_id;
    let generated_max_bin_id = amount_in_bins[amount_in_bins.len() - 1].bin_id;

    assert_eq!(generated_min_bin_id, expected_min_bin_id);
    assert_eq!(generated_max_bin_id, expected_max_bin_id);
    // println!("amount_in_bins {:#?}", amount_in_bins);

    for &AmountInBin {
        bin_id,
        amount_x,
        amount_y,
    } in amount_in_bins.iter()
    {
        if bin_id < active_id {
            assert_eq!(amount_x, 0);
        }
        if bin_id == active_id {
            assert!(amount_x == 0);
        }
        if bin_id > active_id {
            assert!(amount_x > 0);
            assert!(amount_y == 0);
        }
    }

    let liquidity_distributions = get_liquidity_distribution(&amount_in_bins, bin_step);
    println!("{:?}", liquidity_distributions);

    let (amount_x, amount_y) = get_total_amount(&amount_in_bins);
    assert_eq!(amount_x, 0);
    println!("amount_y {}", amount_y);
    assert_eq!(amount_y <= total_amount_y, true);
    assert_diff_amount(total_amount_y, amount_y, 10); // less than 10bps
}

#[test]
fn test_strategy_only_ask_side_favour_y() {
    let active_id = 100;
    let bin_step = 10;
    let total_amount_x = 100_000_000;
    let min_delta_id = 1;
    let max_delta_id = 100;
    let favor_x_in_active_id = false;

    let params = build_add_liquidity_params(
        total_amount_x,
        0,
        active_id,
        bin_step,
        min_delta_id,
        max_delta_id,
        favor_x_in_active_id,
        STRATEGY,
    );

    let amount_in_bins = get_bin_add_liquidity(&params, active_id, bin_step).unwrap();
    let expected_min_bin_id = active_id.checked_add(min_delta_id).unwrap();
    let expected_max_bin_id = active_id.checked_add(max_delta_id).unwrap();

    let generated_min_bin_id = amount_in_bins[0].bin_id;
    let generated_max_bin_id = amount_in_bins[amount_in_bins.len() - 1].bin_id;

    assert_eq!(generated_min_bin_id, expected_min_bin_id);
    assert_eq!(generated_max_bin_id, expected_max_bin_id);

    for &AmountInBin {
        bin_id,
        amount_x,
        amount_y,
    } in amount_in_bins.iter()
    {
        if bin_id < active_id {
            assert_eq!(amount_x, 0);
            assert!(amount_y > 0);
        }
        if bin_id == active_id {
            assert!(amount_y == 0);
        }
        if bin_id > active_id {
            assert!(amount_y == 0);
        }
    }

    let liquidity_distributions = get_liquidity_distribution(&amount_in_bins, bin_step);
    println!("liquidity distribution {:?}", liquidity_distributions);

    let (amount_x, amount_y) = get_total_amount(&amount_in_bins);
    assert_eq!(amount_y, 0);
    println!("amount_x {}", amount_x);
    assert_eq!(amount_x <= total_amount_x, true);
    assert_diff_amount(total_amount_x, amount_x, 10); // less than 10bps
}

#[test]
fn test_strategy_only_ask_side_favour_x() {
    let active_id = 100;
    let bin_step = 10;
    let total_amount_x = 100_000_000;
    let min_delta_id = 1;
    let max_delta_id = 100;
    let favor_x_in_active_id = true;

    let params = build_add_liquidity_params(
        total_amount_x,
        0,
        active_id,
        bin_step,
        min_delta_id,
        max_delta_id,
        favor_x_in_active_id,
        STRATEGY,
    );

    let amount_in_bins = get_bin_add_liquidity(&params, active_id, bin_step).unwrap();
    let expected_min_bin_id = active_id.checked_add(min_delta_id).unwrap();
    let expected_max_bin_id = active_id.checked_add(max_delta_id).unwrap();

    let generated_min_bin_id = amount_in_bins[0].bin_id;
    let generated_max_bin_id = amount_in_bins[amount_in_bins.len() - 1].bin_id;

    assert_eq!(generated_min_bin_id, expected_min_bin_id);
    assert_eq!(generated_max_bin_id, expected_max_bin_id);

    for &AmountInBin {
        bin_id,
        amount_x,
        amount_y,
    } in amount_in_bins.iter()
    {
        if bin_id < active_id {
            assert_eq!(amount_x, 0);
            assert!(amount_y > 0);
        }
        if bin_id == active_id {
            assert!(amount_y == 0);
        }
        if bin_id > active_id {
            assert!(amount_y == 0);
        }
    }

    let liquidity_distributions = get_liquidity_distribution(&amount_in_bins, bin_step);
    println!("liquidity distribution {:?}", liquidity_distributions);

    let (amount_x, amount_y) = get_total_amount(&amount_in_bins);
    assert_eq!(amount_y, 0);
    println!("amount_x {}", amount_x);
    assert_eq!(amount_x <= total_amount_x, true);
    assert_diff_amount(total_amount_x, amount_x, 10); // less than 10bps
}

#[test]
fn test_strategy_both_sides_favor_y() {
    let active_id = 100;
    let bin_step = 10;
    let total_amount_x = 100_000_000;
    let total_amount_y = 200_000_000;
    let min_delta_id = -50;
    let max_delta_id = 100;
    let favor_x_in_active_id = false;

    let params = build_add_liquidity_params(
        total_amount_x,
        total_amount_y,
        active_id,
        bin_step,
        min_delta_id,
        max_delta_id,
        favor_x_in_active_id,
        STRATEGY,
    );

    let amount_in_bins = get_bin_add_liquidity(&params, active_id, bin_step).unwrap();
    let expected_min_bin_id = active_id.checked_add(min_delta_id).unwrap();
    let expected_max_bin_id = active_id.checked_add(max_delta_id).unwrap();

    let generated_min_bin_id = amount_in_bins[0].bin_id;
    let generated_max_bin_id = amount_in_bins[amount_in_bins.len() - 1].bin_id;

    assert_eq!(generated_min_bin_id, expected_min_bin_id);
    assert_eq!(generated_max_bin_id, expected_max_bin_id);

    for &AmountInBin {
        bin_id,
        amount_x,
        amount_y,
    } in amount_in_bins.iter()
    {
        if bin_id < active_id {
            assert_eq!(amount_x, 0);
        }
        if bin_id == active_id {
            assert!(amount_x == 0);
        }
        if bin_id > active_id {
            assert!(amount_y == 0);
        }
    }

    let liquidity_distributions = get_liquidity_distribution(&amount_in_bins, bin_step);
    println!("{:?}", liquidity_distributions);

    let (amount_x, amount_y) = get_total_amount(&amount_in_bins);
    println!("amount_x {} amount_y {}", amount_x, amount_y);
    assert_eq!(amount_x <= total_amount_x, true);
    assert_eq!(amount_y <= total_amount_y, true);

    assert_diff_amount(total_amount_x, amount_x, 10); // less than 10bps
    assert_diff_amount(total_amount_y, amount_y, 10); // less than 10bps
}

#[test]
fn test_strategy_both_sides_favor_x() {
    let active_id = 100;
    let bin_step = 10;
    let total_amount_x = 100_000_000;
    let total_amount_y = 200_000_000;
    let min_delta_id = -50;
    let max_delta_id = 100;
    let favor_x_in_active_id = true;

    let params = build_add_liquidity_params(
        total_amount_x,
        total_amount_y,
        active_id,
        bin_step,
        min_delta_id,
        max_delta_id,
        favor_x_in_active_id,
        STRATEGY,
    );

    let amount_in_bins = get_bin_add_liquidity(&params, active_id, bin_step).unwrap();
    let expected_min_bin_id = active_id.checked_add(min_delta_id).unwrap();
    let expected_max_bin_id = active_id.checked_add(max_delta_id).unwrap();

    let generated_min_bin_id = amount_in_bins[0].bin_id;
    let generated_max_bin_id = amount_in_bins[amount_in_bins.len() - 1].bin_id;

    assert_eq!(generated_min_bin_id, expected_min_bin_id);
    assert_eq!(generated_max_bin_id, expected_max_bin_id);

    for &AmountInBin {
        bin_id,
        amount_x,
        amount_y,
    } in amount_in_bins.iter()
    {
        if bin_id < active_id {
            assert_eq!(amount_x, 0);
        }
        if bin_id == active_id {
            assert!(amount_y == 0);
        }
        if bin_id > active_id {
            assert!(amount_y == 0);
        }
    }

    let liquidity_distributions = get_liquidity_distribution(&amount_in_bins, bin_step);
    println!("{:?}", liquidity_distributions);

    let (amount_x, amount_y) = get_total_amount(&amount_in_bins);
    println!("amount_x {} amount_y {}", amount_x, amount_y);
    assert_eq!(amount_x <= total_amount_x, true);
    assert_eq!(amount_y <= total_amount_y, true);

    assert_diff_amount(total_amount_x, amount_x, 10); // less than 10bps
    assert_diff_amount(total_amount_y, amount_y, 10); // less than 10bps
}
