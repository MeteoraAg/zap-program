use crate::SwapDammV2Params;

#[test]
fn damm_v2_parameters_pack_unpack() {
    let swap_damm_v2_parameters = SwapDammV2Params {
        minimum_amount_out: 1000u64,
    };

    let data = swap_damm_v2_parameters.pack();
    let unpack_data = SwapDammV2Params::unpack(&data).unwrap();
    assert_eq!(
        unpack_data.minimum_amount_out,
        swap_damm_v2_parameters.minimum_amount_out
    )
}
