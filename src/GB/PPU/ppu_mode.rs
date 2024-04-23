use crate::{default_enum_u8};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum PPUMode {
    OAMScan = 0,
    Drawing = 3,
    HBlank = 2,
    VBlank = 1,
}

default_enum_u8!(PPUMode {OAMScan = 0, Drawing = 3, HBlank = 2, VBlank = 1});
