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
use crate::GB::memory;
use crate::GB::memory::addresses::OAM_AREA_ADDRESS;
use crate::GB::memory::interrupts::InterruptFlagsMask;
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
        let mut line = self.get_line() as usize;

        // Execute
        if line > constants::SCREEN_HEIGHT - 1 {
            if line == constants::SCREEN_HEIGHT && self.line_dots == 0 {
                self.set_mode(PPUMode::VBlank);
                let old_if = self.memory.borrow().read(memory::registers::IF);
                self.memory.borrow_mut().write(memory::registers::IF, old_if | InterruptFlagsMask::VBlank);
            }
        } else {
            let scx = self.read_memory(addresses::SCX_ADDRESS as u16) as usize;
            let scy = self.read_memory(addresses::SCY_ADDRESS as u16) as usize;
            if self.line_dots == SCAN_OAM_DOTS {
                // Just entered in draw mode
                self.dots_penalties += scx % 8;
            }

            match self.line_dots {
                // Read OAM data to retrieve line sprites
                0..=SCAN_OAM_DOTS_END => {
                    // You can scan a maximum of 40 OAMs and a maximum of 10 OAMs per line
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
                // Update pixels of the frame
                constants::SCAN_OAM_DOTS..=DRAW_DOTS_END => {
                    if self.screen_dot < SCREEN_WIDTH {
                        let screen_pixel_index = self.screen_dot + line * SCREEN_WIDTH;
                        let is_bg_enabled = self.is_bg_win_enabled();
                        let is_sprite_enabled = self.is_obj_enabled();
                        let mut pixel_set = false;
                        if is_sprite_enabled && self.line_oam_number < self.line_oam.len() {
                            let oam = &self.line_oam[self.line_oam_number];
                            let obj_dot = self.screen_dot as isize - oam.get_x_screen();
                            if obj_dot >= 0 && obj_dot < TILE_WIDTH as isize {
                                let obj_line = line as isize - oam.get_y_screen();
                                if obj_line >= 0 && obj_line < TILE_HEIGHT as isize {
                                    let tile = self.get_tile(oam.get_tile_id(), false);
                                    let tile_pixel_index = obj_dot + obj_line * TILE_WIDTH as isize;
                                    let pixel = tile.get_tile_map()[tile_pixel_index as usize].clone();
                                    if pixel != GbPaletteId::Id0 {
                                        self.frame[screen_pixel_index] = pixel;
                                        pixel_set = true;
                                    }
                                }
                            }
                        } else if is_bg_enabled && !pixel_set {
                            let tile = self.get_tile(
                                self.get_bg_chr(self.get_bg_chr_id(self.screen_dot as u8, line as u8)),
                                true);
                            let x_tile = self.get_bg_x() as usize % TILE_WIDTH;
                            let y_tile = self.get_bg_y() as usize % TILE_HEIGHT;
                            self.frame[screen_pixel_index] = tile.get_tile_map()[x_tile + y_tile * TILE_WIDTH].clone();
                        } else {
                            self.frame[screen_pixel_index] = GbPaletteId::Id0;
                        }
                        self.screen_dot += 1;
                    }
                }
                // During HBlank PPU is doing nothing
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

    /// Get true if BG/Window should be drawn
    pub fn is_bg_win_enabled(&self) -> bool {
        let lcdc = self.read_memory(addresses::LCDC_ADDRESS as u16);
        (lcdc & LCDCMasks::BgWinEnabled) != 0
    }

    /// Get true if OBJs should be drawn
    pub fn is_obj_enabled(&self) -> bool {
        let lcdc = self.read_memory(addresses::LCDC_ADDRESS as u16);
        (lcdc & LCDCMasks::ObjEnabled) != 0
    }

    pub fn get_bg(&self) -> Vec<Tile> {
        let mut tiles = Vec::with_capacity(1024);
        for i in 0..constants::MAP_TILES {
            tiles.push(self.get_tile(self.get_bg_chr(i), true));
        }
        tiles
    }

    pub fn get_line(&self) -> u8 {
        self.read_memory(addresses::LY_ADDRESS as u16)
    }

    pub fn get_scy(&self) -> u8 {
        self.read_memory(addresses::SCY_ADDRESS as u16)
    }

    pub fn get_scx(&self) -> u8 {
        self.read_memory(addresses::SCX_ADDRESS as u16)
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

    pub fn get_bg_x(&self) -> u8 {
        ((self.get_scx() as usize + self.screen_dot) % constants::MAP_ROW_PIXELS) as u8
    }

    pub fn get_bg_y(&self) -> u8 {
        ((self.get_scy() as usize + self.get_line() as usize) % constants::MAP_HEIGHT_PIXELS) as u8
    }

    /// In DMG CHR represent ID of the in-memory tile.
    pub fn get_bg_chr(&self, id: usize) -> u8 {
        self.read_memory((addresses::BG_DATA_1_ADDRESS + id) as u16)
    }

    pub fn get_bg_chr_id(&self, x: u8, y: u8) -> usize {
        let scy = self.get_scy() as usize;
        let scx = self.get_scx() as usize;
        let y = (scy + y as usize) % constants::MAP_HEIGHT_PIXELS;
        let x = (scx + x as usize) % constants::MAP_ROW_PIXELS;
        let map_row = y / TILE_HEIGHT;
        let map_column = x / TILE_WIDTH;
        map_column + map_row * constants::MAP_ROW_TILES
    }

    pub fn get_frame_string(&self, doubled: bool) -> String {
        let mut s = "".to_string();
        for i in 0..constants::SCREEN_HEIGHT {
            for j in 0..constants::SCREEN_WIDTH {
                let frame_char = tile::PALETTE_ID_REPR[&self.frame[j + i * constants::SCREEN_WIDTH]];
                s.push_str(frame_char);
                if doubled {
                    s.push_str(frame_char);
                }
            }
            s.push('\n')
        }
        s
    }

    /// String/Draw map of tiles in VRAM. Can be useful for debug.
    pub fn get_tile_map(&self, bank: u8) -> String {
        let bank = bank % 2;
        // TODO: we should use Tile Map bank as GB switch between 2 different VRAM banks
        let mut ret_s = "".to_string();
        let tile_per_row: u8 = 16;
        let tile_rows: u8 = 16;
        for i in 0..tile_rows {
            let mut row_tiles: Vec<String> = vec!["".to_string(); TILE_HEIGHT];
            for j in 0..tile_per_row {
                let tile = self.get_tile(i * 16 + j, false).get_printable_id_map(true);
                let tile_lines: Vec<&str> = tile.split('\n').collect();
                for line in 0..tile_lines.len()-1 {
                    row_tiles[line].push_str(tile_lines[line]);
                }
            }
            ret_s.push_str(&row_tiles.join("\n"));
            ret_s.push('\n');
        }
        ret_s
    }

    /// String/Draw map of OAM tiles in VRAM. OAM item contain ID of its tile and other useful data. This function can be useful for debug.
    pub fn get_oam_tile_map(&self, oam_bank: u8, tile_bank: u8) -> String {
        let tile_bank = tile_bank % 2;
        let oam_bank = oam_bank % 2;
        // TODO: we should use Tile Map bank as GB switch between 2 different VRAM banks
        let mut ret_s = "".to_string();
        let tile_per_row: u8 = 10;
        let tile_rows: u8 = 4;
        for i in 0..tile_rows {
            let mut row_tiles: Vec<String> = vec!["".to_string(); TILE_HEIGHT];
            for j in 0..tile_per_row {
                let oam = self.get_oam((i * tile_per_row + j) as usize);
                let tile = self.get_tile(oam.get_tile_id(), false).get_printable_id_map(true);
                let tile_lines: Vec<&str> = tile.split('\n').collect();
                for line in 0..tile_lines.len()-1 {
                    row_tiles[line].push_str(tile_lines[line]);
                }
            }
            ret_s.push_str(&row_tiles.join("\n"));
            ret_s.push('\n');
        }
        ret_s
    }

    pub fn get_bg_map(&self) -> String {
        let tiles = self.get_bg();
        let mut ret_s = "".to_string();
        for i in 0..constants::MAP_LINES {
            let mut row: Vec<String> = vec!["".to_string(); TILE_HEIGHT];
            for j in 0..constants::MAP_ROW_TILES {
                let tile = tiles[i * constants::MAP_ROW_TILES + j].get_printable_id_map(true);
                let tile_lines: Vec<&str> = tile.split('\n').collect();
                for line in 0..tile_lines.len()-1 {
                    row[line].push_str(tile_lines[line]);
                }
            }
            ret_s.push_str(&row.join("\n"));
            ret_s.push('\n');
        }
        ret_s
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
