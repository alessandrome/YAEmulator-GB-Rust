use std::fs::File;
use std::io::{Read, Seek};
use crate::GB::bus::BusDevice;
use crate::GB::cartridge::{RomController, RomLoader};
use crate::GB::cartridge::header::RomHeader;
use crate::GB::memory::Memory;
use crate::GB::types::address::{Address, AddressRangeInclusive};
use crate::GB::types::Byte;

const MBC1_ROM_BANKS: usize = 80;
pub const MBC1_RAM_BANKS: usize = 4;

#[derive(Copy, Clone, Debug)]
enum Mbc1BankMode {
    Simple,
    Advanced,
}

struct Mbc1Mask {}
impl Mbc1Mask {
    const ROM_BANK: u8 = 0b_0001_1111;
    const RAM_BANK: u8 = 0b_0000_0011;
    const ENABLE_RAM: u8 = 0b_0000_1111;
    const ROM_BANK_MODE: u8 = 0b_0000_0001;
}

#[derive(Clone, Debug)]
pub struct Mbc1 {
    header: RomHeader,
    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,
    rom: Vec<Byte>,
    ram: Vec<Byte>,
    banking_mode: Mbc1BankMode,
}

impl Mbc1 {
    pub const MBC1_ROM_BANKS: usize = 80;
    pub const MBC1_ROM_BANK_SIZE: usize = 0x4000;
    pub const MBC1_RAM_BANKS: usize = 4;
    pub const MBC1_RAM_BANK_SIZE: usize = 0x2000;
    pub const MBC1_RAM_ENABLE_VALUE: u8 = 0xA;
    pub const MBC1_ROM_BANK_0_START: Address = Address(0x0);
    pub const MBC1_ROM_BANK_0_END: Address = Address(0x3FFF);
    pub const MBC1_ROM_BANK_0_RANGE: AddressRangeInclusive = Self::MBC1_ROM_BANK_0_START..=Self::MBC1_ROM_BANK_0_END;
    pub const MBC1_ROM_BANK_1_START: Address = Address(0x4000);
    pub const MBC1_ROM_BANK_1_END: Address = Address(0x7FFF);
    pub const MBC1_ROM_BANK_1_RANGE: AddressRangeInclusive = Self::MBC1_ROM_BANK_1_START..=Self::MBC1_ROM_BANK_1_END;
    pub const MBC1_RAM_BANK_0_START: Address = Address(0xA000);
    pub const MBC1_RAM_BANK_0_END: Address = Address(0xBFFF);
    pub const MBC1_RAM_BANK_0_RANGE: AddressRangeInclusive = Self::MBC1_RAM_BANK_0_START..=Self::MBC1_RAM_BANK_0_END;
    pub const MBC1_RAM_ENABLE_START: Address = Address(0x0);
    pub const MBC1_RAM_ENABLE_END: Address = Address(0x1FFF);
    pub const MBC1_RAM_ENABLE_RANGE: AddressRangeInclusive = Self::MBC1_RAM_ENABLE_START..=Self::MBC1_RAM_ENABLE_END;
    pub const MBC1_ROM_BANK_SELECTOR_START: Address = Address(0x2000);
    pub const MBC1_ROM_BANK_SELECTOR_END: Address = Address(0x3FFF);
    pub const MBC1_ROM_BANK_SELECTOR_RANGE: AddressRangeInclusive = Self::MBC1_ROM_BANK_SELECTOR_START..=Self::MBC1_ROM_BANK_SELECTOR_END;
    pub const MBC1_RAM_BANK_SELECTOR_START: Address = Address(0x4000);
    pub const MBC1_RAM_BANK_SELECTOR_END: Address = Address(0x5FFF);
    pub const MBC1_RAM_BANK_SELECTOR_RANGE: AddressRangeInclusive = Self::MBC1_RAM_BANK_SELECTOR_START..=Self::MBC1_RAM_BANK_SELECTOR_END;
    pub const MBC1_ROM_BANK_MODE_START: Address = Address(0x6000);
    pub const MBC1_ROM_BANK_MODE_END: Address = Address(0x7FFF);
    pub const MBC1_ROM_BANK_MODE_RANGE: AddressRangeInclusive = Self::MBC1_ROM_BANK_MODE_START..=Self::MBC1_ROM_BANK_MODE_END;
}

impl Mbc1 {
    #[inline]
    /// Return the index of addressed RAM bank. Idx ∈ [0, 3]
    pub fn ram_bank(&self) -> u8 {
        self.ram_bank & Mbc1Mask::RAM_BANK
    }

