use std::collections::HashMap;
// use winit::event::VirtualKeyCode;

pub const GB_A_BUTTON: u32 = 0x00;
pub const GB_B_BUTTON: u32 = 0x01;
pub const GB_START_BUTTON: u32 = 0x02;
pub const GB_SELECT_BUTTON: u32 = 0x03;
pub const GB_UP_BUTTON: u32 = 0x04;
pub const GB_DOWN_BUTTON: u32 = 0x05;
pub const GB_LEFT_BUTTON: u32 = 0x06;
pub const GB_RIGHT_BUTTON: u32 = 0x07;

struct GBInputMapping {
    pub a: u32,
    pub b: u32,
    pub start: u32,
    pub select: u32,
    pub up: u32,
    pub down: u32,
    pub left: u32,
    pub right: u32,
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
        hashmap.insert(GB_A_BUTTON, 0x5A);
        let mut gb_map = GBInputMapping::new();
        Self {
            mapping: hashmap,
            gb_mapping: gb_map,
        }
    }

    pub fn set_mapping(&mut self, input_code: u32, key_code: u32) {
        self.mapping.insert(input_code, key_code);
    }

    pub fn get_key_code(&self, input_code: u32) -> Option<&u32> {
        self.mapping.get(&input_code)
    }
}
