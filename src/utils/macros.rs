#[macro_export]
macro_rules! mask_flag_enum_default_impl {
    ($type_name: ty) => {
        impl Into<u8> for $type_name {
            fn into(self) -> u8 {
                self as u8
            }
        }

        impl std::ops::BitAnd<u8> for $type_name {
            type Output = u8;

            fn bitand(self, rhs: u8) -> Self::Output {
                self as u8 & rhs
            }
        }

        impl std::ops::BitOr<u8> for $type_name {
            type Output = u8;

            fn bitor(self, rhs: u8) -> Self::Output {
                self as u8 | rhs
            }
        }

        impl std::ops::BitXor<u8> for $type_name {
            type Output = u8;

            fn bitxor(self, rhs: u8) -> Self::Output {
                self as u8 ^ rhs
            }
        }

        impl std::ops::Not for $type_name {
            type Output = u8;

            fn not(self) -> Self::Output {
                !(self as u8)
            }
        }

        impl std::ops::BitAndAssign<$type_name> for u8 {
            fn bitand_assign(&mut self, rhs: $type_name){
                *self &= rhs as u8
            }
        }

        impl std::ops::BitOrAssign<$type_name> for u8 {
            fn bitor_assign(&mut self, rhs: $type_name){
                *self |= rhs as u8
            }
        }

        impl std::ops::BitXorAssign<$type_name> for u8 {
            fn bitxor_assign(&mut self, rhs: $type_name){
                *self ^= rhs as u8
            }
        }

        impl std::ops::BitAnd<$type_name> for u8 {
            type Output = u8;

            fn bitand(self, rhs: $type_name) -> Self::Output {
                self & rhs as u8
            }
        }

        impl std::ops::BitOr<$type_name> for u8 {
            type Output = u8;

            fn bitor(self, rhs: $type_name) -> Self::Output {
                self | rhs as u8
            }
        }

        impl std::ops::BitXor<$type_name> for u8 {
            type Output = u8;

            fn bitxor(self, rhs: $type_name) -> Self::Output {
                self ^ rhs as u8
            }
        }
    };
}