use anchor_lang::prelude::*;

use crate::{error::ZapError, safe_math::SafeMath};

// Number of bits to scale. This will decide the position of the radix point.
const SCALE_OFFSET: u8 = 64;
const BASIS_POINT_MAX: i32 = 10000;
// 1.0000... representation of 64x64
pub const ONE: u128 = 1u128 << SCALE_OFFSET;
const MAX_EXPONENTIAL: u32 = 0x80000; // 1048576

// In Trader Joe, the active_id need to be shifted by 2 ** 23 to get the actual ID.
// The reason is because they mint LP for each bin based on active_id using ERC1155, which the ID do not support negative

/// Calculate price based on the given bin id. Eg: 1.0001 ^ 5555. The returned value is in Q64.64
pub fn get_price_from_id(active_id: i32, bin_step: u16) -> Result<u128> {
    let base = get_price_base_factor(bin_step)?;
    pow(base, active_id).ok_or_else(|| ZapError::MathOverflow.into())
}

// get price base factor
pub fn get_price_base_factor(bin_step: u16) -> Result<u128> {
    // Make bin_step into Q64x64, and divided by BASIS_POINT_MAX. If bin_step = 1, we get 0.0001 in Q64x64
    let bps = u128::from(bin_step)
        .safe_shl(SCALE_OFFSET.into())?
        .safe_div(BASIS_POINT_MAX as u128)?;
    // Add 1 to bps, we get 1.0001 in Q64.64
    let base = ONE.safe_add(bps)?;
    Ok(base)
}

pub fn pow(base: u128, exp: i32) -> Option<u128> {
    // https://doc.rust-lang.org/std/primitive.i32.html#method.abs
    // The absolute value of i32::MIN cannot be represented as an i32, and attempting to calculate it will cause an overflow.
    if exp == i32::MIN {
        return None;
    }

    // If exponent is negative. We will invert the result later by 1 / base^exp.abs()
    let mut invert = exp.is_negative();

    // When exponential is 0, result will always be 1
    if exp == 0 {
        return Some(1u128 << 64);
    }

    // Make the exponential positive. Which will compute the result later by 1 / base^exp
    let exp: u32 = if invert { exp.abs() as u32 } else { exp as u32 };

    // No point to continue the calculation as it will overflow the maximum value Q64.64 can support
    if exp >= MAX_EXPONENTIAL {
        return None;
    }

    let mut squared_base = base;
    let mut result = ONE;

    // When multiply the base twice, the number of bits double from 128 -> 256, which overflow.
    // The trick here is to inverse the calculation, which make the upper 64 bits (number bits) to be 0s.
    // For example:
    // let base = 1.001, exp = 5
    // let neg = 1 / (1.001 ^ 5)
    // Inverse the neg: 1 / neg
    // By using a calculator, you will find out that 1.001^5 == 1 / (1 / 1.001^5)
    if squared_base >= result {
        // This inverse the base: 1 / base
        squared_base = u128::MAX.checked_div(squared_base)?;
        // If exponent is negative, the above already inverted the result. Therefore, at the end of the function, we do not need to invert again.
        invert = !invert;
    }

    // The following code is equivalent to looping through each binary value of the exponential.
    // As explained in MAX_EXPONENTIAL, 19 exponential bits are enough to covert the full bin price.
    // Therefore, there will be 19 if statements, which similar to the following pseudo code.
    /*
        let mut result = 1;
        while exponential > 0 {
            if exponential & 1 > 0 {
                result *= base;
            }
            base *= base;
            exponential >>= 1;
        }
    */

    // From right to left
    // squared_base = 1 * base^1
    // 1st bit is 1
    if exp & 0x1 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    // squared_base = base^2
    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    // 2nd bit is 1
    if exp & 0x2 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    // Example:
    // If the base is 1.001, exponential is 3. Binary form of 3 is ..0011. The last 2 1's bit fulfill the above 2 bitwise condition.
    // The result will be 1 * base^1 * base^2 == base^3. The process continues until reach the 20th bit

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x4 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x8 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x10 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x20 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x40 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x80 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x100 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x200 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x400 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x800 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x1000 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x2000 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x4000 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x8000 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x10000 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x20000 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    squared_base = (squared_base.checked_mul(squared_base)?) >> SCALE_OFFSET;
    if exp & 0x40000 > 0 {
        result = (result.checked_mul(squared_base)?) >> SCALE_OFFSET
    }

    // Stop here as the next is 20th bit, which > MAX_EXPONENTIAL
    if result == 0 {
        return None;
    }

    if invert {
        result = u128::MAX.checked_div(result)?;
    }

    Some(result)
}
