use std::ops::{Add, Sub, BitAnd, BitOr, BitXor, Shl, Shr, RangeInclusive, Range};

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Address(pub u16);

impl Address {
    pub const ZERO: Address = Address(0);

    #[inline]
    pub fn hi(self) -> u8 {
        (self.0 >> 8) as u8
    }

    #[inline]
    pub fn lo(self) -> u8 {
        self.0 as u8
    }

    #[inline]
    pub fn to_index(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn from_index(index: usize) -> Address {
        Address(index as u16)
    }
}

// ---------------------------------------
// Operator overloading (as Left operator)
// ---------------------------------------

impl Add<u16> for Address {
    type Output = Address;
    #[inline]
    fn add(self, rhs: u16) -> Self::Output {
        Address(self.0.wrapping_add(rhs))
    }
}

impl Sub<u16> for Address {
    type Output = Address;
    #[inline]
    fn sub(self, rhs: u16) -> Self::Output {
        Address(self.0.wrapping_sub(rhs))
    }
}

impl BitAnd<u16> for Address {
    type Output = Address;
    #[inline]
    fn bitand(self, rhs: u16) -> Self::Output {
        Address(self.0 & rhs)
    }
}

impl BitOr<u16> for Address {
    type Output = Address;
    #[inline]
    fn bitor(self, rhs: u16) -> Self::Output {
        Address(self.0 | rhs)
    }
}

impl BitXor<u16> for Address {
    type Output = Address;
    #[inline]
    fn bitxor(self, rhs: u16) -> Self::Output {
        Address(self.0 ^ rhs)
    }
}

impl Shl<u32> for Address {
    type Output = Address;
    #[inline]
    fn shl(self, rhs: u32) -> Self::Output {
        Address(self.0 << rhs)
    }
}

impl Shr<u32> for Address {
    type Output = Address;
    #[inline]
    fn shr(self, rhs: u32) -> Self::Output {
        Address(self.0 >> rhs)
    }
}

// ---------------------------------------
// Operator overloading (as Right operator)
// ---------------------------------------
impl Add<Address> for u16 {
    type Output = Address;
    #[inline]
    fn add(self, rhs: Address) -> Self::Output {
        Address(self.wrapping_add(rhs.0))
    }
}

impl Sub<Address> for u16 {
    type Output = Address;
    #[inline]
    fn sub(self, rhs: Address) -> Self::Output {
        Address(self.wrapping_sub(rhs.0))
    }
}

impl BitAnd<Address> for u16 {
    type Output = Address;
    #[inline]
    fn bitand(self, rhs: Address) -> Self::Output {
        Address(self & rhs.0)
    }
}

impl BitOr<Address> for u16 {
    type Output = Address;
    #[inline]
    fn bitor(self, rhs: Address) -> Self::Output {
        Address(self | rhs.0)
    }
}

impl BitXor<Address> for u16 {
    type Output = Address;
    #[inline]
    fn bitxor(self, rhs: Address) -> Self::Output {
        Address(self ^ rhs.0)
    }
}

impl Shl<Address> for u16 {
    type Output = Address;
    #[inline]
    fn shl(self, rhs: Address) -> Self::Output {
        Address(self << rhs.0)
    }
}

impl Shr<Address> for u16 {
    type Output = Address;
    #[inline]
    fn shr(self, rhs: Address) -> Self::Output {
        Address(self >> rhs.0)
    }
}

// --------------------------
// Other Addres-related types
// --------------------------

pub type AddressSize = u16;
pub type AddressRange = Range<AddressSize>;
pub type AddressRangeInclusive = RangeInclusive<AddressSize>;
