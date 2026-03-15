pub mod new_license_code;

use crate::GB::types::address::Address;
use crate::GB::types::Byte;

const HEADER_START_ADDRESS: Address = Address(0x0100);
const HEADER_END_ADDRESS: Address = Address(0x014F);

pub struct RomHeader<'a> {
    raw_header: [u8; HEADER_END_ADDRESS.as_usize() - HEADER_START_ADDRESS.as_usize()],
    title: &'a str,
}

impl RomHeader<'_> {
    pub const HEADER_START_ADDRESS: Address = HEADER_START_ADDRESS;
    pub const HEADER_END_ADDRESS: Address = HEADER_END_ADDRESS;
    pub const HEADER_SIZE: usize = Self::HEADER_END_ADDRESS.as_usize() - HEADER_START_ADDRESS.as_usize();
    pub const HEADER_TITLE_START_ADDRESS: Address = Address(0x0134);
    pub const HEADER_TITLE_END_ADDRESS: Address = Address(0x0143);
}

impl RomHeader<'_> {
    pub fn new(header_slice: &[Byte; Self::HEADER_SIZE]) -> Self {
        let title_result = str::from_utf8(
            &header_slice[
                (Self::HEADER_TITLE_START_ADDRESS.as_usize() - Self::HEADER_START_ADDRESS.as_usize())..=(Self::HEADER_TITLE_START_ADDRESS.as_usize() - Self::HEADER_END_ADDRESS.as_usize())
                ]
        ).unwrap();
        Self {
            raw_header: header_slice.clone(),
            title: title_result,
        }
    }
}
