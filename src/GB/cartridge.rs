pub mod addresses;

#[cfg(test)]
mod tests;
pub mod header;
pub mod controller;

use std::io::prelude::*;
use std::fs::File;
use crate::GB::bus::BusDevice;
use crate::GB::cartridge::addresses::{TITLE, TITLE_OLD_SIZE};
use crate::GB::cartridge::addresses::mbc1::{MBC1_BANKING_MODE_ADDRESS_END, MBC1_BANKING_MODE_ADDRESS_START, MBC1_RAM_BANK_SELECTION_ADDRESS_END, MBC1_RAM_BANK_SELECTION_ADDRESS_START, MBC1_RAM_ENABLE_ADDRESS_END, MBC1_RAM_ENABLE_ADDRESS_START, MBC1_ROM_BANK_SELECTION_ADDRESS_END, MBC1_ROM_BANK_SELECTION_ADDRESS_START};
use crate::GB::memory::Memory;
use crate::GB::memory::addresses::{EXTERNAL_RAM_ADDRESS, EXTERNAL_RAM_LAST_ADDRESS, ROM_BANK_0_ADDRESS, ROM_BANK_0_LAST_ADDRESS, ROM_BANK_1_ADDRESS, ROM_BANK_1_LAST_ADDRESS};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
enum CartridgeController {
    RomOnly = 0,
    Mbc1 = 0x01,
    Mbc1Ram = 0x02,
    Mbc1RamBattery = 0x03,
    Mbc2 = 0x05,
    Mbc2Battery = 0x06,
    RomRam = 0x08,
    RomRamBattery = 0x09,
    Mmm01 = 0x0B,
    Mmm01Ram = 0x0C,
    Mmm01RamBattery = 0x0D,
    Mbc3TimerBattery = 0x0F,
    Mbc3TimerRamBattery = 0x10,
    Mbc3 = 0x11,
    Mbc3Ram = 0x12,
    Mbc3RamBattery = 0x13,
    Mbc5 = 0x19,
    Mbc5Ram = 0x1A,
    Mbc5RamBattery = 0x1B,
    Mbc5Rumble = 0x1C,
    Mbc5RumbleRam = 0x1D,
    Mbc5RumbleRamBattery = 0x1E,
    Mbc6 = 0x20,
    Mbc7SensorRumbleRamBattery = 0x22,
    PocketCamera = 0xFC,
    BandaiTama5 = 0xFD,
    HuC3 = 0xFE,
    HuC1RamBattery = 0xFF,
    Unknown = 0xEE,
}

pub struct Cartridge {
    rom: Box<dyn RomController>,
    // rom: Memory<u8>,
    ram: Memory<u8>,
    cartridge_type: CartridgeController,
    rom_path: String,
    ram_enabled: bool,
    rom_bank: usize,
    ram_bank: usize,
    bank_switch_mode: bool, // False = ROM mode - True = RAM mode
}

/// Alias name for cartridge type, as it is commonly known as ROM
pub type ROM = Cartridge;

pub trait RomController: BusDevice {
    fn new() -> Self;
    fn load(&mut self, rom_path: &str) -> Result<(), std::io::Error>;
    fn header_slice(&self) -> &[u8; 0x50];
}

impl Cartridge {
    pub const ROM_BANK_SIZE: usize = 0x4000;
    pub const RAM_BANK_SIZE: usize = 0x2000;

    pub fn new(file: String) -> Result<Self, std::io::Error> {
        let mut f = File::open(&file)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        let ram_size: usize = match buffer[addresses::ROM_SIZE] {
            0 => 8 * 1024,
            3 => 32 * 1024,
            4 => 128 * 1024,
            5 => 64 * 1024,
            _ => 0
        };
        let cartridge_type: CartridgeController = Self::get_cartridge_type(buffer[addresses::CARTRIDGE_TYPE]);
        Ok(Self {
            rom: Memory::<u8>::new_from_vec(buffer),
            ram: Memory::<u8>::new(0, ram_size),
            cartridge_type,
            rom_path: file,
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            bank_switch_mode: false,
        })
    }

