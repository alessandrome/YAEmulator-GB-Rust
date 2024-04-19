mod addresses;

#[cfg(test)]
mod tests;

use std::{fmt, io};
use std::cell::RefCell;
use std::io::prelude::*;
use std::fs::File;
use std::rc::Rc;
use std::string::FromUtf8Error;
use crate::GB::cartridge::addresses::{MBC_RAM_ENABLE_ADDRESS_START, MBC_RAM_ENABLE_ADDRESS_END, TITLE, TITLE_OLD_SIZE, MBC_ROM_BANK_SELECTION_ADDRESS_START, MBC_ROM_BANK_SELECTION_ADDRESS_END};
use crate::GB::memory::Memory;
use crate::GB::registers::Registers;

pub struct Cartridge {
    rom: Memory<u8>,
    ram: Memory<u8>,
    cartridge_type: CartridgeType,
    rom_path: String,
    ram_enabled: bool,
    rom_bank: usize,
    ram_bank: usize,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum CartridgeType {
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

impl Cartridge {
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
        let cartridge_type: CartridgeType = Self::get_cartridge_type(buffer[addresses::CARTRIDGE_TYPE]);
        Ok(Self {
            rom: Memory::<u8>::new_from_vec(buffer),
            ram: Memory::<u8>::new(0, ram_size),
            cartridge_type,
            rom_path: file,
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 1,
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
        0
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let address_usize = address as usize;
        match address_usize {
            MBC_RAM_ENABLE_ADDRESS_START..=MBC_RAM_ENABLE_ADDRESS_END => {
                self.ram_enabled = value == 0x0A;
            }
            MBC_ROM_BANK_SELECTION_ADDRESS_START..=MBC_ROM_BANK_SELECTION_ADDRESS_END => {
                // TODO: implement
            }
            _ => {
                // TODO: implement
            }
        }
    }

    pub fn get_cartridge_type(code: u8) -> CartridgeType {
        match code {
            0x00 => CartridgeType::RomOnly,
            0x01 => CartridgeType::Mbc1,
            0x02 => CartridgeType::Mbc1Ram,
            0x03 => CartridgeType::Mbc1RamBattery,
            0x05 => CartridgeType::Mbc2,
            0x06 => CartridgeType::Mbc2Battery,
            0x08 => CartridgeType::RomRam,
            0x09 => CartridgeType::RomRamBattery,
            0x0B => CartridgeType::Mmm01,
            0x0C => CartridgeType::Mmm01Ram,
            0x0D => CartridgeType::Mmm01RamBattery,
            0x0F => CartridgeType::Mbc3TimerBattery,
            0x10 => CartridgeType::Mbc3TimerRamBattery,
            0x11 => CartridgeType::Mbc3,
            0x12 => CartridgeType::Mbc3Ram,
            0x13 => CartridgeType::Mbc3RamBattery,
            0x19 => CartridgeType::Mbc5,
            0x1A => CartridgeType::Mbc5Ram,
            0x1B => CartridgeType::Mbc5RamBattery,
            0x1C => CartridgeType::Mbc5Rumble,
            0x1D => CartridgeType::Mbc5RumbleRam,
            0x1E => CartridgeType::Mbc5RumbleRamBattery,
            0x20 => CartridgeType::Mbc6,
            0x22 => CartridgeType::Mbc7SensorRumbleRamBattery,
            0xFC => CartridgeType::PocketCamera,
            0xFD => CartridgeType::BandaiTama5,
            0xFE => CartridgeType::HuC3,
            0xFF => CartridgeType::HuC1RamBattery,
            _ => CartridgeType::Unknown
        }
    }
    pub fn get_cartridge_type_string(code: &CartridgeType) -> String {
        let s = match code {
            CartridgeType::RomOnly => "ROM-Only",
            CartridgeType::Mbc1 => "MBC1",
            CartridgeType::Mbc1Ram => "MBC1+RAN",
            CartridgeType::Mbc1RamBattery => "MBC1+RAN+BATTERY",
            CartridgeType::Mbc2 => "MBC2",
            CartridgeType::Mbc2Battery => "MBC2+BATTERY",
            CartridgeType::RomRam => "ROM+RAM",
            CartridgeType::RomRamBattery => "ROM+RAM+BATTERY",
            CartridgeType::Mmm01 => "MMM01",
            CartridgeType::Mmm01Ram => "MMM01+RAM",
            CartridgeType::Mmm01RamBattery => "MMM01+RAM+BATTERY",
            CartridgeType::Mbc3TimerBattery => "MBC3+TIMER+BATTERY",
            CartridgeType::Mbc3TimerRamBattery => "MBC3+TIMER+RAM+BATTERY",
            CartridgeType::Mbc3 => "MBC3",
            CartridgeType::Mbc3Ram => "MBC3+RAM",
            CartridgeType::Mbc3RamBattery => "MBC3+RAM+BATTERY",
            CartridgeType::Mbc5 => "MB5",
            CartridgeType::Mbc5Ram => "MBC5+RAM",
            CartridgeType::Mbc5RamBattery => "MBC5+RAM+BATTERY",
            CartridgeType::Mbc5Rumble => "MBC5+RUMBLE",
            CartridgeType::Mbc5RumbleRam => "MBC5+RUMBLE+RAM",
            CartridgeType::Mbc5RumbleRamBattery => "MBC5+RUMBLE+RAM+BATTERY",
            CartridgeType::Mbc6 => "MBC6",
            CartridgeType::Mbc7SensorRumbleRamBattery => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
            CartridgeType::PocketCamera => "POCKET-CAMERA",
            CartridgeType::BandaiTama5 => "BANDAI-TAMA5",
            CartridgeType::HuC3 => "HuC3",
            CartridgeType::HuC1RamBattery => "HuC1+RAM+BATTERY",
            CartridgeType::Unknown => "Unknown",
        };
        s.to_string()
    }
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cartridge \"{}\"(0x{:02x}) {{ ROM/RAM: {}KB/{}KB, Title: \"{}\", Path: \"{}\" }}",
            Self::get_cartridge_type_string(&self.cartridge_type),
            self.rom[addresses::CARTRIDGE_TYPE],
            self.rom.len() / 1024, self.ram.len() / 1024,
            self.get_title(),
            self.rom_path,
        )
    }
}

pub trait UseCartridge {
    fn set_cartridge(&mut self, rom: Rc<RefCell<Option<Cartridge>>>);
}
