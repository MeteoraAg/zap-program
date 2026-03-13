use anchor_lang::prelude::Result;
use damm_v2::u128x128_math::Rounding;
use ruint::aliases::U256;

use crate::{error::ZapError, safe_math::SafeMath};

#[inline]
pub fn safe_mul_div_cast_u64(x: u64, y: u64, denominator: u64, rounding: Rounding) -> Result<u64> {
    let prod = u128::from(x).safe_mul(y.into())?;
    let denominator: u128 = denominator.into();

    let result = match rounding {
        Rounding::Up => prod
            .safe_add(denominator)?
            .safe_sub(1u128)?
            .safe_div(denominator)?,
        Rounding::Down => prod.safe_div(denominator)?,
    };

    result
        .try_into()
        .map_err(|_| ZapError::TypeCastFailed.into())
}

#[inline]
pub fn safe_mul_div_cast_u128(
    x: u128,
    y: u128,
    denominator: u128,
    rounding: Rounding,
) -> Result<u128> {
    let prod = U256::from(x).safe_mul(U256::from(y))?;
    let denominator = U256::from(denominator);

    let result = match rounding {
        Rounding::Up => prod
            .safe_add(denominator)?
            .safe_sub(U256::from(1))?
            .safe_div(denominator)?,
        Rounding::Down => prod.safe_div(denominator)?,
    };

    result
        .try_into()
        .map_err(|_| ZapError::TypeCastFailed.into())
}
