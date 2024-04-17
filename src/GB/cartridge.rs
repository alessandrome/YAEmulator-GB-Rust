mod addresses;

use std::io;
use std::io::prelude::*;
use std::fs::File;
use crate::GB::memory::Memory;

struct Cartridge {
    rom: Memory<u8>,
    ram: Memory<u8>,
    cartridge_type: CartridgeType,
    rom_path: String,
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
        f.read_exact(&mut buffer)?;
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
        })
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
}