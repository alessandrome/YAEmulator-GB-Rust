pub mod new_license_code;
pub mod old_license_code;

use crate::GB::cartridge::header::new_license_code::NewLicenseCode;
use crate::GB::cartridge::header::old_license_code::OldLicenseCode;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;

const HEADER_NINTENDO_LOGO: [Byte; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E
];

const HEADER_START_ADDRESS: Address = Address(0x0100);
const HEADER_END_ADDRESS: Address = Address(0x014F);

#[derive(Clone, Debug)]
pub struct RomHeader {
    raw_header: [u8; HEADER_END_ADDRESS.as_usize() - HEADER_START_ADDRESS.as_usize()],
    rom_banks: usize,
    ram_banks: usize,
    old_license_code: OldLicenseCode,
    new_license_code: NewLicenseCode,
    rom_version: Byte,
    title: String,
    rom_checksum: u8,
    global_checksum: u16,
}

impl RomHeader {
    pub const HEADER_NINTENDO_LOGO: [Byte; 48] = HEADER_NINTENDO_LOGO;
    pub const HEADER_START_ADDRESS: Address = HEADER_START_ADDRESS;
    pub const HEADER_END_ADDRESS: Address = HEADER_END_ADDRESS;
    pub const HEADER_SIZE: usize = Self::HEADER_END_ADDRESS.as_usize() - HEADER_START_ADDRESS.as_usize();
    pub const HEADER_TITLE_START_ADDRESS: Address = Address(0x0134);
    pub const HEADER_TITLE_END_ADDRESS: Address = Address(0x0143);
    pub const HEADER_NEW_LICENSE_HIGH_BYTE_ADDRESS: Address = Address(0x0144);
    pub const HEADER_NEW_LICENSE_LOW_BYTE_ADDRESS: Address = Address(0x0145);
    pub const HEADER_ROM_SIZE_ADDRESS: Address = Address(0x0148);
    pub const HEADER_RAM_SIZE_ADDRESS: Address = Address(0x0149);
    pub const HEADER_OLD_LICENSE_ADDRESS: Address = Address(0x014B);
    pub const HEADER_ROM_VERSION_ADDRESS: Address = Address(0x014C);
    pub const HEADER_ROM_CHECKSUM_ADDRESS: Address = Address(0x014D);
    pub const HEADER_GLOBAL_CHECKSUM_HIGH_BYTE_ADDRESS: Address = Address(0x014E);
    pub const HEADER_GLOBAL_CHECKSUM_LOW_BYTE_ADDRESS: Address = Address(0x014F);
}

impl RomHeader {
    pub fn new(header_slice: &[Byte; Self::HEADER_SIZE]) -> Self {
        let title_result = String::from_utf8(
            Vec::from(&header_slice[
                (Self::HEADER_TITLE_START_ADDRESS.as_usize() - Self::HEADER_START_ADDRESS.as_usize())..=(Self::HEADER_TITLE_START_ADDRESS.as_usize() - Self::HEADER_END_ADDRESS.as_usize())
                ])
        ).unwrap();

        let old_license_code = header_slice[Self::HEADER_OLD_LICENSE_ADDRESS.as_usize() - Self::HEADER_START_ADDRESS.as_usize()];
        let new_license_high_byte = header_slice[Self::HEADER_NEW_LICENSE_HIGH_BYTE_ADDRESS.as_usize() - Self::HEADER_START_ADDRESS.as_usize()];
        let new_license_low_byte = header_slice[Self::HEADER_NEW_LICENSE_LOW_BYTE_ADDRESS.as_usize() - Self::HEADER_START_ADDRESS.as_usize()];
        let new_license_code = ((new_license_high_byte as u16) << 8) | new_license_low_byte as u16;

        let rom_version = header_slice[Self::HEADER_ROM_VERSION_ADDRESS.as_usize() - Self::HEADER_START_ADDRESS.as_usize()];
        let rom_checksum = header_slice[Self::HEADER_ROM_CHECKSUM_ADDRESS.as_usize() - Self::HEADER_START_ADDRESS.as_usize()];
        let global_checksum_high = header_slice[Self::HEADER_GLOBAL_CHECKSUM_HIGH_BYTE_ADDRESS.as_usize() - Self::HEADER_START_ADDRESS.as_usize()];
        let global_checksum_low = header_slice[Self::HEADER_GLOBAL_CHECKSUM_LOW_BYTE_ADDRESS.as_usize() - Self::HEADER_START_ADDRESS.as_usize()];
        let global_checksum = ((global_checksum_high as u16) << 8) | global_checksum_low as u16;

        Self {
            raw_header: header_slice.clone(),
            rom_banks: Self::rom_banks_from_byte(header_slice[Self::HEADER_ROM_SIZE_ADDRESS.as_usize() - Self::HEADER_START_ADDRESS.as_usize()]),
            ram_banks: Self::ram_banks_from_byte(header_slice[Self::HEADER_RAM_SIZE_ADDRESS.as_usize() - Self::HEADER_START_ADDRESS.as_usize()]),
            old_license_code: old_license_code.into(),
            new_license_code: new_license_code.into(),
            rom_version,
            title: title_result,
            rom_checksum,
            global_checksum,
        }
    }

    #[inline]
    pub fn rom_banks_from_byte(byte: Byte) -> usize {
        match byte {
            0..=8 => 2 << byte,
            0x52..=0x54 => 64 + (2 << (byte & 0x0F)),
            _ => unimplemented!(),
        }
    }

    #[inline]
    pub fn ram_banks_from_byte(byte: Byte) -> usize {
        match byte {
            0 => 0,
            1 => unimplemented!(),
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            _ => unimplemented!(),
        }
    }

    #[inline]
    pub fn raw_header(&self) -> &[Byte; Self::HEADER_SIZE] {
        &self.raw_header
    }

    #[inline]
    pub fn rom_banks(&self) -> usize {
        self.rom_banks
    }

    #[inline]
    pub fn ram_banks(&self) -> usize {
        self.ram_banks
    }

    #[inline]
    pub fn old_license_code(&self) -> OldLicenseCode {
        self.old_license_code
    }

    #[inline]
    pub fn new_license_code(&self) -> NewLicenseCode {
        self.new_license_code
    }

    #[inline]
    pub fn rom_version(&self) -> Byte {
        self.rom_version
    }

    #[inline]
    pub fn rom_checksum(&self) -> u8 {
        self.rom_checksum
    }

    #[inline]
    pub fn global_checksum(&self) -> u16 {
        self.global_checksum
    }

    #[inline]
    pub fn title(&self) -> &String {
        &self.title
    }
}
