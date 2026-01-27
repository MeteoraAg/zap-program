use jupiter::types::{RoutePlanStep, Swap};

use crate::jup_v6_zap::ensure_route_plan_fully_converges;

#[test]
fn test_route_plan_converges_with_100_percent() {
    let route_plan_1_market = vec![RoutePlanStep {
        swap: Swap::Meteora,
        percent: 100,
        input_index: 0,
        output_index: 1,
    }];
    assert!(ensure_route_plan_fully_converges(&route_plan_1_market).is_ok());
    let route_plan_multi_market: Vec<RoutePlanStep> = vec![
        RoutePlanStep {
            swap: Swap::Meteora,
            percent: 50,
            input_index: 0,
            output_index: 1,
        },
        RoutePlanStep {
            swap: Swap::Raydium,
            percent: 50,
            input_index: 0,
            output_index: 1,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDlmm,
            percent: 100,
            input_index: 1,
            output_index: 2,
        },
    ];
    assert!(ensure_route_plan_fully_converges(&route_plan_multi_market).is_ok());
}

#[test]
fn test_route_plan_fails_with_partial_percent() {
    let route_plan_1_market = vec![RoutePlanStep {
        swap: Swap::Meteora,
        percent: 50,
        input_index: 0,
        output_index: 1,
    }];
    assert!(ensure_route_plan_fully_converges(&route_plan_1_market).is_err());
    let route_plan_1_market = vec![
        RoutePlanStep {
            swap: Swap::Meteora,
            percent: 100,
            input_index: 0,
            output_index: 1,
        },
        RoutePlanStep {
            swap: Swap::Raydium,
            percent: 50,
            input_index: 1,
            output_index: 2,
        },
        RoutePlanStep {
            swap: Swap::MeteoraDlmm,
            percent: 50,
            input_index: 1,
            output_index: 3,
        },
    ];
    assert!(ensure_route_plan_fully_converges(&route_plan_1_market).is_err());
}
