use jupiter::types::{RoutePlanStep, Swap};

use crate::jup_v6_zap::ensure_route_plan_fully_converges;

#[test]
fn test_ensure_route_plan_fully_converges_success() {
    let route_plan_1_market = vec![RoutePlanStep {
        swap: Swap::Meteora,
        percent: 100,
        input_index: 0,
        output_index: 1,
    }];
    assert!(ensure_route_plan_fully_converges(&route_plan_1_market).is_ok());

    // Layer 1 Split into 3:    0 -> 1 (60%), 0 -> 2 (5%), 0 -> 3 (35%)
    // Layer 2 Split into 3:    1 -> 5 (50%), 1 -> 6 (50%), 2 -> 6 (100%), 3 -> 6 (33%), 3 -> 7 (67%)
    // Layer 3 Converge into 1: 5 -> 8 (100%), 6 -> 8 (100%), 7 -> 8 (100%)
    let route_plan_funnel = vec![
        // Layer 1
        RoutePlanStep {
            swap: Swap::MeteoraDammV2,
            percent: 60,
            input_index: 0,
            output_index: 1,
        },
        RoutePlanStep {
            swap: Swap::Raydium,
            percent: 5,
            input_index: 0,
            output_index: 2,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDlmm,
            percent: 35,
            input_index: 0,
            output_index: 3,
        },
        // Layer 2
        RoutePlanStep {
            swap: Swap::RaydiumClmm,
            percent: 50,
            input_index: 1,
            output_index: 5,
        },
        RoutePlanStep {
            swap: Swap::Mercurial,
            percent: 50,
            input_index: 1,
            output_index: 6,
        },
        RoutePlanStep {
            swap: Swap::RaydiumClmmV2,
            percent: 100,
            input_index: 2,
            output_index: 6,
        },
        RoutePlanStep {
            swap: Swap::RaydiumV2,
            percent: 33,
            input_index: 3,
            output_index: 6,
        },
        RoutePlanStep {
            swap: Swap::RaydiumCP,
            percent: 67,
            input_index: 3,
            output_index: 7,
        },
        // Layer 3
        RoutePlanStep {
            swap: Swap::Meteora,
            percent: 100,
            input_index: 5,
            output_index: 8,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDammV2WithRemainingAccounts,
            percent: 100,
            input_index: 6,
            output_index: 8,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDammV2,
            percent: 100,
            input_index: 7,
            output_index: 8,
        },
    ];
    assert!(ensure_route_plan_fully_converges(&route_plan_funnel).is_ok());

    let route_plan_sequential = vec![
        RoutePlanStep {
            swap: Swap::Meteora,
            percent: 100,
            input_index: 0,
            output_index: 1,
        },
        RoutePlanStep {
            swap: Swap::RaydiumClmm,
            percent: 100,
            input_index: 1,
            output_index: 2,
        },
        RoutePlanStep {
            swap: Swap::Mercurial,
            percent: 100,
            input_index: 2,
            output_index: 3,
        },
        RoutePlanStep {
            swap: Swap::RaydiumCP,
            percent: 100,
            input_index: 3,
            output_index: 4,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDammV2,
            percent: 100,
            input_index: 4,
            output_index: 5,
        },
    ];
    assert!(ensure_route_plan_fully_converges(&route_plan_sequential).is_ok());

    // diamond: 1 -> 2 -> 4 -> 2 -> 1
    // Layer 1: 0 -> 1 (60%), 0 -> 2 (40%)
    // Layer 2: 1 -> 3 (70%), 1 -> 4 (30%), 2 -> 5 (25%), 2 -> 6 (75%)
    // Layer 3: 3 -> 7 (100%), 4 -> 7 (100%), 5 -> 8 (100%), 6 -> 8 (100%)
    // Layer 4: 7 -> 9 (100%), 8 -> 9 (100%)
    let route_plan_diamond = vec![
        // Layer 1: split 1 -> 2
        RoutePlanStep {
            swap: Swap::Meteora,
            percent: 60,
            input_index: 0,
            output_index: 1,
        },
        RoutePlanStep {
            swap: Swap::Raydium,
            percent: 40,
            input_index: 0,
            output_index: 2,
        },
        // Layer 2: split 2 -> 4
        RoutePlanStep {
            swap: Swap::MeteoraDlmm,
            percent: 70,
            input_index: 1,
            output_index: 3,
        },
        RoutePlanStep {
            swap: Swap::RaydiumCP,
            percent: 30,
            input_index: 1,
            output_index: 4,
        },
        RoutePlanStep {
            swap: Swap::RaydiumClmm,
            percent: 25,
            input_index: 2,
            output_index: 5,
        },
        RoutePlanStep {
            swap: Swap::Mercurial,
            percent: 75,
            input_index: 2,
            output_index: 6,
        },
        // Layer 3: converge 4 -> 2
        RoutePlanStep {
            swap: Swap::MeteoraDammV2,
            percent: 100,
            input_index: 3,
            output_index: 7,
        },
        RoutePlanStep {
            swap: Swap::RaydiumV2,
            percent: 100,
            input_index: 4,
            output_index: 7,
        },
        RoutePlanStep {
            swap: Swap::RaydiumClmmV2,
            percent: 100,
            input_index: 5,
            output_index: 8,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDammV2WithRemainingAccounts,
            percent: 100,
            input_index: 6,
            output_index: 8,
        },
        // Layer 4: converge 2 -> 1
        RoutePlanStep {
            swap: Swap::Meteora,
            percent: 100,
            input_index: 7,
            output_index: 9,
        },
        RoutePlanStep {
            swap: Swap::Raydium,
            percent: 100,
            input_index: 8,
            output_index: 9,
        },
    ];
    assert!(ensure_route_plan_fully_converges(&route_plan_diamond).is_ok());
}

