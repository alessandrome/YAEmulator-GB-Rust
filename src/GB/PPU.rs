use crate::mask_flag_enum_default_impl;
use crate::GB::memory::registers::LCDC;
use crate::GB::memory::{
    UseMemory, RAM, VRAM_BLOCK_0_ADDRESS, VRAM_BLOCK_1_ADDRESS, VRAM_BLOCK_2_ADDRESS,
};
use crate::GB::PPU::tile::{GbPaletteId, Tile, TILE_SIZE, TILE_HEIGHT, TILE_WIDTH};
use lcd_stats_masks::LCDStatMasks;
use lcdc_masks::LCDCMasks;
use ppu_mode::PPUMode;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;
use crate::GB::memory::addresses::OAM_AREA_ADDRESS;
use crate::GB::PPU::constants::{SCAN_OAM_DOTS, SCREEN_WIDTH};
use crate::GB::PPU::oam::{OAM, OAM_BYTE_SIZE};

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
    screen_dot: usize,
    dots_penalties: usize,
    dots_penalties_counter: usize,
    line_oam: Vec<OAM>,
    line_oam_number: usize,
}

impl PPU {
    pub fn new(memory: Rc<RefCell<RAM>>) -> Self {
        Self {
            memory,
            frame: Box::new([GbPaletteId::Id0; constants::SCREEN_PIXELS]),
            // mode: PPUMode::OAMScan,
            line_dots: 0,
            screen_dot: 0, // Actual elaborated pixel on screen (between 0 - 143)
            dots_penalties: 0,
            dots_penalties_counter: 0,
            line_oam: Vec::with_capacity(constants::MAX_SPRITE_PER_LINE),
            line_oam_number: 0,
        }
    }

    /// Execute a cycle of PPU. Each cycle is the equivalent of 1 Dot.
    ///
    /// Drawing penalties are emulated doing nothing during them. Theme are then added to HBlank mode to reduce its available dots.
    pub fn cycle(&mut self) {
        const SCAN_OAM_DOTS_END: usize = constants::SCAN_OAM_DOTS - 1;
        const DRAW_DOTS_END: usize = constants::DRAW_LINE_MAX_DOTS - 1 + constants::SCAN_OAM_DOTS;
        const HBLANK_DOTS_START: usize = DRAW_DOTS_END + 1;
        const HBLANK_DOTS_END: usize = HBLANK_DOTS_START + constants::HBLANK_MIN_DOTS - 1;

        // Get line, check if we are counting penalties, increment line DOT (and line if needed)
        let mut line = self.read_memory(addresses::LY_ADDRESS as u16) as usize;

        // Execute
        if line > constants::SCREEN_HEIGHT - 1 {
            self.set_mode(PPUMode::VBlank);
        } else {
            let scx = self.read_memory(addresses::SCX_ADDRESS as u16) as usize;
            let scy = self.read_memory(addresses::SCY_ADDRESS as u16) as usize;
            if self.line_dots == SCAN_OAM_DOTS {
                // Just entered in draw mode
                self.dots_penalties += scx % 8;
            }

            match self.line_dots {
                //! Read OAM data to retrieve line sprites
                0..=SCAN_OAM_DOTS_END => {
                    if self.line_dots < constants::OAM_NUMBERS && self.line_oam.len() < constants::MAX_SPRITE_PER_LINE {
                        let line_isize = line as isize;
                        let tile_mod = self.get_tile_mode();
                        let oam = self.get_oam(self.line_dots);
                        let oam_y_screen = oam.get_y_screen();
                        let tile_height = TILE_HEIGHT * (tile_mod as usize + 1); // If dual tile sprite is enabled sprite has doubled the height
                        if oam_y_screen <= line_isize && (oam_y_screen + tile_height as isize) > line_isize {
                            self.line_oam.push(oam);
                        }
                    }
                }
                //! Update pixels of the frame
                constants::SCAN_OAM_DOTS..=DRAW_DOTS_END => {
                    if self.screen_dot < SCREEN_WIDTH {
                        if self.line_oam_number < self.line_oam.len() {
                            let oam = &self.line_oam[self.line_oam_number];
                            let drawing_dot = (self.line_dots - constants::SCAN_OAM_DOTS - self.dots_penalties) as isize;
                            let obj_dot = self.screen_dot as isize - oam.get_x_screen();
                            if obj_dot >= 0 && obj_dot < TILE_WIDTH as isize {
                                let obj_line = line as isize - oam.get_y_screen();
                                let tile = self.get_tile(oam.get_tile_id(), false);
                                let screen_index = self.screen_dot + line * SCREEN_WIDTH;
                                let tile_index = obj_dot + obj_line * TILE_WIDTH as isize;
                                self.frame[screen_index] = tile.get_tile_map()[tile_index as usize].clone();
                            }
                            let tile = self.get_tile(oam.get_tile_id(), false);
                        }
                        self.screen_dot += 1;
                    }
                }
                //! During HBlank PPU is doing nothing
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
                    self.line_oam.sort();
                }
                HBLANK_DOTS_START => {
                    self.line_dots += self.dots_penalties;
                    self.set_mode(PPUMode::HBlank);
                    self.dots_penalties = 0;
                    self.line_oam.clear();
                    self.line_oam_number = 0;
                    self.screen_dot = 0;
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

    pub fn get_tile(&self, mut tile_id: u8, bg_win: bool) -> Tile {
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

    /// Retrieve tile/obj size mode. Return False if OBJ is a single 8x8 obj or True if a dual tile in 8x16 obj
    pub fn get_tile_mode(&self) -> bool {
        let lcdc = self.read_memory(addresses::LCDC_ADDRESS as u16);
        (lcdc & LCDCMasks::ObjSize) != 0
    }

    /// Retrieve tile/obj size mode. Return False if OBJ is a single 8x8 obj or True if a dual tile in 8x16 obj
    pub fn is_bg_win_enabled(&self) -> bool {
        let lcdc = self.read_memory(addresses::LCDC_ADDRESS as u16);
        (lcdc & LCDCMasks::BgWinEnabled) != 0
    }

    pub fn get_bg_map(&self) -> Vec<Tile> {
        let mut tiles = Vec::with_capacity(1024);
        for i in 0..=255 {
            tiles.push(self.get_tile(i, true));
        }
        tiles
    }

    pub fn get_oam(&self, id: usize) -> OAM {
        let address = (id * OAM_BYTE_SIZE + OAM_AREA_ADDRESS) as u16;
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

impl fmt::Display for PPU {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let line = self.read_memory(addresses::LY_ADDRESS as u16);
        write!(
            f,
            "PPU {{ Y: {}, X: {}, ldot: {} }}",
            line, self.screen_dot, self.line_dots
        )
    }
}
