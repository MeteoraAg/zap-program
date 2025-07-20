use anchor_lang::prelude::*;

pub trait ZapOutParametersDecoder: Sized {
    fn decode(payload_data: Vec<u8>) -> Result<Self>;
}
