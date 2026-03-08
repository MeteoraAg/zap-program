use jupiter::types::{RemainingAccountsInfo, RoutePlanStep, Swap};
use pinocchio::sysvars::{
    self,
    instructions::{Instructions, IntrospectedInstruction},
};
use solana_sdk::{
    pubkey::Pubkey,
    sysvar::instructions::{construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction},
};

use crate::{
    error::ProtocolZapError,
    get_jup_route_base_account_end_index,
    jup_swap_step_referral_fee_parser::{
        get_referral_fee_parser, mercurial,
        meteora_damm_v1::{self, Meteora},
        meteora_damm_v2,
        meteora_dlmm::{self, MeteoraDLMM, MeteoraDLMMSwapV2},
        raydium::{self, Raydium, RaydiumV2},
        raydium_clmm::{self, RaydiumClmm, RaydiumClmmV2},
        raydium_cp::{self, RaydiumCp},
        whirlpool::{self, Whirlpool, WhirlpoolSwapV2},
        SwapStepReferralFeeParser, PROGRAM_ACCOUNT_LENGTH,
    },
    WhitelistedSwapStep, ZapInfoProcessor, ZapJupV6RouteInfoProcessor,
};

// ---------------------------------------------------------------------------
// Types & constants
// ---------------------------------------------------------------------------

type RoutePlanStepWithReferralFee = (RoutePlanStep, bool);

#[derive(Clone, Copy, Debug, Default)]
struct MakeInstructionAccount {
    pubkey: Pubkey,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Clone, Debug)]
struct MakeInstructionParams {
    program_id: Pubkey,
    accounts: Vec<MakeInstructionAccount>,
    data: Vec<u8>,
}

const ZAP_OUT_JUP_V6_BASE_ACCOUNT_LENGTH: usize =
    2 + ZapJupV6RouteInfoProcessor::ROUTE_BASE_ACCOUNT_LENGTH;

// ---------------------------------------------------------------------------
// Low-level instruction helpers
// ---------------------------------------------------------------------------

fn create_borrowed_instruction<'a>(params: &'a MakeInstructionParams) -> BorrowedInstruction<'a> {
    let MakeInstructionParams {
        program_id,
        accounts,
        data,
    } = params;
    BorrowedInstruction {
        program_id,
        accounts: accounts
            .iter()
            .map(|account| BorrowedAccountMeta {
                pubkey: &account.pubkey,
                is_signer: account.is_signer,
                is_writable: account.is_writable,
            })
            .collect::<Vec<_>>(),
        data: data.as_ref(),
    }
}

fn create_pinocchio_sysvar_instruction(
    instruction_params: &[MakeInstructionParams],
) -> Instructions<Vec<u8>> {
    let borrowed_instructions = instruction_params
        .iter()
        .map(create_borrowed_instruction)
        .collect::<Vec<_>>();
    let data = construct_instructions_data(&borrowed_instructions);

    unsafe { Instructions::new_unchecked(data) }
}

// ---------------------------------------------------------------------------
// Account generators
// ---------------------------------------------------------------------------

fn generate_jupiter_route_accounts() -> [MakeInstructionAccount; ZAP_OUT_JUP_V6_BASE_ACCOUNT_LENGTH]
{
    [MakeInstructionAccount::default(); ZAP_OUT_JUP_V6_BASE_ACCOUNT_LENGTH]
}

fn generate_meteora_swap_accounts(has_referral_fee: bool) -> Vec<MakeInstructionAccount> {
    let meteora = Meteora;
    let capacity = PROGRAM_ACCOUNT_LENGTH
        + meteora.get_base_account_length()
        + Meteora::REMAINING_ACCOUNTS_LENGTH;
    let mut accounts = Vec::with_capacity(capacity);

    let meteora_program_account = MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(meteora_damm_v1::ID),
        is_signer: false,
        is_writable: false,
    };

    accounts.push(meteora_program_account);

    let base_account_end_index = capacity - Meteora::REMAINING_ACCOUNTS_LENGTH;

    for _ in accounts.len()..base_account_end_index {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    // Remaining account 0: Referral fee account (jupiter::ID as placeholder when no referral fee)
    let referral_fee_pubkey = if has_referral_fee {
        Pubkey::new_unique()
    } else {
        jupiter::ID
    };

    accounts.push(MakeInstructionAccount {
        pubkey: referral_fee_pubkey,
        is_signer: false,
        is_writable: false,
    });

    // Remaining account 1: Stake pool account
    accounts.push(MakeInstructionAccount {
        pubkey: jupiter::ID,
        is_signer: false,
        is_writable: false,
    });

    accounts
}

