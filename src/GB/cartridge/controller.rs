pub mod mbc1;

pub use mbc1::Mbc1;
use crate::GB::bus::BusDevice;
use crate::GB::cartridge::header::RomHeader;

pub trait RomController: BusDevice {
    fn load(&mut self, rom_path: &str) -> Result<(), std::io::Error>;
    fn header(&self) -> &RomHeader;
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum CartridgeControllerType {
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

impl From<u8> for CartridgeControllerType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::RomOnly,
            0x01 => Self::Mbc1,
            0x02 => Self::Mbc1Ram,
            0x03 => Self::Mbc1RamBattery,
            0x05 => Self::Mbc2,
            0x06 => Self::Mbc2Battery,
            0x08 => Self::RomRam,
            0x09 => Self::RomRamBattery,
            0x0B => Self::Mmm01,
            0x0C => Self::Mmm01Ram,
            0x0D => Self::Mmm01RamBattery,
            0x0F => Self::Mbc3TimerBattery,
            0x10 => Self::Mbc3TimerRamBattery,
            0x11 => Self::Mbc3,
            0x12 => Self::Mbc3Ram,
            0x13 => Self::Mbc3RamBattery,
            0x19 => Self::Mbc5,
            0x1A => Self::Mbc5Ram,
            0x1B => Self::Mbc5RamBattery,
            0x1C => Self::Mbc5Rumble,
            0x1D => Self::Mbc5RumbleRam,
            0x1E => Self::Mbc5RumbleRamBattery,
            0x20 => Self::Mbc6,
            0x22 => Self::Mbc7SensorRumbleRamBattery,
            0xFC => Self::PocketCamera,
            0xFD => Self::BandaiTama5,
            0xFE => Self::HuC3,
            0xFF => Self::HuC1RamBattery,
            _ => Self::Unknown,
        }
    }
}

impl From<CartridgeControllerType> for &str {
    fn from(value: CartridgeControllerType) -> Self {
        match value {
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
        }
    }
}

impl From<CartridgeControllerType> for String {
    fn from(value: CartridgeControllerType) -> Self {
        let s: &str = value.into();
        s.to_string()
    }
}

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum CartridgeController {
    Mbc1(Mbc1),
}
