use crate::{
    error::ProtocolZapError,
    jup_swap_step_referral_fee_parser::{
        find_next_swap_step_program_account_index, SwapStepReferralFeeParser,
    },
    safe_math::SafeMath,
    WhitelistedSwapStep,
};
use jupiter::types::RoutePlanStep;
use pinocchio::{pubkey::Pubkey, sysvars::instructions::IntrospectedInstruction};
use pinocchio_pubkey::from_str;

pub const ID: Pubkey = from_str("MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky");
pub const BASE_ACCOUNT_LENGTH: usize = 6;

// Mercurial
#[derive(Default)]
pub struct Mercurial {
    next_swap_step: Option<WhitelistedSwapStep>,
}

impl SwapStepReferralFeeParser for Mercurial {
    fn get_base_account_length(&self) -> usize {
        BASE_ACCOUNT_LENGTH
    }

    fn load_next_swap_step(
        &mut self,
        next_swap_step: Option<&RoutePlanStep>,
    ) -> Result<(), ProtocolZapError> {
        self.next_swap_step = next_swap_step
            .map(|step| WhitelistedSwapStep::try_from(&step.swap))
            .transpose()?;

        Ok(())
    }

    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        let end_base = self.get_end_account_index_default(processed_index)?;

        // In order to know the length of token_account_addresses, we need to deserialize the Swap account which is impossible with introspected instruction
        // We will look for next swap step's program account index -1 as the end account index of current swap step since program account is always the first account of a swap step
        let Some(next_swap_step) = self.next_swap_step.as_ref() else {
            return Ok(end_base);
        };

        let next_swap_step_program_index = find_next_swap_step_program_account_index(
            zap_out_instruction,
            processed_index,
            next_swap_step,
        )?;

        next_swap_step_program_index.safe_sub(1)
    }
}