#[test]
fn test_ensure_route_plan_fully_converges_failure() {
    let route_plan_1_market = vec![RoutePlanStep {
        swap: Swap::Meteora,
        percent: 50,
        input_index: 0,
        output_index: 1,
    }];
    assert!(ensure_route_plan_fully_converges(&route_plan_1_market).is_err());

    let route_plan_empty: Vec<RoutePlanStep> = vec![];
    assert!(ensure_route_plan_fully_converges(&route_plan_empty).is_err());

    let route_plan_cycle = vec![
        RoutePlanStep {
            swap: Swap::Meteora,
            percent: 100,
            input_index: 0,
            output_index: 1,
        },
        RoutePlanStep {
            swap: Swap::Raydium,
            percent: 100,
            input_index: 1,
            output_index: 0,
        },
    ];
    assert!(ensure_route_plan_fully_converges(&route_plan_cycle).is_err());

    let route_plan_over_100_pct = vec![
        RoutePlanStep {
            swap: Swap::Meteora,
            percent: 60,
            input_index: 0,
            output_index: 1,
        },
        RoutePlanStep {
            swap: Swap::Raydium,
            percent: 60,
            input_index: 0,
            output_index: 2,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDlmm,
            percent: 100,
            input_index: 1,
            output_index: 3,
        },
        RoutePlanStep {
            swap: Swap::RaydiumCP,
            percent: 100,
            input_index: 2,
            output_index: 3,
        },
    ];
    assert!(ensure_route_plan_fully_converges(&route_plan_over_100_pct).is_err());

    let route_plan_sequential = vec![
        RoutePlanStep {
            swap: Swap::Meteora,
            percent: 100,
            input_index: 0,
            output_index: 1,
        },
        RoutePlanStep {
            swap: Swap::RaydiumClmm,
            percent: 100,
            input_index: 1,
            output_index: 2,
        },
        RoutePlanStep {
            swap: Swap::Mercurial,
            percent: 80, // not 100
            input_index: 2,
            output_index: 3,
        },
        RoutePlanStep {
            swap: Swap::RaydiumCP,
            percent: 100,
            input_index: 3,
            output_index: 4,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDammV2,
            percent: 100,
            input_index: 4,
            output_index: 5,
        },
    ];
    assert!(ensure_route_plan_fully_converges(&route_plan_sequential).is_err());

    // Same as happy diamond but layer 4 outputs diverge: 7 -> 9, 8 -> 10
    // Two terminal outputs (9 and 10)
    // Layer 1: 0 -> 1 (60%), 0 -> 2 (40%)
    // Layer 2: 1 -> 3 (70%), 1 -> 4 (30%), 2 -> 5 (25%), 2 -> 6 (75%)
    // Layer 3: 3 -> 7 (100%), 4 -> 7 (100%), 5 -> 8 (100%), 6 -> 8 (100%)
    // Layer 4: 7 -> 9 (100%), 8 -> 10 (100%)  <-- diverges
    let route_plan_diamond = vec![
        RoutePlanStep {
            swap: Swap::Meteora,
            percent: 60,
            input_index: 0,
            output_index: 1,
        },
        RoutePlanStep {
            swap: Swap::Raydium,
            percent: 40,
            input_index: 0,
            output_index: 2,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDlmm,
            percent: 70,
            input_index: 1,
            output_index: 3,
        },
        RoutePlanStep {
            swap: Swap::RaydiumCP,
            percent: 30,
            input_index: 1,
            output_index: 4,
        },
        RoutePlanStep {
            swap: Swap::RaydiumClmm,
            percent: 25,
            input_index: 2,
            output_index: 5,
        },
        RoutePlanStep {
            swap: Swap::Mercurial,
            percent: 75,
            input_index: 2,
            output_index: 6,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDammV2,
            percent: 100,
            input_index: 3,
            output_index: 7,
        },
        RoutePlanStep {
            swap: Swap::RaydiumV2,
            percent: 100,
            input_index: 4,
            output_index: 7,
        },
        RoutePlanStep {
            swap: Swap::RaydiumClmmV2,
            percent: 100,
            input_index: 5,
            output_index: 8,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDammV2WithRemainingAccounts,
            percent: 100,
            input_index: 6,
            output_index: 8,
        },
        RoutePlanStep {
            swap: Swap::Meteora,
            percent: 100,
            input_index: 7,
            output_index: 9, // two terminal outputs (9 and 10)
        },
        RoutePlanStep {
            swap: Swap::Raydium,
            percent: 100,
            input_index: 8,
            output_index: 10,
        },
    ];
    assert!(ensure_route_plan_fully_converges(&route_plan_diamond).is_err());
}
