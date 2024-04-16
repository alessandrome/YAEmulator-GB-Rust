use std::cell::RefCell;
use std::rc::Rc;
use crate::GB::memory::{RAM, UseMemory, VRAM_BLOCK_0_ADDRESS, VRAM_BLOCK_1_ADDRESS, VRAM_BLOCK_2_ADDRESS};
use crate::GB::memory::registers::{LCDC};
use crate::GB::PPU::tile::{Tile, TILE_SIZE};
use crate::mask_flag_enum_default_impl;

pub mod tile;
#[cfg(test)]
mod tests;

macro_rules! ppu_get_set_flag_bit {
    ($get_func: ident, $set_func: ident, $register_ident: ident, $mask_ident: expr) => {
        pub fn $get_func(&self) -> bool {
            (self.read_memory($register_ident as u16) & $mask_ident) != 0
        }
        pub fn $set_func(&mut self, flag: bool) {
            let flag_byte = self.read_memory($register_ident as u16);
            let base_mask = !$mask_ident as u8;
            let bit_num = base_mask.trailing_ones();
            self.write_memory($register_ident as u16, flag_byte & base_mask | ((flag as u8) << bit_num));
        }
    };
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum LCDCMasks {
    BgWinEnabled = 0b0000_0001,
    ObjEnabled = 0b0000_0010,
    ObjSize = 0b0000_0100,
    BgTileMapArea = 0b0000_1000,
    BgWinTilesArea = 0b0001_0000,
    WinEnabled = 0b0010_0000,
    WinTileMapArea = 0b0100_0000,
    LcdEnabled = 0b1000_0000,
}

mask_flag_enum_default_impl!(LCDCMasks);

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
        let bg_wind_tile = (lcdc & LCDCMasks::BgWinTilesArea) == 0;
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

    ppu_get_set_flag_bit!(get_bg_win_enabled_flag, set_bg_win_enabled_flag, LCDC, LCDCMasks::BgWinEnabled);
    ppu_get_set_flag_bit!(get_obj_enabled_flag, set_obj_enabled_flag, LCDC, LCDCMasks::ObjEnabled);
    ppu_get_set_flag_bit!(get_obj_size_flag, set_obj_size_flag, LCDC, LCDCMasks::ObjSize);
    ppu_get_set_flag_bit!(get_bg_tile_map_area_flag, set_bg_tile_map_area_flag, LCDC, LCDCMasks::BgTileMapArea);
    ppu_get_set_flag_bit!(get_bg_win_tiles_area_flag, set_bg_win_tiles_area_flag, LCDC, LCDCMasks::BgWinTilesArea);
    ppu_get_set_flag_bit!(get_win_enabled_flag, set_win_enabled_flag, LCDC, LCDCMasks::WinEnabled);
    ppu_get_set_flag_bit!(get_win_tile_map_area_flag, set_win_tile_map_area_flag, LCDC, LCDCMasks::WinTileMapArea);
    ppu_get_set_flag_bit!(get_lcd_enabled_flag, set_lcd_enabled_flag, LCDC, LCDCMasks::LcdEnabled);
}

impl UseMemory for PPU {
    fn read_memory(&self, address: u16) -> u8 {
        self.memory.borrow().read(address)
    }

    fn write_memory(&self, address: u16, data: u8) {
        self.memory.borrow_mut().write(address, data)
    }
}
