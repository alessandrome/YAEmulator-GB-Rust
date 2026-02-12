use crate::GB::types::address::{Address, AddressRangeInclusive};

pub const NR10: Address = Address(0xFF10);
pub const NR11: Address = Address(0xFF11);
pub const NR12: Address = Address(0xFF12);
pub const NR13: Address = Address(0xFF13);
pub const NR14: Address = Address(0xFF14);
pub const NR20: Address = Address(0xFF15);  // Not used
pub const NR21: Address = Address(0xFF16);
pub const NR22: Address = Address(0xFF17);
pub const NR23: Address = Address(0xFF18);
pub const NR24: Address = Address(0xFF19);
pub const NR30: Address = Address(0xFF1A);
pub const NR31: Address = Address(0xFF1B);
pub const NR32: Address = Address(0xFF1C);
pub const NR33: Address = Address(0xFF1D);
pub const NR34: Address = Address(0xFF1E);
pub const NR40: Address = Address(0xFF1F);  // Not used
pub const NR41: Address = Address(0xFF20);
pub const NR42: Address = Address(0xFF21);
pub const NR43: Address = Address(0xFF22);
pub const NR44: Address = Address(0xFF23);
pub const NR50: Address = Address(0xFF24);
pub const NR51: Address = Address(0xFF25);
pub const NR52: Address = Address(0xFF26);
pub const AUDIO_RANGE: AddressRangeInclusive = NR10..=NR52;
pub const WAVE_RAM_START: Address = Address(0xff30);
pub const WAVE_RAM_END: Address = Address(0xFF39);
pub const WAVE_RAM_RANGE: AddressRangeInclusive = WAVE_RAM_START..=WAVE_RAM_END;