fn generate_mercurial_swap_accounts(
    mut token_address_length: usize,
) -> Vec<MakeInstructionAccount> {
    token_address_length = token_address_length.max(1); // Ensure at least 1 token address for swap step

    let capacity = PROGRAM_ACCOUNT_LENGTH + mercurial::BASE_ACCOUNT_LENGTH + token_address_length;
    let mut accounts = Vec::with_capacity(capacity);

    let mercurial_program_account = MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(mercurial::ID),
        is_signer: false,
        is_writable: false,
    };
    accounts.push(mercurial_program_account);

    for _ in accounts.len()..accounts.capacity() {
        let token_account = MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        };
        accounts.push(token_account);
    }

    accounts
}

fn generate_meteora_damm_v2_swap_accounts(has_referral_fee: bool) -> Vec<MakeInstructionAccount> {
    let base_account_length: usize = meteora_damm_v2::BASE_ACCOUNT_LENGTH;
    let referral_account_index: usize = meteora_damm_v2::REFERRAL_ACCOUNT_INDEX;

    let capacity = PROGRAM_ACCOUNT_LENGTH + base_account_length;
    let mut accounts = Vec::with_capacity(capacity);

    accounts.push(MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(meteora_damm_v2::ID),
        is_signer: false,
        is_writable: false,
    });

    for i in 0..base_account_length {
        let pubkey = if i == referral_account_index {
            if has_referral_fee {
                Pubkey::new_unique()
            } else {
                Pubkey::new_from_array(meteora_damm_v2::ID)
            }
        } else {
            Pubkey::new_unique()
        };
        accounts.push(MakeInstructionAccount {
            pubkey,
            is_signer: false,
            is_writable: false,
        });
    }

    accounts
}

fn generate_meteora_damm_v2_with_remaining_swap_accounts(
    has_referral_fee: bool,
) -> Vec<MakeInstructionAccount> {
    let mut accounts = generate_meteora_damm_v2_swap_accounts(has_referral_fee);

    accounts.push(MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(sysvars::instructions::INSTRUCTIONS_ID),
        is_signer: false,
        is_writable: false,
    });

    // Placeholder terminator
    accounts.push(MakeInstructionAccount {
        pubkey: jupiter::ID,
        is_signer: false,
        is_writable: false,
    });

    accounts
}

fn generate_meteora_dlmm_swap_accounts(
    has_referral_fee: bool,
    bin_array_length: usize,
) -> Vec<MakeInstructionAccount> {
    let base_account_length: usize = MeteoraDLMM::BASE_ACCOUNT_LENGTH;
    let referral_account_index: usize = meteora_dlmm::REFERRAL_ACCOUNT_INDEX;
    let capacity = PROGRAM_ACCOUNT_LENGTH + base_account_length;
    let mut accounts = Vec::with_capacity(capacity);

    accounts.push(MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(meteora_dlmm::ID),
        is_signer: false,
        is_writable: false,
    });

    for i in 0..base_account_length {
        let pubkey = if i == referral_account_index {
            if has_referral_fee {
                Pubkey::new_unique()
            } else {
                Pubkey::new_from_array(meteora_dlmm::ID)
            }
        } else {
            Pubkey::new_unique()
        };
        accounts.push(MakeInstructionAccount {
            pubkey,
            is_signer: false,
            is_writable: false,
        });
    }

    // Remaining bin array accounts before placeholder
    for _ in 0..bin_array_length {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    // Placeholder terminator
    accounts.push(MakeInstructionAccount {
        pubkey: jupiter::ID,
        is_signer: false,
        is_writable: false,
    });

    accounts
}