    #[inline]
    /// Return the $X0 ROM bank index based on Bank Mode. On advanced mode ram bank register is used to get the rom bank index.
    ///
    /// Idx ∈ {0x0, 0x20, 0x40, 0x60}
    pub fn rom_bank_high(&self) -> u8 {
        match self.banking_mode {
            Mbc1BankMode::Simple => 0,
            Mbc1BankMode::Advanced => {
                self.ram_bank() << 5
            }
        }
    }

    /// Return the $XX ROM bank index based on Bank Mode.
    /// If the lower 5 bits are equal to 0 the selected bank is the following one (e.g. rom bank value of 0x20 selects rom bank 0x21).
    ///
    /// Idx ∈ [0, 0x7F]
    pub fn rom_bank(&self) -> u8 {
        let base_bank = self.rom_bank & Mbc1Mask::ROM_BANK;
        match self.banking_mode {
            Mbc1BankMode::Simple => base_bank,
            Mbc1BankMode::Advanced => {
                let high_bank = self.ram_bank << 5;
                let adj = if (base_bank & 0b1_1111) == 0 { 1 } else { 0 };
                (base_bank | high_bank) + adj
            }
        }
    }
}

impl BusDevice for Mbc1 {
    fn read(&self, address: Address) -> Byte {
        match address {
            address if Self::MBC1_ROM_BANK_0_RANGE.contains(&address) => {
                match self.banking_mode {
                    Mbc1BankMode::Simple => {
                        self.rom[address.as_usize()]
                    }
                    Mbc1BankMode::Advanced => {
                        let base_offset = 0x20 * Self::MBC1_ROM_BANK_SIZE * self.ram_bank as usize;
                        self.rom[base_offset + address.as_usize()]
                    }
                }
            }
            address if Self::MBC1_ROM_BANK_1_RANGE.contains(&address) => {
                match self.banking_mode {
                    Mbc1BankMode::Simple => {
                        let rom_bank = if (self.rom_bank & Mbc1Mask::ROM_BANK_MODE) == 0 { 0 } else { self.rom_bank - 1 };
                        self.rom[address.as_usize()]
                    }
                    Mbc1BankMode::Advanced => {
                        let base_offset_high = 0x20 * Self::MBC1_ROM_BANK_SIZE * self.ram_bank as usize;
                        let base_offset_low = Self::MBC1_ROM_BANK_SIZE * self.rom_bank as usize;
                        self.rom[base_offset_high + base_offset_low + address.as_usize()]
                    }
                }
            }
            _ => unreachable!()
        }
    }

    fn write(&mut self, address: Address, data: Byte) {
        match address {
            address if Self::MBC1_RAM_ENABLE_RANGE.contains(&address) => {
                self.ram_enabled = if (data & Mbc1Mask::ENABLE_RAM) == Self::MBC1_RAM_ENABLE_VALUE { true } else { false };
            }
            address if Self::MBC1_ROM_BANK_SELECTOR_RANGE.contains(&address) => {
                self.rom_bank = data & Mbc1Mask::ROM_BANK;
            }
            address if Self::MBC1_RAM_BANK_SELECTOR_RANGE.contains(&address) => {
                self.ram_bank = data & Mbc1Mask::RAM_BANK;
            }
            address if Self::MBC1_ROM_BANK_MODE_RANGE.contains(&address) => {
                self.banking_mode = if (data & Mbc1Mask::ROM_BANK_MODE) != 0 { Mbc1BankMode::Advanced } else { Mbc1BankMode::Simple };
            }
            _ => unreachable!(),
        }
    }
}

impl RomController for Mbc1 {
    fn load(&mut self, rom_path: &str) -> Result<(), std::io::Error> {
        let open_result = File::open(rom_path);
        match open_result {
            Ok(f) => {
                Ok(())
            }
            Err(err) => Result::Err(err),
        }
    }

    fn header_slice(&self) -> &[u8; 0x50] {
        self.header.raw_header()
    }

    fn header(&self) -> &RomHeader {
        &self.header
    }
}

impl RomLoader for Mbc1 {
    fn new(mut file: File) -> Result<Self, std::io::Error> {
        file.rewind()?;
        let mut rom: Vec<Byte> = Vec::new();
        file.read_to_end(&mut rom)?;
        let header = RomHeader::new(&rom[RomHeader::HEADER_START_ADDRESS..=RomHeader::HEADER_END_ADDRESS]);

        Ok(Self {
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            rom,
            ram: vec![0; Self::MBC1_RAM_BANK_SIZE * header.rom_banks()],
            banking_mode: Mbc1BankMode::Simple,
            header,
        })
    }
}
