use dlmm::types::{RemainingAccountsInfo, RemainingAccountsSlice};

use crate::{encode_account_type, SwapDlmmParams};

#[test]
fn dlmm_parameters_pack_unpack_empty_slices() {
    let swap_dlmm_parameters = SwapDlmmParams {
        minimum_amount_out: 1000u64,
        remaining_accounts_info: RemainingAccountsInfo { slices: vec![] },
    };

    let data = swap_dlmm_parameters.pack();
    let unpack_data = SwapDlmmParams::unpack(&data).unwrap();
    assert_eq!(
        unpack_data.minimum_amount_out,
        swap_dlmm_parameters.minimum_amount_out
    );

    assert_eq!(
        unpack_data.remaining_accounts_info.slices.len(),
        swap_dlmm_parameters.remaining_accounts_info.slices.len()
    )
}

#[test]
fn dlmm_parameters_pack_unpack_one_slice() {
    let slices = vec![RemainingAccountsSlice {
        accounts_type: dlmm::types::AccountsType::TransferHookX,
        length: 3,
    }];

    let swap_dlmm_parameters = SwapDlmmParams {
        minimum_amount_out: 1000u64,
        remaining_accounts_info: RemainingAccountsInfo { slices },
    };

    let data = swap_dlmm_parameters.pack();
    let unpack_data = SwapDlmmParams::unpack(&data).unwrap();
    assert_eq!(
        unpack_data.minimum_amount_out,
        swap_dlmm_parameters.minimum_amount_out
    );

    assert_eq!(
        unpack_data.remaining_accounts_info.slices.len(),
        swap_dlmm_parameters.remaining_accounts_info.slices.len()
    );

    let unpack_data_account_type =
        encode_account_type(&unpack_data.remaining_accounts_info.slices[0].accounts_type);

    let param_account_type =
        encode_account_type(&swap_dlmm_parameters.remaining_accounts_info.slices[0].accounts_type);
    assert_eq!(unpack_data_account_type, param_account_type);
    assert_eq!(
        unpack_data.remaining_accounts_info.slices[0].length,
        swap_dlmm_parameters.remaining_accounts_info.slices[0].length
    )
}

#[test]
fn dlmm_parameters_pack_unpack_two_slices() {
    let slices = vec![
        RemainingAccountsSlice {
            accounts_type: dlmm::types::AccountsType::TransferHookX,
            length: 3,
        },
        RemainingAccountsSlice {
            accounts_type: dlmm::types::AccountsType::TransferHookY,
            length: 3,
        },
    ];

    let swap_dlmm_parameters = SwapDlmmParams {
        minimum_amount_out: 1000u64,
        remaining_accounts_info: RemainingAccountsInfo { slices },
    };

    let data = swap_dlmm_parameters.pack();
    let unpack_data = SwapDlmmParams::unpack(&data).unwrap();

    assert_eq!(
        unpack_data.minimum_amount_out,
        swap_dlmm_parameters.minimum_amount_out
    );

    assert_eq!(
        unpack_data.remaining_accounts_info.slices.len(),
        swap_dlmm_parameters.remaining_accounts_info.slices.len()
    );

    let unpack_data_account_type_0 =
        encode_account_type(&unpack_data.remaining_accounts_info.slices[0].accounts_type);

    let param_account_type_0 =
        encode_account_type(&swap_dlmm_parameters.remaining_accounts_info.slices[0].accounts_type);
    assert_eq!(unpack_data_account_type_0, param_account_type_0);
    assert_eq!(
        unpack_data.remaining_accounts_info.slices[0].length,
        swap_dlmm_parameters.remaining_accounts_info.slices[0].length
    );

    let unpack_data_account_type_1 =
        encode_account_type(&unpack_data.remaining_accounts_info.slices[1].accounts_type);

    let param_account_type_1 =
        encode_account_type(&swap_dlmm_parameters.remaining_accounts_info.slices[1].accounts_type);
    assert_eq!(unpack_data_account_type_1, param_account_type_1);
    assert_eq!(
        unpack_data.remaining_accounts_info.slices[1].length,
        swap_dlmm_parameters.remaining_accounts_info.slices[1].length
    )
}