fn generate_meteora_dlmm_v2_swap_accounts(
    has_referral_fee: bool,
    bin_array_length: usize,
) -> Vec<MakeInstructionAccount> {
    let base_account_length: usize = MeteoraDLMMSwapV2::BASE_ACCOUNT_LENGTH;
    let referral_account_index: usize = meteora_dlmm::REFERRAL_ACCOUNT_INDEX;
    let capacity = PROGRAM_ACCOUNT_LENGTH + base_account_length;
    let mut accounts = Vec::with_capacity(capacity);

    accounts.push(MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(meteora_dlmm::ID),
        is_signer: false,
        is_writable: false,
    });

    for i in 0..base_account_length {
        let pubkey = if i == referral_account_index {
            if has_referral_fee {
                Pubkey::new_unique()
            } else {
                Pubkey::new_from_array(meteora_dlmm::ID)
            }
        } else {
            Pubkey::new_unique()
        };
        accounts.push(MakeInstructionAccount {
            pubkey,
            is_signer: false,
            is_writable: false,
        });
    }

    // Remaining bin array accounts before placeholder
    for _ in 0..bin_array_length {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    // Placeholder terminator
    accounts.push(MakeInstructionAccount {
        pubkey: jupiter::ID,
        is_signer: false,
        is_writable: false,
    });

    accounts
}

fn generate_whirlpool_swap_accounts() -> Vec<MakeInstructionAccount> {
    let base_account_length: usize = Whirlpool::BASE_ACCOUNT_LENGTH;
    let capacity = PROGRAM_ACCOUNT_LENGTH + base_account_length;
    let mut accounts = Vec::with_capacity(capacity);

    accounts.push(MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(whirlpool::ID),
        is_signer: false,
        is_writable: false,
    });

    for _ in 0..base_account_length {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    accounts
}

fn generate_whirlpool_swap_v2_accounts(
    remaining_account_info: Option<&RemainingAccountsInfo>,
) -> Vec<MakeInstructionAccount> {
    let base_account_length: usize = WhirlpoolSwapV2::BASE_ACCOUNT_LENGTH;
    let remaining_accounts_length = remaining_account_info
        .map(|info| info.slices.iter().map(|s| s.length as usize).sum::<usize>())
        .unwrap_or(0);
    let capacity = PROGRAM_ACCOUNT_LENGTH + base_account_length + remaining_accounts_length;
    let mut accounts = Vec::with_capacity(capacity);

    accounts.push(MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(whirlpool::ID),
        is_signer: false,
        is_writable: false,
    });

    for _ in 0..base_account_length + remaining_accounts_length {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    accounts
}

fn generate_raydium_swap_accounts() -> Vec<MakeInstructionAccount> {
    let base_account_length: usize = Raydium::BASE_ACCOUNT_LENGTH;
    let capacity = PROGRAM_ACCOUNT_LENGTH + base_account_length;
    let mut accounts = Vec::with_capacity(capacity);

    accounts.push(MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(raydium::ID),
        is_signer: false,
        is_writable: false,
    });

    for _ in 0..base_account_length {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    accounts
}

fn generate_raydium_v2_swap_accounts() -> Vec<MakeInstructionAccount> {
    let base_account_length: usize = RaydiumV2::BASE_ACCOUNT_LENGTH;
    let capacity = PROGRAM_ACCOUNT_LENGTH + base_account_length;
    let mut accounts = Vec::with_capacity(capacity);

    accounts.push(MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(raydium::ID),
        is_signer: false,
        is_writable: false,
    });

    for _ in 0..base_account_length {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    accounts
}

fn generate_raydium_cp_swap_accounts() -> Vec<MakeInstructionAccount> {
    let base_account_length: usize = RaydiumCp::BASE_ACCOUNT_LENGTH;
    let capacity = PROGRAM_ACCOUNT_LENGTH + base_account_length;
    let mut accounts = Vec::with_capacity(capacity);

    accounts.push(MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(raydium_cp::ID),
        is_signer: false,
        is_writable: false,
    });

    for _ in 0..base_account_length {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    accounts
}

