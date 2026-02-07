use crate::GB::types::address::Address;
use crate::GB::types::Byte;

pub trait BusDevice {
    fn read(&self, address: Address) -> Byte;
    fn write(&self, address: Address, data: Byte);
}

pub trait MmioDevice: BusDevice {}
pub trait MemoryDevice: BusDevice {}