    pub fn get_title(&self) -> String {
        let mut v: Vec<u8> = Vec::with_capacity(TITLE_OLD_SIZE);
        for i in 0..TITLE_OLD_SIZE {
            if self.rom[TITLE + i] == 0 { break };
            v.push(self.rom[TITLE + i]);
        }
        let s = String::from_utf8(v);
        match s {
            Ok(s) => {s}
            Err(_) => {"".to_string()}
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        // TODO: implement read
        let address_usize = address as usize;
        let mut return_val: u8 = 0xFF;
        match self.cartridge_type {
            CartridgeController::Mbc1 => {
                return_val = self.read_mbc1(address_usize);
            }
            _ => {}
        };
        return_val
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let address_usize = address as usize;
        match self.cartridge_type {
            CartridgeController::Mbc1 => {
                self.write_mbc1(address_usize, value);
            }
            _ => {}
        }
    }

    pub fn read_mbc1(&self, address: usize) -> u8 {
        let mut return_val = 0xFF;
        match address {
            ROM_BANK_0_ADDRESS..=ROM_BANK_0_LAST_ADDRESS => {
                return_val = self.rom[address];
            }
            ROM_BANK_1_ADDRESS..=ROM_BANK_1_LAST_ADDRESS => {
                let rom_bank = self.rom_bank | if !self.bank_switch_mode {self.ram_bank << 5} else {0};
                return_val = self.rom[address - ROM_BANK_1_ADDRESS + rom_bank * ROM_BANK_SIZE];
            }
            EXTERNAL_RAM_ADDRESS..=EXTERNAL_RAM_LAST_ADDRESS => {
                let ram_bank = if self.bank_switch_mode {self.ram_bank} else {0};
                return_val = self.ram[address - EXTERNAL_RAM_ADDRESS + ram_bank * RAM_BANK_SIZE];
            }
            _ => {
                // Still here? How do you do?
            }
        };
        return_val
    }

    fn write_mbc1(&mut self, address: usize, value: u8) {
        match address {
            MBC1_RAM_ENABLE_ADDRESS_START..=MBC1_RAM_ENABLE_ADDRESS_END => {
                self.ram_enabled = value == 0x0A;
            }
            MBC1_ROM_BANK_SELECTION_ADDRESS_START..=MBC1_ROM_BANK_SELECTION_ADDRESS_END => {
                // TODO: Check if conform to https://gbdev.io/pandocs/MBC1.html#20003fff--rom-bank-number-write-only
                let mut bank_selection: usize = (value as usize) & 0b0001_1111; // 5 Bits addressing
                if bank_selection == 0 {
                    // Bank ROM 0 is not selectable as it is always addressed on mapped memory
                    bank_selection = 1;
                }
                if self.get_rom_banks_number() <= 0x10 {
                    // If rom banks are less/equal than 16 che chip mask the value as a 4 bit value
                    bank_selection &= 0b0000_1111;
                }
                self.rom_bank = bank_selection;
            }
            MBC1_RAM_BANK_SELECTION_ADDRESS_START..=MBC1_RAM_BANK_SELECTION_ADDRESS_END => {
                let mut bank_selection: usize = (value as usize) & 0b0000_0011; // 2 Bits addressing
                self.ram_bank = bank_selection;
            }
            MBC1_BANKING_MODE_ADDRESS_START..=MBC1_BANKING_MODE_ADDRESS_END => {
                self.bank_switch_mode = (value & 1) != 0;
            }
            EXTERNAL_RAM_ADDRESS..=EXTERNAL_RAM_LAST_ADDRESS => {
                if self.ram_enabled {
                    self.ram[address - EXTERNAL_RAM_ADDRESS] = value;
                }
            }
            _ => {
                // Nothing Happens! How did you arrive here?
            }
        }
    }

    pub fn get_rom_banks_number(&self) -> usize {
        self.rom.len() / ROM_BANK_SIZE
    }

    pub fn get_ram_banks_number(&self) -> usize {
        self.ram.len() / ROM_BANK_SIZE
    }

    pub fn get_cartridge_type(code: u8) -> CartridgeController {
        match code {
            0x00 => CartridgeController::RomOnly,
            0x01 => CartridgeController::Mbc1,
            0x02 => CartridgeController::Mbc1Ram,
            0x03 => CartridgeController::Mbc1RamBattery,
            0x05 => CartridgeController::Mbc2,
            0x06 => CartridgeController::Mbc2Battery,
            0x08 => CartridgeController::RomRam,
            0x09 => CartridgeController::RomRamBattery,
            0x0B => CartridgeController::Mmm01,
            0x0C => CartridgeController::Mmm01Ram,
            0x0D => CartridgeController::Mmm01RamBattery,
            0x0F => CartridgeController::Mbc3TimerBattery,
            0x10 => CartridgeController::Mbc3TimerRamBattery,
            0x11 => CartridgeController::Mbc3,
            0x12 => CartridgeController::Mbc3Ram,
            0x13 => CartridgeController::Mbc3RamBattery,
            0x19 => CartridgeController::Mbc5,
            0x1A => CartridgeController::Mbc5Ram,
            0x1B => CartridgeController::Mbc5RamBattery,
            0x1C => CartridgeController::Mbc5Rumble,
            0x1D => CartridgeController::Mbc5RumbleRam,
            0x1E => CartridgeController::Mbc5RumbleRamBattery,
            0x20 => CartridgeController::Mbc6,
            0x22 => CartridgeController::Mbc7SensorRumbleRamBattery,
            0xFC => CartridgeController::PocketCamera,
            0xFD => CartridgeController::BandaiTama5,
            0xFE => CartridgeController::HuC3,
            0xFF => CartridgeController::HuC1RamBattery,
            _ => CartridgeController::Unknown
        }
    }
    pub fn get_cartridge_type_string(code: &CartridgeController) -> String {
        let s = match code {
            CartridgeController::RomOnly => "ROM-Only",
            CartridgeController::Mbc1 => "MBC1",
            CartridgeController::Mbc1Ram => "MBC1+RAN",
            CartridgeController::Mbc1RamBattery => "MBC1+RAM+BATTERY",
            CartridgeController::Mbc2 => "MBC2",
            CartridgeController::Mbc2Battery => "MBC2+BATTERY",
            CartridgeController::RomRam => "ROM+RAM",
            CartridgeController::RomRamBattery => "ROM+RAM+BATTERY",
            CartridgeController::Mmm01 => "MMM01",
            CartridgeController::Mmm01Ram => "MMM01+RAM",
            CartridgeController::Mmm01RamBattery => "MMM01+RAM+BATTERY",
            CartridgeController::Mbc3TimerBattery => "MBC3+TIMER+BATTERY",
            CartridgeController::Mbc3TimerRamBattery => "MBC3+TIMER+RAM+BATTERY",
            CartridgeController::Mbc3 => "MBC3",
            CartridgeController::Mbc3Ram => "MBC3+RAM",
            CartridgeController::Mbc3RamBattery => "MBC3+RAM+BATTERY",
            CartridgeController::Mbc5 => "MB5",
            CartridgeController::Mbc5Ram => "MBC5+RAM",
            CartridgeController::Mbc5RamBattery => "MBC5+RAM+BATTERY",
            CartridgeController::Mbc5Rumble => "MBC5+RUMBLE",
            CartridgeController::Mbc5RumbleRam => "MBC5+RUMBLE+RAM",
            CartridgeController::Mbc5RumbleRamBattery => "MBC5+RUMBLE+RAM+BATTERY",
            CartridgeController::Mbc6 => "MBC6",
            CartridgeController::Mbc7SensorRumbleRamBattery => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
            CartridgeController::PocketCamera => "POCKET-CAMERA",
            CartridgeController::BandaiTama5 => "BANDAI-TAMA5",
            CartridgeController::HuC3 => "HuC3",
            CartridgeController::HuC1RamBattery => "HuC1+RAM+BATTERY",
            CartridgeController::Unknown => "Unknown",
        };
        s.to_string()
    }

    pub fn get_rom_bank(&self) -> usize {
        self.rom_bank
    }

    pub fn get_ram_bank(&self) -> usize {
        self.ram_bank
    }

    pub fn get_rom_size(&self) -> usize {
        self.rom.len()
    }

    pub fn get_ram_size(&self) -> usize {
        self.ram.len()
    }

    pub fn get_cart_type(&self) -> CartridgeController {
        self.cartridge_type
    }
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cartridge \"{}\"(0x{:02x}) {{ ROM/RAM: {}KB/{}KB, ROM B.: {}, RAM B.: {}, Title: \"{}\", Path: \"{}\" }}",
            Self::get_cartridge_type_string(&self.cartridge_type),
            self.rom[addresses::CARTRIDGE_TYPE],
            self.rom.len() / 1024, self.ram.len() / 1024,
            self.rom_bank, self.ram_bank,
            self.get_title(),
            self.rom_path,
        )
    }
}
