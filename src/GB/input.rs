use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use winit::{keyboard::{KeyCode, PhysicalKey}};
use crate::GB::memory::{addresses, RAM, UseMemory};
use crate::{mask_flag_enum_default_impl, default_enum_u8_bit_ops};
// use winit::event::VirtualKeyCode;

pub const GB_A_BUTTON: u32 = 0x00;
pub const GB_B_BUTTON: u32 = 0x01;
pub const GB_START_BUTTON: u32 = 0x02;
pub const GB_SELECT_BUTTON: u32 = 0x03;
pub const GB_UP_BUTTON: u32 = 0x04;
pub const GB_DOWN_BUTTON: u32 = 0x05;
pub const GB_LEFT_BUTTON: u32 = 0x06;
pub const GB_RIGHT_BUTTON: u32 = 0x07;


#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum GBInputSelectionBits {
    Buttons = 0b_0100_0000,
    DPad = 0b_0010_0000,
}
mask_flag_enum_default_impl!(GBInputSelectionBits);

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum GBInputButtonsBits {
    A = 0b_0000_0001,
    B = 0b_0000_0010,
    Select = 0b_0000_0100,
    Start = 0b_0000_1000,
}
mask_flag_enum_default_impl!(GBInputButtonsBits);

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum GBInputDPadBits {
    Right = 0b_0000_0001,
    Left = 0b_0000_0010,
    Up = 0b_0000_0100,
    Down = 0b_0000_1000,
}
mask_flag_enum_default_impl!(GBInputDPadBits);

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

/// Struct representing state of all buttons on Game Boy. Each button as 2 states: "true" when is pressed, "false" when is not.
///
/// This input structure can write to memory to update status of buttons as reading 0xFF00 memory address returns the status of buttons (bit 0 if button pressed, 1 if not)
pub struct GBInput {
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl GBInput {
    pub fn get_buttons_byte(&self) -> u8 {
        0_u8
            | ((!self.a as u8) << (GBInputButtonsBits::A as u8).trailing_zeros() | (GBInputButtonsBits::A as u8))
            | ((!self.b as u8) << (GBInputButtonsBits::B as u8).trailing_zeros() | (GBInputButtonsBits::B as u8))
            | ((!self.select as u8) << (GBInputButtonsBits::Select as u8).trailing_zeros() | (GBInputButtonsBits::Select as u8))
            | ((!self.start as u8) << (GBInputButtonsBits::Start as u8).trailing_zeros() | (GBInputButtonsBits::Start as u8))
    }

    pub fn get_dpad_byte(&self) -> u8 {
        0_u8
            | ((!self.right as u8) << (GBInputDPadBits::Right as u8).trailing_zeros() | (GBInputDPadBits::Right as u8))
            | ((!self.left as u8) << (GBInputDPadBits::Left as u8).trailing_zeros() | (GBInputDPadBits::Left as u8))
            | ((!self.up as u8) << (GBInputDPadBits::Up as u8).trailing_zeros() | (GBInputDPadBits::Up as u8))
            | ((!self.down as u8) << (GBInputDPadBits::Down as u8).trailing_zeros() | (GBInputDPadBits::Down as u8))
    }
}

impl Default for GBInput {
    fn default() -> Self {
        Self {
            a: false,
            b: false,
            start: false,
            select: false,
            up: false,
            down: false,
            left: false,
            right: false,
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
