use crate::GB::registers::FlagBits;
macro_rules! mask_flag_enum_default_impl {
    ($enum_name: ident) => {
        impl Into<u8> for $enum_type {
            fn into(self) -> u8 {
                self as u8
            }
        }

        impl std::ops::BitAnd<u8> for FlagBits {
            type Output = u8;

            fn bitand(self, rhs: u8) -> Self::Output {
                self as u8 & rhs
            }
        }

        impl std::ops::BitOr<u8> for FlagBits {
            type Output = u8;

            fn bitor(self, rhs: u8) -> Self::Output {
                self as u8 | rhs
            }
        }

        impl std::ops::BitXor<u8> for FlagBits {
            type Output = u8;

            fn bitxor(self, rhs: u8) -> Self::Output {
                self as u8 ^ rhs
            }
        }

        impl std::ops::Not for FlagBits {
            type Output = u8;

            fn not(self) -> Self::Output {
                !(self as u8)
            }
        }

        impl std::ops::BitAndAssign<FlagBits> for u8 {
            fn bitand_assign(&mut self, rhs: FlagBits){
                *self &= rhs as u8
            }
        }

        impl std::ops::BitOrAssign<FlagBits> for u8 {
            fn bitor_assign(&mut self, rhs: FlagBits){
                *self |= rhs as u8
            }
        }

        impl std::ops::BitXorAssign<FlagBits> for u8 {
            fn bitxor_assign(&mut self, rhs: FlagBits){
                *self ^= rhs as u8
            }
        }

        impl std::ops::BitAnd<FlagBits> for u8 {
            type Output = u8;

            fn bitand(self, rhs: FlagBits) -> Self::Output {
                self & rhs as u8
            }
        }

        impl std::ops::BitOr<FlagBits> for u8 {
            type Output = u8;

            fn bitor(self, rhs: FlagBits) -> Self::Output {
                self | rhs as u8
            }
        }

        impl std::ops::BitXor<FlagBits> for u8 {
            type Output = u8;

            fn bitxor(self, rhs: FlagBits) -> Self::Output {
                self ^ rhs as u8
            }
        }
    };
}