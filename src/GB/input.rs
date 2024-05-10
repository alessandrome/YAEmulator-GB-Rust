use std::collections::HashMap;

pub const GB_A_BUTTON: u32 = 0x00;
pub const GB_B_BUTTON: u32 = 0x01;
pub const GB_START_BUTTON: u32 = 0x02;
pub const GB_SELECT_BUTTON: u32 = 0x03;
pub const GB_UP_BUTTON: u32 = 0x04;
pub const GB_DOWN_BUTTON: u32 = 0x05;
pub const GB_LEFT_BUTTON: u32 = 0x06;
pub const GB_RIGHT_BUTTON: u32 = 0x07;

struct GBInputMapping {
    a: u32,
    b: u32,
    start: u32,
    select: u32,
    up: u32,
    down: u32,
    left: u32,
    right: u32,
}

impl GBInputMapping {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            start: 0,
            select: 0,
            up: 0,
            down: 0,
            left: 0,
            right: 0,
        }
    }
}

struct InputMapping {
    mapping: HashMap<u32, u32>,
    gb_mapping: GBInputMapping,
}

impl InputMapping {
    pub fn new() -> Self {
        let mut hashmap: HashMap<u32, u32> = HashMap::new();
        let mut gb_map = GBInputMapping::new();
        Self {
            mapping: hashmap,
            gb_mapping: gb_map,
        }
    }
}
