use anchor_lang::prelude::*;
struct MockZapOutCtx;

impl MockZapOutCtx {
    fn modify_instruction_data(
        &self,
        payload_data: &mut Vec<u8>,
        amount_in: u64,
        offset_amount_in: usize,
    ) -> Result<()> {
        let amount_in_bytes = amount_in.to_le_bytes();
        let end_offset_index = offset_amount_in + amount_in_bytes.len();

        payload_data.splice(
            offset_amount_in..end_offset_index,
            amount_in_bytes.iter().cloned(),
        );

        Ok(())
    }
}

#[test]
fn test_modify_instruction_data() {
    let ctx = MockZapOutCtx;
    let disc = [229, 23, 203, 151, 122, 227, 173, 42];
    let amount = 10000000u64;

    let mut payload = vec![];
    payload.extend_from_slice(&disc);
    payload.extend_from_slice(&amount.to_le_bytes());
    let new_amount = 999999999999u64; // This will be [255, 15, 165, 212, 232, 0, 0, 0] in le bytes
    let offset = 8;
    println!("payload {:?}", payload);
    let result = ctx.modify_instruction_data(&mut payload, new_amount, offset);

    assert!(result.is_ok());
    assert_eq!(payload.len(), 16); // Length should remain the same

    // Check that bytes 8-15 are replaced with amount bytes
    let mut expected: Vec<u8> = vec![];

    expected.extend_from_slice(&disc);
    expected.extend_from_slice(&new_amount.to_le_bytes());

    println!("payload {:?}", payload);

    println!("expected {:?}", expected);
    assert_eq!(payload, expected);
}
