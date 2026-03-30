use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::{mask_flag_enum_default_impl, default_enum_u8_bit_ops};
use crate::GB::bus::{Bus, BusDevice, MmioContextWrite};
use crate::GB::cpu::cpu_mmio::CpuMmio;
use crate::GB::cpu::registers::interrupt_registers::{InterruptFlagsMask, InterruptRegisters};
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

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
pub enum JoypadSelectionBits {
    Buttons = 0b_0100_0000,
    DPad = 0b_0010_0000,
}
mask_flag_enum_default_impl!(JoypadSelectionBits);

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum JoypadButtonsBits {
    A = 0b_0000_0001,
    B = 0b_0000_0010,
    Select = 0b_0000_0100,
    Start = 0b_0000_1000,
}
mask_flag_enum_default_impl!(JoypadButtonsBits);

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum JoypadDPadBits {
    Right = 0b_0000_0001,
    Left = 0b_0000_0010,
    Up = 0b_0000_0100,
    Down = 0b_0000_1000,
}
mask_flag_enum_default_impl!(JoypadDPadBits);


#[derive(Copy, Clone, Debug)]
pub enum JoypadButton {
    Button(JoypadButtonsBits),
    DPad(JoypadDPadBits),
}

#[derive(Copy, Clone, Debug)]
struct JoypadMapping {
    pub a: u32,
    pub b: u32,
    pub start: u32,
    pub select: u32,
    pub up: u32,
    pub down: u32,
    pub left: u32,
    pub right: u32,
}

