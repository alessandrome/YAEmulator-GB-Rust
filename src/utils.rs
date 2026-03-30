use std::ops::{BitAnd, Shl};

#[macro_use]
pub mod macros;

pub fn falling_edge<T, B>(old_value: T, new_value: T, bit: B) -> bool
where
    T: Shl<B, Output = T> + From<u8> + std::ops::BitAnd<Output = T> + PartialEq + Copy,
    B: Copy,
{
    let mask: T = (T::from(1)) << bit;
    (old_value & mask) != T::from(0) && (new_value & mask) == T::from(0)
}

/// Take a byte (u8) and expand it adding zeroes between original bits. Zeroes are added at left of the bit.
/// E.g. 1001_1011 become the following 16-bit value 0100_0001_0100_0101
pub fn expand_byte_bits(byte: u8) -> u16 {
    let mut x = byte as u16;
    x = (x | (x << 4)) & 0x0F0F;
    x = (x | (x << 2)) & 0x3333;
    x = (x | (x << 1)) & 0x5555;
    x
}

#[cfg(test)]
mod test {
    use crate::utils::falling_edge;
    macro_rules! test_falling_edge {
        ($test_name: ident, $value_type: ty, $bit_type: ty) => {
            #[test]
            fn $test_name() {
                assert_eq!(falling_edge::<$value_type, $bit_type>(0b00, 0b00, 0), false);
                assert_eq!(falling_edge::<$value_type, $bit_type>(0b00, 0b01, 0), false);
                assert_eq!(falling_edge::<$value_type, $bit_type>(0b01, 0b00, 0), true);
                assert_eq!(falling_edge::<$value_type, $bit_type>(0b10, 0b00, 0), false);
                
                assert_eq!(falling_edge::<$value_type, $bit_type>(0b10, 0b10, 1), false);
                assert_eq!(falling_edge::<$value_type, $bit_type>(0b00, 0b11, 1), false);
                assert_eq!(falling_edge::<$value_type, $bit_type>(0b10, 0b00, 1), true);
                assert_eq!(falling_edge::<$value_type, $bit_type>(0b10, 0b01, 1), true);
            }
        }
    }
    
    test_falling_edge!(test_falling_edge_u8, u8, u8);
    test_falling_edge!(test_falling_edge_u16_val_u8_shift, u16, u8);
    test_falling_edge!(test_falling_edge_u16, u16, u16);
}