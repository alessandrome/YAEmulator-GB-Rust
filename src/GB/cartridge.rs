mod addresses;

use std::io;
use std::io::prelude::*;
use std::fs::File;
use crate::GB::memory::Memory;

struct Cartridge<const N: usize, const M: usize> {
    rom: Memory<u8, N>,
    ram: Memory<u8, M>,
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
    Unknown
}

impl<const N: usize, const M: usize> Cartridge<N, M> {
    pub fn new(file: String) -> Result<Self, std::io::Error> {
        let mut f = File::open(&file)?;
        let mut buffer = Vec::new();
        f.read_exact(&mut buffer)?;
        let ram_size: usize = match buffer[addresses::ROM_SIZE] {
            2 => 8 * 1024,
            3 => 32 * 1024,
            4 => 128 * 1024,
            5 => 64 * 1024,
            _ => 0
        };
        let rom_size: usize = buffer.len();
        Ok(Self {
            rom: Memory::<u8, rom_size>::new(0),
            ram: Memory::<u8, ram_size>::new(0),
            cartridge_type: CartridgeType::RomOnly,
            rom_path: file,
        })
    }
}