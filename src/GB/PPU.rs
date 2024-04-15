use std::cell::RefCell;
use std::rc::Rc;
use crate::GB::memory::{RAM, UseMemory, VRAM_BLOCK_0_ADDRESS, VRAM_BLOCK_1_ADDRESS, VRAM_BLOCK_2_ADDRESS};
use crate::GB::memory::registers::LCDC;
use crate::GB::PPU::tile::{Tile, TILE_SIZE};

pub mod tile;

#[repr(u8)]
pub enum LCDCMasks {
    BgWinTiles = 0b0001_0000
}

pub struct PPU {
    memory: Rc<RefCell<RAM>>
}

impl PPU {
    pub fn new(memory: Rc<RefCell<RAM>>) -> Self {
        Self {
            memory,
        }
    }

    pub fn get_tile(&self, mut tile_id: u16) -> Tile {
        let mut data: [u8; TILE_SIZE] = [0; TILE_SIZE];
        let lcdc = self.read_memory(LCDC);
        let bg_wind_tile = (lcdc & LCDCMasks::BgWinTiles.into()) == 0;
        let mut start_address = VRAM_BLOCK_0_ADDRESS;
        if bg_wind_tile {
            start_address = if tile_id > 127 {VRAM_BLOCK_1_ADDRESS} else {VRAM_BLOCK_2_ADDRESS};
            tile_id %= 128;
        }
        start_address += tile_id as usize * TILE_SIZE;
        for i in 0..TILE_SIZE {
            data[i] = self.read_memory((start_address + i) as u16);
        }
        Tile::new(data)
    }
}

impl UseMemory for PPU {
    fn read_memory(&self, address: u16) -> u8 {
        self.memory.borrow().read(address)
    }

    fn write_memory(&self, address: u16, data: u8) {
        self.memory.borrow_mut().write(address, data)
    }
}