impl JoypadMapping {
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

#[derive(Copy, Clone, Debug)]
pub struct JoypadInputs {
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl JoypadInputs {
    pub fn symbolic_display(&self) -> String {
        let mut string = String::new();
        string.push(if self.up { '↑' } else { '_' });
        string.push(if self.down { '↓' } else { '_' });
        string.push(if self.left { '←' } else { '_' });
        string.push(if self.right { '→' } else { '_' });
        string.push(if self.a { 'A' } else { '_' });
        string.push(if self.b { 'B' } else { '_' });
        string.push(if self.start { '○' } else { '_' });
        string.push(if self.select { '◙' } else { '_' });
        string
    }
}

/// Struct representing state of all buttons on Game Boy. Each button as 2 states: "true" when is pressed, "false" when is not.
///
/// This input structure can write to memory to update status of buttons as reading 0xFF00 memory address returns the status of buttons (bit 0 if button pressed, 1 if not)
pub struct Joypad {
    mode_selection: Byte,
    buttons_byte: Byte,
    dpad_byte: Byte,
    a: bool,
    b: bool,
    start: bool,
    select: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl Joypad {
    pub const JOYPAD_REGISTER_ADDRESS: Address = Address(0xFF00);
    pub const WRITABLE_BITS_MASK: u8 = 0b0011_0000;
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            mode_selection: 0b0011_0000,
            buttons_byte: 0b0000_1111,
            dpad_byte: 0b0000_1111,
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

    pub fn as_byte(&self) -> Byte {
        let mut return_byte = self.mode_selection;
        let button_mode_on = (self.mode_selection & JoypadSelectionBits::Buttons as u8) == 0;
        let dpad_mode_on = (self.mode_selection & JoypadSelectionBits::DPad as u8) == 0;

        // Set lower nibble of return byte
        if button_mode_on && dpad_mode_on {
            return_byte |= self.buttons_byte & self.dpad_byte;
        } else if button_mode_on {
            return_byte |= self.buttons_byte
        } else if dpad_mode_on {
            return_byte |= self.dpad_byte
        } else {
            return_byte |= 0b0000_1111;
        }
        return_byte
    }

    pub fn set_button_status(&mut self, interrupt_registers: &mut InterruptRegisters, btn: JoypadButton, pressed: bool) {
        let old_byte = self.as_byte();
        match btn {
            JoypadButton::Button(btn) => {
                match btn {
                    JoypadButtonsBits::A => {
                        self.a = pressed;
                    }
                    JoypadButtonsBits::B => {
                        self.b = pressed;
                    }
                    JoypadButtonsBits::Select => {
                        self.select = pressed;
                    }
                    JoypadButtonsBits::Start => {
                        self.start = pressed;
                    }
                }
                if pressed {
                    self.buttons_byte &= !(btn as Byte);
                } else {
                    self.buttons_byte |= btn as Byte;
                }
            }
            JoypadButton::DPad(direction) => {
                match direction {
                    JoypadDPadBits::Up => {
                        self.up = pressed;
                    }
                    JoypadDPadBits::Right => {
                        self.right = pressed;
                    }
                    JoypadDPadBits::Down => {
                        self.down = pressed;
                    }
                    JoypadDPadBits::Left => {
                        self.left = pressed;
                    }
                }
                if pressed {
                    self.dpad_byte &= !(direction as Byte);
                } else {
                    self.dpad_byte |= direction as Byte;
                }
            }
        }
        let new_byte = self.as_byte();
        if new_byte < old_byte {
            interrupt_registers.set_if_bit(InterruptFlagsMask::JoyPad);
        }
    }

    // pub fn get_buttons_byte(&self) -> u8 {
    //     0_u8
    //         | ((!self.a as u8) << (JoypadButtonsBits::A as u8).trailing_zeros() | (JoypadButtonsBits::A as u8))
    //         | ((!self.b as u8) << (JoypadButtonsBits::B as u8).trailing_zeros() | (JoypadButtonsBits::B as u8))
    //         | ((!self.select as u8) << (JoypadButtonsBits::Select as u8).trailing_zeros() | (JoypadButtonsBits::Select as u8))
    //         | ((!self.start as u8) << (JoypadButtonsBits::Start as u8).trailing_zeros() | (JoypadButtonsBits::Start as u8))
    // }
    //
    // pub fn get_dpad_byte(&self) -> u8 {
    //     0_u8
    //         | ((!self.right as u8) << (JoypadDPadBits::Right as u8).trailing_zeros() | (JoypadDPadBits::Right as u8))
    //         | ((!self.left as u8) << (JoypadDPadBits::Left as u8).trailing_zeros() | (JoypadDPadBits::Left as u8))
    //         | ((!self.up as u8) << (JoypadDPadBits::Up as u8).trailing_zeros() | (JoypadDPadBits::Up as u8))
    //         | ((!self.down as u8) << (JoypadDPadBits::Down as u8).trailing_zeros() | (JoypadDPadBits::Down as u8))
    // }

    #[inline]
    pub fn joypad_view(&self) -> JoypadInputs {
        JoypadInputs {
            a: self.a,
            b: self.b,
            start: self.start,
            select: self.select,
            up: self.up,
            down: self.down,
            left: self.left,
            right: self.right,
        }
    }

    #[inline]
    pub fn symbolic_display(&self) -> String {
        self.joypad_view().symbolic_display()
    }
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}

impl BusDevice for Joypad {
    fn read(&self, address: Address) -> Byte {
        match address {
            Self::JOYPAD_REGISTER_ADDRESS => {
                self.as_byte()
            },
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: Address, data: Byte) {
        match address {
            Self::JOYPAD_REGISTER_ADDRESS => {
                self.mode_selection = data & Self::WRITABLE_BITS_MASK;
            },
            _ => unreachable!(),
        }
    }
}

struct InputMapping {
    mapping: HashMap<u32, u32>,
    gb_mapping: JoypadMapping,
}

impl InputMapping {
    pub fn new() -> Self {
        let mut hashmap: HashMap<u32, u32> = HashMap::new();
        hashmap.insert(GB_A_BUTTON, 0x5A);
        let mut gb_map = JoypadMapping::new();
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

impl Display for Joypad {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Inputs: [UP: {}, DOWN: {}, LEFT: {}, RIGHT: {}] [A: {}, B: {}, SELECT: {}, START: {}]",
            if self.up { "ON" } else { "OFF" },
            if self.down { "ON" } else { "OFF" },
            if self.left { "ON" } else { "OFF" },
            if self.right { "ON" } else { "OFF" },
            if self.a { "ON" } else { "OFF" },
            if self.b { "ON" } else { "OFF" },
            if self.select { "ON" } else { "OFF" },
            if self.start { "ON" } else { "OFF" },
        )
    }
}
