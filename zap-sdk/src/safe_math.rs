use crate::error::ZapSdkError;
use ruint::aliases::U256;
use solana_program::msg;
use std::panic::Location;

/// safe math module
pub trait SafeMath<T>: Sized {
    /// safe add
    fn safe_add(self, rhs: Self) -> Result<Self, ZapSdkError>;
    /// safe mul
    fn safe_mul(self, rhs: Self) -> Result<Self, ZapSdkError>;
    /// safe div
    fn safe_div(self, rhs: Self) -> Result<Self, ZapSdkError>;
    /// safe sub
    fn safe_sub(self, rhs: Self) -> Result<Self, ZapSdkError>;
}

macro_rules! checked_impl {
    ($t:ty, $offset:ty) => {
        impl SafeMath<$offset> for $t {
            #[inline(always)]
            fn safe_add(self, v: $t) -> Result<$t, ZapSdkError> {
                match self.checked_add(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(ZapSdkError::MathOverflow)
                    }
                }
            }

            #[inline(always)]
            fn safe_sub(self, v: $t) -> Result<$t, ZapSdkError> {
                match self.checked_sub(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(ZapSdkError::MathOverflow)
                    }
                }
            }

            #[inline(always)]
            fn safe_mul(self, v: $t) -> Result<$t, ZapSdkError> {
                match self.checked_mul(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(ZapSdkError::MathOverflow)
                    }
                }
            }

            #[inline(always)]
            fn safe_div(self, v: $t) -> Result<$t, ZapSdkError> {
                match self.checked_div(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Math error thrown at {}:{}", caller.file(), caller.line());
                        Err(ZapSdkError::MathOverflow)
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
    fn safe_cast(self) -> Result<T, ZapSdkError>;
}

macro_rules! try_into_impl {
    ($t:ty, $v:ty) => {
        impl SafeCast<$v> for $t {
            #[track_caller]
            fn safe_cast(self) -> Result<$v, ZapSdkError> {
                match self.try_into() {
                    Ok(result) => Ok(result),
                    Err(_) => {
                        let caller = Location::caller();
                        msg!("TypeCast is failed at {}:{}", caller.file(), caller.line());
                        Err(ZapSdkError::TypeCastFailed)
                    }
                }
            }
        }
    };
}

try_into_impl!(usize, u16);
