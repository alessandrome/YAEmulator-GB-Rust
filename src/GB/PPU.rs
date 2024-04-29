use crate::mask_flag_enum_default_impl;
use crate::GB::memory::registers::LCDC;
use crate::GB::memory::{
    UseMemory, RAM, VRAM_BLOCK_0_ADDRESS, VRAM_BLOCK_1_ADDRESS, VRAM_BLOCK_2_ADDRESS,
};
use crate::GB::PPU::tile::{GbPaletteId, Tile, TILE_SIZE};
use lcd_stats_masks::LCDStatMasks;
use lcdc_masks::LCDCMasks;
use ppu_mode::PPUMode;
use std::cell::RefCell;
use std::rc::Rc;
use crate::GB::memory::addresses::OAM_AREA_ADDRESS;
use crate::GB::PPU::constants::SCAN_OAM_DOTS;
use crate::GB::PPU::oam::{OAM, OAM_SIZE};

pub mod addresses;
pub mod constants;
pub mod lcd_stats_masks;
pub mod lcdc_masks;
pub mod ppu_mode;
#[cfg(test)]
mod tests;
pub mod tile;
pub mod oam;

macro_rules! ppu_get_set_flag_bit {
    ($get_func: ident, $set_func: ident, $register_ident: ident, $mask_ident: expr) => {
        pub fn $get_func(&self) -> bool {
            (self.read_memory($register_ident as u16) & $mask_ident) != 0
        }
        pub fn $set_func(&mut self, flag: bool) {
            let flag_byte = self.read_memory($register_ident as u16);
            let base_mask = !$mask_ident as u8;
            let bit_num = base_mask.trailing_ones();
            self.write_memory(
                $register_ident as u16,
                flag_byte & base_mask | ((flag as u8) << bit_num),
            );
        }
    };
}

pub struct PPU {
    memory: Rc<RefCell<RAM>>,
    frame: Box<[GbPaletteId; constants::SCREEN_PIXELS]>,
    // mode: PPUMode, -> The mod is mapped in STAT - LCDC Register (bits 1/0)
    line_dots: usize,
    dots_penalties: usize,
    dots_penalties_counter: usize,
}

impl PPU {
    pub fn new(memory: Rc<RefCell<RAM>>) -> Self {
        Self {
            memory,
            frame: Box::new([GbPaletteId::Id0; constants::SCREEN_PIXELS]),
            // mode: PPUMode::OAMScan,
            line_dots: 0,
            dots_penalties: 0,
            dots_penalties_counter: 0,
        }
    }

    pub fn cycle(&mut self) {
        const SCAN_OAM_DOTS_END: usize = constants::SCAN_OAM_DOTS - 1;
        const DRAW_DOTS_END: usize = constants::DRAW_LINE_MAX_DOTS - 1 + constants::SCAN_OAM_DOTS;
        const HBLANK_DOTS_START: usize = DRAW_DOTS_END + 1;
        const HBLANK_DOTS_END: usize = HBLANK_DOTS_START + constants::HBLANK_MIN_DOTS - 1;

        // Get line, check if we are counting penalties, increment line DOT (and line if needed)
        let mut line = self.read_memory(addresses::LY_ADDRESS as u16) as usize;

        // Execute
        if line > constants::SCREEN_HEIGHT - 2 {
            self.set_mode(PPUMode::VBlank);
        } else {
            let scx = self.read_memory(addresses::SCX_ADDRESS as u16) as usize;
            let scy = self.read_memory(addresses::SCY_ADDRESS as u16) as usize;
            if self.line_dots == SCAN_OAM_DOTS {
                // Just entered in draw mode
                self.dots_penalties += scx % 8;
            }

            match self.line_dots {
                0..=SCAN_OAM_DOTS_END => {

                }
                constants::SCAN_OAM_DOTS..=DRAW_DOTS_END => {

                }
                _ => {

                }
            }
        }

        // Update
        if self.dots_penalties_counter > 0 {
            self.dots_penalties_counter -= 1;
        } else {
            self.line_dots = (self.line_dots + 1) % constants::LINE_DOTS;
            match self.line_dots {
                0 => {
                    self.set_mode(PPUMode::OAMScan);
                    line += 1;
                    line %= constants::FRAME_LINES;
                    self.write_memory(addresses::LY_ADDRESS as u16, line as u8);
                }
                constants::SCAN_OAM_DOTS => {
                    self.set_mode(PPUMode::Drawing);
                }
                HBLANK_DOTS_START => {
                    self.line_dots += self.dots_penalties;
                    self.set_mode(PPUMode::HBlank);
                    self.dots_penalties = 0;
                }
                _ => {}
            }
        }
    }

    fn set_mode(&mut self, mode: PPUMode) {
        const LCD_STAT_ADDR_USIZE: u16 = addresses::LCD_STAT_ADDRESS as u16;
        let register = self.read_memory(LCD_STAT_ADDR_USIZE) & !LCDStatMasks::PPUMode;
        self.write_memory(LCD_STAT_ADDR_USIZE, register | mode);
    }

    pub fn get_tile(&self, mut tile_id: u16, bg_win: bool) -> Tile {
        let mut data: [u8; TILE_SIZE] = [0; TILE_SIZE];
        let lcdc = self.read_memory(LCDC);
        let mut start_address = VRAM_BLOCK_0_ADDRESS;
        if bg_win {
            let bg_wind_tile = (lcdc & LCDCMasks::BgWinTilesArea) == 0;
            if bg_wind_tile {
                start_address = if tile_id > 127 {
                    VRAM_BLOCK_1_ADDRESS
                } else {
                    VRAM_BLOCK_2_ADDRESS
                };
                tile_id %= 128;
            }
        }
        start_address += tile_id as usize * TILE_SIZE;
        for i in 0..TILE_SIZE {
            data[i] = self.read_memory((start_address + i) as u16);
        }
        Tile::new(data)
    }

    pub fn get_bg_map(&self) -> Vec<Tile> {
        let mut tiles = Vec::with_capacity(1024);
        for i in 0..256 {
            tiles.push(self.get_tile(i, true));
        }
        tiles
    }

    pub fn get_oam(&self, id: usize) -> OAM {
        let address = (id * OAM_SIZE + OAM_AREA_ADDRESS) as u16;
        let (y, x, tile_id, attributes) =
            (self.read_memory(address),
             self.read_memory(address + 1),
             self.read_memory(address + 2),
             self.read_memory(address + 3));
        OAM::new(y, x, tile_id, attributes, Option::from(id))
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