fn generate_raydium_clmm_swap_accounts() -> Vec<MakeInstructionAccount> {
    let base_account_length: usize = RaydiumClmm::BASE_ACCOUNT_LENGTH;
    let capacity = PROGRAM_ACCOUNT_LENGTH + base_account_length;
    let mut accounts = Vec::with_capacity(capacity);

    accounts.push(MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(raydium_clmm::ID),
        is_signer: false,
        is_writable: false,
    });

    for _ in 0..base_account_length {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    // Remaining tick array accounts before placeholder
    for _ in 0..3 {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    // Placeholder terminator
    accounts.push(MakeInstructionAccount {
        pubkey: jupiter::ID,
        is_signer: false,
        is_writable: false,
    });

    accounts
}

fn generate_raydium_clmm_v2_swap_accounts() -> Vec<MakeInstructionAccount> {
    let base_account_length: usize = RaydiumClmmV2::BASE_ACCOUNT_LENGTH;
    let capacity = PROGRAM_ACCOUNT_LENGTH + base_account_length;
    let mut accounts = Vec::with_capacity(capacity);

    accounts.push(MakeInstructionAccount {
        pubkey: Pubkey::new_from_array(raydium_clmm::ID),
        is_signer: false,
        is_writable: false,
    });

    for _ in 0..base_account_length {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    // Remaining tick bitmap + tick array accounts before placeholder
    for _ in 0..4 {
        accounts.push(MakeInstructionAccount {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        });
    }

    // Placeholder terminator
    accounts.push(MakeInstructionAccount {
        pubkey: jupiter::ID,
        is_signer: false,
        is_writable: false,
    });

    accounts
}

// ---------------------------------------------------------------------------
// Test context & setup
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct TestContextDataContainer {
    instruction_params: [MakeInstructionParams; 1],
}

#[derive(Debug)]
struct TestContext {
    data: TestContextDataContainer,
    p_sysvar_instruction: Instructions<Vec<u8>>,
}

impl TestContext {
    fn get_prerequisite_for_testing(
        &self,
    ) -> (Box<dyn ZapInfoProcessor>, IntrospectedInstruction<'_>) {
        let zap_out_instruction = self
            .p_sysvar_instruction
            .load_instruction_at(self.p_sysvar_instruction.load_current_index().into())
            .unwrap();

        let instruction_param = &self.data.instruction_params[0];
        let payload_data = &instruction_param.data[8..]; // Skip discriminator

        let processor = ZapJupV6RouteInfoProcessor::new(payload_data).unwrap();
        (Box::new(processor), zap_out_instruction)
    }
}

fn build_route_plan_steps_and_accounts(
    route_plan_steps: &[RoutePlanStepWithReferralFee],
) -> (Vec<RoutePlanStep>, Vec<MakeInstructionAccount>) {
    let mut raw_route_plan_steps = Vec::with_capacity(route_plan_steps.len());
    let mut route_plan_steps_accounts = Vec::with_capacity(route_plan_steps.len());

    for (step, has_referral_fee) in route_plan_steps {
        let Ok(whitelisted_swap_step) = WhitelistedSwapStep::try_from(&step.swap) else {
            panic!("Unsupported swap type in test: {:?}", step.swap);
        };

        raw_route_plan_steps.push(step.clone());

        let accounts = match whitelisted_swap_step {
            WhitelistedSwapStep::Meteora => generate_meteora_swap_accounts(*has_referral_fee),
            WhitelistedSwapStep::MeteoraDammV2 => {
                generate_meteora_damm_v2_swap_accounts(*has_referral_fee)
            }
            WhitelistedSwapStep::MeteoraDammV2WithRemainingAccounts => {
                generate_meteora_damm_v2_with_remaining_swap_accounts(*has_referral_fee)
            }
            WhitelistedSwapStep::MeteoraDlmm => {
                generate_meteora_dlmm_swap_accounts(*has_referral_fee, 3)
            }
            WhitelistedSwapStep::MeteoraDlmmSwapV2 => {
                generate_meteora_dlmm_v2_swap_accounts(*has_referral_fee, 3)
            }
            WhitelistedSwapStep::Mercurial => generate_mercurial_swap_accounts(3),
            WhitelistedSwapStep::Whirlpool => generate_whirlpool_swap_accounts(),
            WhitelistedSwapStep::WhirlpoolSwapV2 {
                remaining_accounts_info,
            } => generate_whirlpool_swap_v2_accounts(remaining_accounts_info.as_ref()),
            WhitelistedSwapStep::Raydium => generate_raydium_swap_accounts(),
            WhitelistedSwapStep::RaydiumV2 => generate_raydium_v2_swap_accounts(),
            WhitelistedSwapStep::RaydiumCP => generate_raydium_cp_swap_accounts(),
            WhitelistedSwapStep::RaydiumClmm => generate_raydium_clmm_swap_accounts(),
            WhitelistedSwapStep::RaydiumClmmV2 => generate_raydium_clmm_v2_swap_accounts(),
        };
        route_plan_steps_accounts.extend(accounts);
    }

    (raw_route_plan_steps, route_plan_steps_accounts)
}

fn setup_jupiter_v6_route_test_context(
    route_plan_steps: &[RoutePlanStepWithReferralFee],
    in_amount: u64,
    quoted_out_amount: u64,
    slippage_bps: u16,
    platform_fee_bps: u8,
) -> TestContext {
    let jup_v6_base_accounts = generate_jupiter_route_accounts();

    let (route_plan, route_plan_steps_accounts) =
        build_route_plan_steps_and_accounts(route_plan_steps);

    let route_accounts = [jup_v6_base_accounts.to_vec(), route_plan_steps_accounts].concat();

    let data = jupiter::client::args::Route {
        route_plan,
        in_amount,
        quoted_out_amount,
        slippage_bps,
        platform_fee_bps,
    };

    let payload = borsh::to_vec(&data).unwrap();
    let payload_with_discriminator =
        [zap_sdk::constants::JUP_V6_ROUTE_DISC.to_vec(), payload].concat();

    let instruction_param = MakeInstructionParams {
        program_id: zap_sdk::constants::JUP_V6,
        accounts: route_accounts,
        data: payload_with_discriminator,
    };

    let instruction_params = [instruction_param];

    let data_container = TestContextDataContainer { instruction_params };

    let p_sysvar_instruction =
        create_pinocchio_sysvar_instruction(&data_container.instruction_params);

    TestContext {
        data: data_container,
        p_sysvar_instruction,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[test]
fn test_multi_hop_and_split_step_without_referral_fee() {
    let route_step_plan = vec![
        (
            RoutePlanStep {
                swap: Swap::MeteoraDlmm,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::RaydiumClmmV2,
                percent: 50,
                input_index: 1,
                output_index: 2,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::WhirlpoolSwapV2 {
                    a_to_b: false,
                    remaining_accounts_info: None,
                },
                percent: 50,
                input_index: 1,
                output_index: 2,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::MeteoraDammV2WithRemainingAccounts,
                percent: 100,
                input_index: 2,
                output_index: 3,
            },
            false,
        ),
    ];

    let test_context =
        setup_jupiter_v6_route_test_context(&route_step_plan, 100_000, 100_000, 0, 0);

    let (processor, zap_out_instruction) = test_context.get_prerequisite_for_testing();
    let result = processor.validate_route_plan(&zap_out_instruction);
    assert!(result.is_ok());
}

#[test]
fn test_multi_hop_and_split_step_with_referral_fee() {
    let route_step_plan = vec![
        (
            RoutePlanStep {
                swap: Swap::MeteoraDlmm,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            true,
        ),
        (
            RoutePlanStep {
                swap: Swap::RaydiumClmmV2,
                percent: 50,
                input_index: 1,
                output_index: 2,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::WhirlpoolSwapV2 {
                    a_to_b: false,
                    remaining_accounts_info: None,
                },
                percent: 50,
                input_index: 1,
                output_index: 2,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::MeteoraDammV2WithRemainingAccounts,
                percent: 100,
                input_index: 2,
                output_index: 3,
            },
            true,
        ),
    ];

    let test_context =
        setup_jupiter_v6_route_test_context(&route_step_plan, 100_000, 100_000, 0, 0);

    let (processor, zap_out_instruction) = test_context.get_prerequisite_for_testing();
    let err = processor
        .validate_route_plan(&zap_out_instruction)
        .unwrap_err();
    assert_eq!(err, ProtocolZapError::ReferralFeeNotAllowed);
}

#[test]
fn test_split_step_with_referral_fee() {
    let route_step_plan = vec![
        (
            RoutePlanStep {
                swap: Swap::MeteoraDlmm,
                percent: 30,
                input_index: 0,
                output_index: 1,
            },
            true,
        ),
        (
            RoutePlanStep {
                swap: Swap::RaydiumClmmV2,
                percent: 30,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::WhirlpoolSwapV2 {
                    a_to_b: false,
                    remaining_accounts_info: None,
                },
                percent: 40,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
    ];

    let test_context =
        setup_jupiter_v6_route_test_context(&route_step_plan, 100_000, 100_000, 0, 0);

    let (processor, zap_out_instruction) = test_context.get_prerequisite_for_testing();
    let err = processor
        .validate_route_plan(&zap_out_instruction)
        .unwrap_err();
    assert_eq!(err, ProtocolZapError::ReferralFeeNotAllowed);
}

#[test]
fn test_split_step_without_referral_fee() {
    let route_step_plan = vec![
        (
            RoutePlanStep {
                swap: Swap::MeteoraDlmm,
                percent: 30,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::MeteoraDammV2,
                percent: 30,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::Mercurial,
                percent: 40,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
    ];

    let test_context =
        setup_jupiter_v6_route_test_context(&route_step_plan, 100_000, 100_000, 0, 0);

    let (processor, zap_out_instruction) = test_context.get_prerequisite_for_testing();
    let result = processor.validate_route_plan(&zap_out_instruction);
    assert!(result.is_ok());
}

#[test]
fn test_multi_hop_step_with_referral_fee() {
    let route_step_plan = vec![
        (
            RoutePlanStep {
                swap: Swap::Mercurial,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::MeteoraDammV2WithRemainingAccounts,
                percent: 100,
                input_index: 1,
                output_index: 2,
            },
            true, // With referral fee
        ),
        (
            RoutePlanStep {
                swap: Swap::RaydiumClmmV2,
                percent: 100,
                input_index: 2,
                output_index: 3,
            },
            false,
        ),
    ];

    let test_context =
        setup_jupiter_v6_route_test_context(&route_step_plan, 100_000, 100_000, 0, 0);

    let (processor, zap_out_instruction) = test_context.get_prerequisite_for_testing();
    let err = processor
        .validate_route_plan(&zap_out_instruction)
        .unwrap_err();
    assert_eq!(err, ProtocolZapError::ReferralFeeNotAllowed);
}

#[test]
fn test_multi_hop_step_without_referral_fee() {
    let route_step_plan = vec![
        (
            RoutePlanStep {
                swap: Swap::MeteoraDammV2WithRemainingAccounts,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false, // No referral fee
        ),
        (
            RoutePlanStep {
                swap: Swap::Mercurial,
                percent: 100,
                input_index: 1,
                output_index: 2,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::RaydiumClmmV2,
                percent: 100,
                input_index: 2,
                output_index: 3,
            },
            false,
        ),
    ];

    let test_context =
        setup_jupiter_v6_route_test_context(&route_step_plan, 100_000, 100_000, 0, 0);

    let (processor, zap_out_instruction) = test_context.get_prerequisite_for_testing();
    let result = processor.validate_route_plan(&zap_out_instruction);
    assert!(result.is_ok());
}

#[test]
fn test_get_end_account_index_for_mercurial() {
    let route_step_plan = vec![
        (
            RoutePlanStep {
                swap: Swap::Mercurial,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::Meteora,
                percent: 100,
                input_index: 1,
                output_index: 2,
            },
            false,
        ),
    ];

    let test_context =
        setup_jupiter_v6_route_test_context(&route_step_plan, 100_000, 100_000, 0, 0);

    let (_processor, zap_out_instruction) = test_context.get_prerequisite_for_testing();

    let whitelisted_swap_step = WhitelistedSwapStep::try_from(&route_step_plan[0].0.swap).unwrap();
    let mut parser = get_referral_fee_parser(&whitelisted_swap_step);
    parser
        .load_next_swap_step(Some(&route_step_plan[1].0))
        .unwrap();

    let processed_index =
        get_jup_route_base_account_end_index(ZapJupV6RouteInfoProcessor::ROUTE_BASE_ACCOUNT_LENGTH)
            .unwrap();

    let end_account_index = parser
        .get_end_account_index(processed_index, &zap_out_instruction)
        .unwrap();

    let account = zap_out_instruction
        .get_account_meta_at(end_account_index)
        .unwrap();

    let account_key = Pubkey::new_from_array(account.key);

    let full_accounts_ref = &test_context.data.instruction_params[0].accounts;

    let end_account_key = full_accounts_ref
        .get(end_account_index)
        .map(|account| account.pubkey)
        .unwrap();

    assert_eq!(account_key, end_account_key);

    let next_account_key = full_accounts_ref
        .get(end_account_index + 1)
        .map(|account| account.pubkey)
        .unwrap();

    assert_eq!(
        next_account_key,
        Pubkey::new_from_array(meteora_damm_v1::ID)
    );
}

#[test]
fn test_get_end_account_index_for_all_swap_steps() {
    let swap_legs: Vec<RoutePlanStepWithReferralFee> = vec![
        (
            RoutePlanStep {
                swap: Swap::MeteoraDammV2,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::MeteoraDammV2WithRemainingAccounts,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::MeteoraDlmm,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::MeteoraDlmmSwapV2 {
                    remaining_accounts_info: RemainingAccountsInfo::default(),
                },
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::Whirlpool { a_to_b: false },
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::WhirlpoolSwapV2 {
                    a_to_b: false,
                    remaining_accounts_info: None,
                },
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::Raydium,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::RaydiumV2,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::RaydiumCP,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::RaydiumClmm,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::RaydiumClmmV2,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
        (
            RoutePlanStep {
                swap: Swap::Meteora,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        ),
    ];

    let processed_index =
        get_jup_route_base_account_end_index(ZapJupV6RouteInfoProcessor::ROUTE_BASE_ACCOUNT_LENGTH)
            .unwrap();

    for (step, has_referral_fee) in &swap_legs {
        let route_step_plan = vec![(step.clone(), *has_referral_fee)];

        let test_context =
            setup_jupiter_v6_route_test_context(&route_step_plan, 100_000, 100_000, 0, 0);

        let (_processor, zap_out_instruction) = test_context.get_prerequisite_for_testing();

        let whitelisted_swap_step =
            WhitelistedSwapStep::try_from(&route_step_plan[0].0.swap).unwrap();
        let parser = get_referral_fee_parser(&whitelisted_swap_step);

        let end_account_index = parser
            .get_end_account_index(processed_index, &zap_out_instruction)
            .unwrap();

        let account = zap_out_instruction
            .get_account_meta_at(end_account_index)
            .unwrap();

        let account_key = Pubkey::new_from_array(account.key);

        let last_account_key_in_test_context = test_context.data.instruction_params[0]
            .accounts
            .last()
            .map(|account| account.pubkey)
            .unwrap();

        assert_eq!(
            account_key, last_account_key_in_test_context,
            "End account index mismatch for swap type: {:?}",
            step.swap
        );
    }
}

#[test]
fn test_ensure_no_referral_fee_for_all_swap_steps_with_referral_fee() {
    let swap_types: Vec<Swap> = vec![
        Swap::Meteora,
        Swap::MeteoraDammV2,
        Swap::MeteoraDammV2WithRemainingAccounts,
        Swap::MeteoraDlmm,
        Swap::MeteoraDlmmSwapV2 {
            remaining_accounts_info: RemainingAccountsInfo::default(),
        },
    ];

    for swap in &swap_types {
        // Without referral fee — should pass
        let route_step_plan = vec![(
            RoutePlanStep {
                swap: swap.clone(),
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            false,
        )];

        let test_context =
            setup_jupiter_v6_route_test_context(&route_step_plan, 100_000, 100_000, 0, 0);
        let (processor, zap_out_instruction) = test_context.get_prerequisite_for_testing();
        let result = processor.validate_route_plan(&zap_out_instruction);
        assert!(
            result.is_ok(),
            "Expected Ok for swap type without referral fee: {:?}",
            swap
        );

        // With referral fee — should fail
        let route_step_plan = vec![(
            RoutePlanStep {
                swap: swap.clone(),
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            true,
        )];

        let test_context =
            setup_jupiter_v6_route_test_context(&route_step_plan, 100_000, 100_000, 0, 0);
        let (processor, zap_out_instruction) = test_context.get_prerequisite_for_testing();
        let err = processor
            .validate_route_plan(&zap_out_instruction)
            .unwrap_err();
        assert_eq!(
            err,
            ProtocolZapError::ReferralFeeNotAllowed,
            "Expected ReferralFeeNotAllowed for swap type with referral fee: {:?}",
            swap
        );
    }
}
