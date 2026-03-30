pub mod addresses;

#[cfg(test)]
mod tests;
pub mod header;
pub mod controller;

use std::io::prelude::*;
use std::fs::File;
use crate::GB::cartridge::header::RomHeader;
// use crate::GB::memory::addresses::{EXTERNAL_RAM_ADDRESS, EXTERNAL_RAM_LAST_ADDRESS, ROM_BANK_0_ADDRESS, ROM_BANK_0_LAST_ADDRESS, ROM_BANK_1_ADDRESS, ROM_BANK_1_LAST_ADDRESS};
use controller::{CartridgeControllerType, RomController};
use crate::GB::bus::BusDevice;
use crate::GB::types::address::{Address, AddressRangeInclusive};
use crate::GB::types::Byte;

pub struct Cartridge {
    rom: Box<dyn RomController>,
    rom_path: String,
}

/// Alias name for cartridge type, as it is commonly known as ROM
pub type ROM = Cartridge;

impl Cartridge {
    pub const CART_ROM_START_ADDRESS: Address = Address(0x0);
    pub const CART_ROM_END_ADDRESS: Address = Address(0x7FFF);
    pub const CART_ROM_RANGE_ADDRESS: AddressRangeInclusive = Self::CART_ROM_START_ADDRESS..=Self::CART_ROM_END_ADDRESS;
    pub const CART_RAM_START_ADDRESS: Address = Address(0xA000);
    pub const CART_RAM_END_ADDRESS: Address = Address(0xBFFF);
    pub const CART_RAM_RANGE_ADDRESS: AddressRangeInclusive = Self::CART_RAM_START_ADDRESS..=Self::CART_RAM_END_ADDRESS;
    pub const ROM_BANK_SIZE: usize = 0x4000;
    pub const RAM_BANK_SIZE: usize = 0x2000;

    pub fn new(file: String) -> Result<Self, std::io::Error> {
        let mut f = File::open(&file)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;

        // TODO: maybe a header should built there and then use it to build a rom item
        // let cartridge_type: CartridgeControllerType = Self::get_cartridge_type(buffer[addresses::CARTRIDGE_TYPE]);
        let rom = controller::Mbc1::new(f)?; // TODO: move to match sentence

        Ok(Self {
            rom: Box::new(rom),
            rom_path: file,
        })
    }

    #[inline]
    pub fn header(&self) -> &RomHeader {
        self.rom.header()
    }

    pub fn title(&self) -> &String {
        self.header().title()
    }

    pub fn cart_type(&self) -> CartridgeControllerType {
        self.header().rom_controller_type()
    }

    pub fn rom_bank(&self) -> u16 {
        self.rom.high_rom_bank_addressed()
    }

    pub fn ram_bank(&self) -> u16 {
        self.rom.ram_bank_addressed()
    }
}

impl BusDevice for Cartridge {
    fn read(&self, address: Address) -> Byte {
        self.rom.read(address)
    }

    fn write(&mut self, address: Address, data: Byte) {
        self.rom.write(address, data);
    }
}

impl std::fmt::Display for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, /*"TODO"*/
            "Cartridge \"{}\"(0x{:02x}) {{ ROM/RAM: {}KB/{}KB, ROM B.: {}, RAM B.: {}, Title: \"{}\", Path: \"{}\" }}",
            self.cart_type() as u8, // TODO: This should be the string version of the controller
            // self.rom[addresses::CARTRIDGE_TYPE],
            self.cart_type() as u8,
            self.header().rom_banks() * 0x4000 / 1024, self.header().ram_banks() * 0x2000 / 1024,
            self.rom_bank(), self.ram_bank(),
            self.title(),
            self.rom_path,
        )
    }
}
