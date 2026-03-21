pub mod addresses;

#[cfg(test)]
mod tests;
pub mod header;
mod controller;

use std::io::prelude::*;
use std::fs::File;
use crate::GB::bus::BusDevice;
use crate::GB::cartridge::addresses::{TITLE, TITLE_OLD_SIZE};
use crate::GB::cartridge::header::RomHeader;
// use crate::GB::memory::addresses::{EXTERNAL_RAM_ADDRESS, EXTERNAL_RAM_LAST_ADDRESS, ROM_BANK_0_ADDRESS, ROM_BANK_0_LAST_ADDRESS, ROM_BANK_1_ADDRESS, ROM_BANK_1_LAST_ADDRESS};
use controller::CartridgeControllerType;

pub struct Cartridge {
    rom: Box<dyn RomController>,
    rom_path: String,
}

/// Alias name for cartridge type, as it is commonly known as ROM
pub type ROM = Cartridge;

pub trait RomController: BusDevice {
    fn load(&mut self, rom_path: &str) -> Result<(), std::io::Error>;
    fn header_slice(&self) -> &[u8; 0x50];
    fn header(&self) -> &RomHeader;
}

pub trait RomLoader: RomController {
    fn new(file: File) -> Result<Self, std::io::Error>;
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
        let cartridge_type: CartridgeControllerType = Self::get_cartridge_type(buffer[addresses::CARTRIDGE_TYPE]);
        let rom = controller::Mbc1::new(f)?; // TODO: move to match sentence
        match cartridge_type {
            CartridgeControllerType::RomOnly => {}
            CartridgeControllerType::Mbc1 => {}
            CartridgeControllerType::Mbc1Ram => {}
            CartridgeControllerType::Mbc1RamBattery => {}
            CartridgeControllerType::Mbc2 => {}
            CartridgeControllerType::Mbc2Battery => {}
            CartridgeControllerType::RomRam => {}
            CartridgeControllerType::RomRamBattery => {}
            CartridgeControllerType::Mmm01 => {}
            CartridgeControllerType::Mmm01Ram => {}
            CartridgeControllerType::Mmm01RamBattery => {}
            CartridgeControllerType::Mbc3TimerBattery => {}
            CartridgeControllerType::Mbc3TimerRamBattery => {}
            CartridgeControllerType::Mbc3 => {}
            CartridgeControllerType::Mbc3Ram => {}
            CartridgeControllerType::Mbc3RamBattery => {}
            CartridgeControllerType::Mbc5 => {}
            CartridgeControllerType::Mbc5Ram => {}
            CartridgeControllerType::Mbc5RamBattery => {}
            CartridgeControllerType::Mbc5Rumble => {}
            CartridgeControllerType::Mbc5RumbleRam => {}
            CartridgeControllerType::Mbc5RumbleRamBattery => {}
            CartridgeControllerType::Mbc6 => {}
            CartridgeControllerType::Mbc7SensorRumbleRamBattery => {}
            CartridgeControllerType::PocketCamera => {}
            CartridgeControllerType::BandaiTama5 => {}
            CartridgeControllerType::HuC3 => {}
            CartridgeControllerType::HuC1RamBattery => {}
            CartridgeControllerType::Unknown => {}
        }
        Ok(Self {
            rom: Box::new(rom),
            rom_path: file,
        })
    }

    pub fn header(&self) -> &RomHeader {
        self.rom.header()
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
            CartridgeControllerType::Mbc1 => {
                return_val = self.read_mbc1(address_usize);
            }
            _ => {}
        };
        return_val
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let address_usize = address as usize;
        match self.cartridge_type {
            CartridgeControllerType::Mbc1 => {
                self.write_mbc1(address_usize, value);
            }
            _ => {}
        }
    }

    pub fn get_cartridge_type(code: u8) -> CartridgeControllerType {
        match code {
            0x00 => CartridgeControllerType::RomOnly,
            0x01 => CartridgeControllerType::Mbc1,
            0x02 => CartridgeControllerType::Mbc1Ram,
            0x03 => CartridgeControllerType::Mbc1RamBattery,
            0x05 => CartridgeControllerType::Mbc2,
            0x06 => CartridgeControllerType::Mbc2Battery,
            0x08 => CartridgeControllerType::RomRam,
            0x09 => CartridgeControllerType::RomRamBattery,
            0x0B => CartridgeControllerType::Mmm01,
            0x0C => CartridgeControllerType::Mmm01Ram,
            0x0D => CartridgeControllerType::Mmm01RamBattery,
            0x0F => CartridgeControllerType::Mbc3TimerBattery,
            0x10 => CartridgeControllerType::Mbc3TimerRamBattery,
            0x11 => CartridgeControllerType::Mbc3,
            0x12 => CartridgeControllerType::Mbc3Ram,
            0x13 => CartridgeControllerType::Mbc3RamBattery,
            0x19 => CartridgeControllerType::Mbc5,
            0x1A => CartridgeControllerType::Mbc5Ram,
            0x1B => CartridgeControllerType::Mbc5RamBattery,
            0x1C => CartridgeControllerType::Mbc5Rumble,
            0x1D => CartridgeControllerType::Mbc5RumbleRam,
            0x1E => CartridgeControllerType::Mbc5RumbleRamBattery,
            0x20 => CartridgeControllerType::Mbc6,
            0x22 => CartridgeControllerType::Mbc7SensorRumbleRamBattery,
            0xFC => CartridgeControllerType::PocketCamera,
            0xFD => CartridgeControllerType::BandaiTama5,
            0xFE => CartridgeControllerType::HuC3,
            0xFF => CartridgeControllerType::HuC1RamBattery,
            _ => CartridgeControllerType::Unknown
        }
    }
    pub fn get_cartridge_type_string(code: &CartridgeControllerType) -> String {
        let s = match code {
            CartridgeControllerType::RomOnly => "ROM-Only",
            CartridgeControllerType::Mbc1 => "MBC1",
            CartridgeControllerType::Mbc1Ram => "MBC1+RAN",
            CartridgeControllerType::Mbc1RamBattery => "MBC1+RAM+BATTERY",
            CartridgeControllerType::Mbc2 => "MBC2",
            CartridgeControllerType::Mbc2Battery => "MBC2+BATTERY",
            CartridgeControllerType::RomRam => "ROM+RAM",
            CartridgeControllerType::RomRamBattery => "ROM+RAM+BATTERY",
            CartridgeControllerType::Mmm01 => "MMM01",
            CartridgeControllerType::Mmm01Ram => "MMM01+RAM",
            CartridgeControllerType::Mmm01RamBattery => "MMM01+RAM+BATTERY",
            CartridgeControllerType::Mbc3TimerBattery => "MBC3+TIMER+BATTERY",
            CartridgeControllerType::Mbc3TimerRamBattery => "MBC3+TIMER+RAM+BATTERY",
            CartridgeControllerType::Mbc3 => "MBC3",
            CartridgeControllerType::Mbc3Ram => "MBC3+RAM",
            CartridgeControllerType::Mbc3RamBattery => "MBC3+RAM+BATTERY",
            CartridgeControllerType::Mbc5 => "MB5",
            CartridgeControllerType::Mbc5Ram => "MBC5+RAM",
            CartridgeControllerType::Mbc5RamBattery => "MBC5+RAM+BATTERY",
            CartridgeControllerType::Mbc5Rumble => "MBC5+RUMBLE",
            CartridgeControllerType::Mbc5RumbleRam => "MBC5+RUMBLE+RAM",
            CartridgeControllerType::Mbc5RumbleRamBattery => "MBC5+RUMBLE+RAM+BATTERY",
            CartridgeControllerType::Mbc6 => "MBC6",
            CartridgeControllerType::Mbc7SensorRumbleRamBattery => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
            CartridgeControllerType::PocketCamera => "POCKET-CAMERA",
            CartridgeControllerType::BandaiTama5 => "BANDAI-TAMA5",
            CartridgeControllerType::HuC3 => "HuC3",
            CartridgeControllerType::HuC1RamBattery => "HuC1+RAM+BATTERY",
            CartridgeControllerType::Unknown => "Unknown",
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

    pub fn get_cart_type(&self) -> CartridgeControllerType {
        self.cartridge_type
    }
}

impl std::fmt::Display for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
