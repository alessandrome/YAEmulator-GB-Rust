use crate::GB::bus::BusDevice;
use crate::GB::ppu::tile::{Tile, TileDataArea, TileMapArea};
use super::{Length, Memory};
use crate::GB::types::address::{Address, AddressRangeInclusive};
use crate::GB::types::Byte;

pub struct VRAM {
    #[cfg(test)]
    pub memory: Memory<u8>,
    #[cfg(not(test))]
    memory: Memory<u8>,
}

impl VRAM {
    pub const VRAM_TILE_BLOCK_0_START: Address = Address(0x8000);
    pub const VRAM_TILE_BLOCK_0_END: Address = Address(0x87FF);
    pub const VRAM_TILE_BLOCK_0_RANGE: AddressRangeInclusive = Self::VRAM_TILE_BLOCK_0_START..=Self::VRAM_TILE_BLOCK_0_END;
    pub const VRAM_TILE_BLOCK_1_START: Address = Address(0x8800);
    pub const VRAM_TILE_BLOCK_1_END: Address = Address(0x8FFF);
    pub const VRAM_TILE_BLOCK_1_RANGE: AddressRangeInclusive = Self::VRAM_TILE_BLOCK_1_START..=Self::VRAM_TILE_BLOCK_1_END;
    pub const VRAM_TILE_BLOCK_2_START: Address = Address(0x9000);
    pub const VRAM_TILE_BLOCK_2_END: Address = Address(0x97FF);
    pub const VRAM_TILE_BLOCK_2_RANGE: AddressRangeInclusive = Self::VRAM_TILE_BLOCK_2_START..=Self::VRAM_TILE_BLOCK_2_END;
    pub const VRAM_TILE_MAP_0_START: Address = Address(0x9800);
    pub const VRAM_TILE_MAP_0_END: Address = Address(0x9BFF);
    pub const VRAM_TILE_MAP_0_RANGE: AddressRangeInclusive = Self::VRAM_TILE_MAP_0_START..=Self::VRAM_TILE_MAP_0_END;
    pub const VRAM_TILE_MAP_1_START: Address = Address(0x9C00);
    pub const VRAM_TILE_MAP_1_END: Address = Address(0x9FFF);
    pub const VRAM_TILE_MAP_1_RANGE: AddressRangeInclusive = Self::VRAM_TILE_MAP_1_START..=Self::VRAM_TILE_MAP_1_END;
    pub const VRAM_START_ADDRESS: Address = Self::VRAM_TILE_BLOCK_0_START; // Video memory
    pub const VRAM_END_ADDRESS: Address = Self::VRAM_TILE_MAP_1_END; // Video memory
    pub const VRAM_ADDRESS_RANGE: AddressRangeInclusive = Self::VRAM_START_ADDRESS..=Self::VRAM_END_ADDRESS; // Video memory

    pub fn new() -> Self {
        Self {
            memory: Memory::<u8>::new(0, 0x2000),
        }
    }

    pub fn read_vec(&self, start_address: u16, length: u16) -> &[Byte] {
        &self.memory[start_address as usize..(start_address + length) as usize]
    }

    fn tile_memory_index(id: u8, tile_block: TileDataArea) -> usize {
        let tile_address_summer: u8;
        let base_tile_idx: usize;
        if tile_block == TileDataArea::DataBlock01 {
            tile_address_summer = id;
            base_tile_idx = Self::VRAM_TILE_BLOCK_0_START.as_index() - Self::VRAM_START_ADDRESS.as_index();
        } else {
            tile_address_summer = ((id as u16 + 128) & 0xFF) as u8; // -- AND 0xFF is like do % 256 but faster
            base_tile_idx = Self::VRAM_TILE_BLOCK_1_START.as_index() - Self::VRAM_START_ADDRESS.as_index();
        }
        base_tile_idx + tile_address_summer as usize * Tile::TILE_SIZE as usize
    }

    pub fn tile(&self, id: u8, tile_block: TileDataArea) -> Tile {
        let memory_idx = Self::tile_memory_index(id, tile_block);
        let slice = &self.memory[memory_idx..memory_idx + Tile::TILE_SIZE as usize];
        Tile::from_bytes(<&[Byte; 16]>::try_from(slice).expect("Expecting 16 bytes"))
    }

    #[inline]
    pub fn tile_line_lsb_byte(&self, id: u8, line: u8, tile_block: TileDataArea) -> Byte {
        assert!(line < Tile::TILE_HEIGHT);
        let memory_idx = Self::tile_memory_index(id, tile_block);
        self.memory[memory_idx + line as usize * 2]
    }

    #[inline]
    pub fn tile_line_msb_byte(&self, id: u8, line: u8, tile_block: TileDataArea) -> Byte {
        let memory_idx = Self::tile_memory_index(id, tile_block);
        self.memory[memory_idx + line as usize * 2 + 1]
    }

    #[inline]
    /// Retrieve ID of a tile from the specified Tile Map Are given the ID position in it.
    /// The Id position value is ANDed (&) with 0x3FF value to ensure value between 0-1023
    pub fn tile_id(&self, id: u16, map_area: TileMapArea) -> Byte {
        match map_area {
            TileMapArea::MapBlock0 => {
                let base_index = Self::VRAM_TILE_BLOCK_0_START.as_usize() - Self::VRAM_START_ADDRESS.as_usize();
                self.memory[base_index + (id & 0x3FF) as usize]
            }
            TileMapArea::MapBlock1 => {
                let base_index = Self::VRAM_TILE_BLOCK_1_START.as_usize() - Self::VRAM_START_ADDRESS.as_usize();
                self.memory[base_index + (id & 0x3FF) as usize]
            }
        }
    }
}

impl Length for VRAM {
    fn len(&self) -> usize {
        self.memory.len()
    }
}

impl BusDevice for VRAM {
    fn read(&self, address: Address) -> Byte {
        match address {
            address if Self::VRAM_ADDRESS_RANGE.contains(&address) => {
                self.memory[address.as_index() - Self::VRAM_START_ADDRESS.as_index()]
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn write(&mut self, address: Address, byte: Byte) {
        match address {
            address if Self::VRAM_ADDRESS_RANGE.contains(&address) => {
                self.memory[address.as_index() - Self::VRAM_START_ADDRESS.as_index()] = byte;
            }
            _ => {
                unreachable!();
            }
        }
    }
}
