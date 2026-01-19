use crate::error::ProtocolZapError;
use anchor_lang::solana_program::msg;
use ruint::aliases::U256;
use std::panic::Location;

/// safe math module
pub trait SafeMath<T>: Sized {
    /// safe add
    fn safe_add(self, rhs: Self) -> Result<Self, ProtocolZapError>;
    /// safe mul
    fn safe_mul(self, rhs: Self) -> Result<Self, ProtocolZapError>;
    /// safe div
    fn safe_div(self, rhs: Self) -> Result<Self, ProtocolZapError>;
    /// safe rem
    fn safe_rem(self, rhs: Self) -> Result<Self, ProtocolZapError>;
    /// safe sub
    fn safe_sub(self, rhs: Self) -> Result<Self, ProtocolZapError>;
    /// safe shl
    fn safe_shl(self, offset: T) -> Result<Self, ProtocolZapError>;
    /// safe shr
    fn safe_shr(self, offset: T) -> Result<Self, ProtocolZapError>;
}

macro_rules! checked_impl {
    ($t:ty, $offset:ty) => {
        impl SafeMath<$offset> for $t {
            #[inline(always)]
            fn safe_add(self, v: $t) -> Result<$t, ProtocolZapError> {
                match self.checked_add(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(ProtocolZapError::MathOverflow)
                    }
                }
            }

            #[inline(always)]
            fn safe_sub(self, v: $t) -> Result<$t, ProtocolZapError> {
                match self.checked_sub(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(ProtocolZapError::MathOverflow)
                    }
                }
            }

            #[inline(always)]
            fn safe_mul(self, v: $t) -> Result<$t, ProtocolZapError> {
                match self.checked_mul(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(ProtocolZapError::MathOverflow)
                    }
                }
            }

            #[inline(always)]
            fn safe_div(self, v: $t) -> Result<$t, ProtocolZapError> {
                match self.checked_div(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(ProtocolZapError::MathOverflow)
                    }
                }
            }

            #[inline(always)]
            fn safe_rem(self, v: $t) -> Result<$t, ProtocolZapError> {
                match self.checked_rem(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(ProtocolZapError::MathOverflow)
                    }
                }
            }

            #[inline(always)]
            fn safe_shl(self, v: $offset) -> Result<$t, ProtocolZapError> {
                match self.checked_shl(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(ProtocolZapError::MathOverflow)
                    }
                }
            }

            #[inline(always)]
            fn safe_shr(self, v: $offset) -> Result<$t, ProtocolZapError> {
                match self.checked_shr(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(ProtocolZapError::MathOverflow)
                    }
                }
            }
        }
    };
}

checked_impl!(u16, u32);
checked_impl!(i32, u32);
checked_impl!(u32, u32);
checked_impl!(u64, u32);
checked_impl!(i64, u32);
checked_impl!(u128, u32);
checked_impl!(i128, u32);
checked_impl!(usize, u32);
checked_impl!(U256, usize);

pub trait SafeCast<T>: Sized {
    fn safe_cast(self) -> Result<T, ProtocolZapError>;
}

macro_rules! try_into_impl {
    ($t:ty, $v:ty) => {
        impl SafeCast<$v> for $t {
            #[track_caller]
            fn safe_cast(self) -> Result<$v, ProtocolZapError> {
                match self.try_into() {
                    Ok(result) => Ok(result),
                    Err(_) => {
                        let caller = Location::caller();
                        msg!("TypeCast is failed at {}:{}", caller.file(), caller.line());
                        Err(ProtocolZapError::TypeCastFailed)
                    }
                }
            }
        }
    };
}

try_into_impl!(usize, u16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_add() {
        assert_eq!(u64::MAX.safe_add(u64::MAX).is_err(), true);
        assert_eq!(100u64.safe_add(100u64).is_ok(), true);
        assert_eq!(100u64.safe_add(100u64).unwrap(), 200u64);
    }

    #[test]
    fn safe_sub() {
        assert_eq!(0u64.safe_sub(u64::MAX).is_err(), true);
        assert_eq!(200u64.safe_sub(100u64).is_ok(), true);
        assert_eq!(200u64.safe_sub(100u64).unwrap(), 100u64);
    }

    #[test]
    fn safe_mul() {
        assert_eq!(u64::MAX.safe_mul(u64::MAX).is_err(), true);
        assert_eq!(100u64.safe_mul(100u64).is_ok(), true);
        assert_eq!(100u64.safe_mul(100u64).unwrap(), 10000u64);
    }

    #[test]
    fn safe_div() {
        assert_eq!(100u64.safe_div(0u64).is_err(), true);
        assert_eq!(200u64.safe_div(100u64).is_ok(), true);
        assert_eq!(200u64.safe_div(100u64), Ok(2u64));
    }

    #[test]
    fn safe_shl() {
        assert_eq!(1u128.safe_shl(8).is_ok(), true);
        assert_eq!(100u128.safe_shl(128).is_err(), true);
        assert_eq!(100u128.safe_shl(8), Ok(25600))
    }

    #[test]
    fn safe_shr() {
        assert_eq!(100u128.safe_shr(1).is_ok(), true);
        assert_eq!(200u128.safe_shr(129).is_err(), true);
        assert_eq!(200u128.safe_shr(1), Ok(100))
    }
}
