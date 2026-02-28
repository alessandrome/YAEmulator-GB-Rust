use crate::mask_flag_enum_default_impl;
use crate::GB::memory::registers::LCDC;
use crate::GB::memory::{
    UseMemory, RAM, VRAM_BLOCK_0_ADDRESS, VRAM_BLOCK_1_ADDRESS, VRAM_BLOCK_2_ADDRESS,
};
use crate::GB::ppu::tile::{GbPaletteId, Tile, TILE_SIZE, TILE_HEIGHT, TILE_WIDTH};
use lcd_stat::LCDStatMasks;
use lcd_control::LCDCMasks;
use ppu_mode::PpuMode;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;
use crate::GB::bus::{Bus, BusDevice, MmioContext};
use crate::GB::memory;
use crate::GB::memory::oam_memory::OamMemory;
use crate::GB::ppu::constants::{SCAN_OAM_DOTS, SCREEN_WIDTH};
use crate::GB::ppu::lcd_control::ObjSize;
use crate::GB::ppu::ppu_mmio::PpuMmio;
use crate::GB::ppu::oam::{OAM};
use crate::GB::traits::Tick;
use crate::GB::types::address::Address;
use crate::GB::types::Byte;
use lcd::LCD;
use crate::GB::ppu::pixel_fetcher::PixelFetcher;

pub mod lcd_stat;
pub mod lcd_control;
pub mod ppu_mode;
#[cfg(test)]
mod tests;
pub mod tile;
pub mod oam;
pub mod ppu_mmio;
pub mod pixel;
pub mod lcd;
pub mod palette;
pub mod pixel_fetcher;

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

const SCREEN_LINES: u16 = 144;
const SCREEN_COLUMNS: u16 = 160;
const OAM_BUFFER: u8 = 10;

pub struct PPU {
    frame: Box<[GbPaletteId; SCREEN_LINES as usize * SCREEN_COLUMNS as usize]>,
    pixel_fetcher: PixelFetcher,
    oam_loading: Vec<Byte>,
    oam_scans: u8,
    dot: u16,
    dots_penalties: u8,
    dots_penalties_counter: u8,
    screen_dot: u8,
    switch_mode: bool,
}

impl PPU {
    pub const SCAN_LINES: u16 = 154;
    pub const SCREEN_LINES: u16 = SCREEN_LINES;
    pub const COLUMN_DOTS: u16 = 456;
    pub const SCREEN_COLUMNS: u16 = SCREEN_COLUMNS;
    pub const OAM_SCAN_DOTS: u16 = 80;
    pub const DOTS_PER_FRAME: u32 = (Self::SCAN_LINES as u32) * (Self::COLUMN_DOTS as u32);
    pub const SCREEN_PIXELS: u32 = (Self::SCREEN_LINES as u32) * (Self::SCREEN_COLUMNS as u32);
    pub const OAM_BUFFER: u8 = OAM_BUFFER;

    pub fn new() -> Self {
        Self {
            frame: Box::new([GbPaletteId::Id0; Self::SCREEN_PIXELS as usize]),
            pixel_fetcher: PixelFetcher::new(),
            oam_loading: Vec::with_capacity(OAM::OAM_BYTES as usize),
            oam_scans: 0,
            dot: 0,
            dots_penalties: 0,
            dots_penalties_counter: 0,
            screen_dot: 0,
            switch_mode: false,
        }
    }

    /// Execute a cycle of PPU. Each cycle is the equivalent of 1 Dot.
    ///
    /// Drawing penalties are emulated doing nothing during them. Theme are then added to HBlank mode to reduce its available dots.
    pub fn _tick(&mut self) {
        const SCAN_OAM_DOTS_END: usize = constants::SCAN_OAM_DOTS - 1;
        const DRAW_DOTS_END: usize = constants::DRAW_LINE_MAX_DOTS - 1 + constants::SCAN_OAM_DOTS;
        const HBLANK_DOTS_START: usize = DRAW_DOTS_END + 1;
        const HBLANK_DOTS_END: usize = HBLANK_DOTS_START + constants::HBLANK_MIN_DOTS - 1;

        // Get line, check if we are counting penalties, increment line DOT (and line if needed)
        let mut line = self.get_line() as usize;

        // Execute
        if line > constants::SCREEN_HEIGHT - 1 {
            if line == constants::SCREEN_HEIGHT && self.line_dots == 0 {
                self.set_mode(PpuMode::VBlank);
                let mut old_if = self.memory.borrow().read(memory::registers::IF) | InterruptFlagsMask::VBlank;
                // Check if VBlank Interrupt mode is enabled on STAT register
                let stat_reg = self.memory.borrow().read(memory::registers::STAT);
                if (stat_reg & LCDStatMasks::Mode1Interrupt) != 0 {
                    old_if = old_if | InterruptFlagsMask::LCD;
                }
                self.memory.borrow_mut().write(memory::registers::IF, old_if);
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
                                    let pixel = tile.tile_map()[tile_pixel_index as usize].clone();
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
                            self.frame[screen_pixel_index] = tile.tile_map()[x_tile + y_tile * TILE_WIDTH].clone();
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
                    self.set_mode(PpuMode::OAMScan);
                    // Check if HBlank Interrupt mode is enabled on STAT register
                    let stat_reg = self.memory.borrow().read(memory::registers::STAT);
                    if (stat_reg & LCDStatMasks::Mode2Interrupt) != 0 {
                        let old_if = self.memory.borrow().read(memory::registers::IF);
                        self.memory.borrow_mut().write(memory::registers::IF, old_if | InterruptFlagsMask::LCD);
                    }
                    line += 1;
                    line %= constants::FRAME_LINES;
                    self.write_memory(addresses::LY_ADDRESS as u16, line as u8);
                }
                constants::SCAN_OAM_DOTS => {
                    self.set_mode(PpuMode::Drawing);
                    self.line_oam.sort();
                }
                HBLANK_DOTS_START => {
                    self.line_dots += self.dots_penalties;
                    self.set_mode(PpuMode::HBlank);
                    // Check if HBlank Interrupt mode is enabled on STAT register
                    let stat_reg = self.memory.borrow().read(memory::registers::STAT);
                    if (stat_reg & LCDStatMasks::Mode0Interrupt) != 0 {
                        let old_if = self.memory.borrow().read(memory::registers::IF);
                        self.memory.borrow_mut().write(memory::registers::IF, old_if | InterruptFlagsMask::LCD);
                    }
                    self.dots_penalties = 0;
                    self.line_oam.clear();
                    self.line_oam_number = 0;
                    self.screen_dot = 0;
                }
                _ => {}
            }
        }

        // Update STAT register LY == LYC bit and
        let stat_reg = self.memory.borrow().read(memory::registers::STAT);
        if line as u8 == self.get_line_compare() {
            self.memory.borrow_mut().write(memory::registers::STAT, stat_reg | LCDStatMasks::LYCeLY);
            if (stat_reg & LCDStatMasks::LYCInterrupt) != 0 {
                let old_if = self.memory.borrow().read(memory::registers::IF);
                self.memory.borrow_mut().write(memory::registers::IF, old_if | InterruptFlagsMask::LCD);
            }
        } else {
            self.memory.borrow_mut().write(memory::registers::STAT, stat_reg & !LCDStatMasks::LYCeLY);
        }
    }
}

impl Tick for PPU {
    fn tick(&mut self, bus: &mut Bus, ctx: &mut MmioContext) {
        let stat_view = ctx.ppu_mmio.stat_view();
        let lcdc_view = ctx.ppu_mmio.lcdc_view();

        // If previous mode is ended and next mode is requested, switch to next ppu mode
        if self.switch_mode {
            ctx.ppu_mmio.next_mode();
            self.switch_mode = false;
        }

        if ctx.ppu_mmio.prev_ppu_mode() != ctx.ppu_mmio.ppu_mode() {
            match ctx.ppu_mmio.ppu_mode() {
                PpuMode::OAMScan => {
                    self.oam_scans = 0;
                    self.oam_loading.clear();
                    self.dot = 0;
                    self.dots_penalties = 0;
                    self.dots_penalties_counter = 0;
                    self.screen_dot = 0;
                }
                PpuMode::Drawing => {}
                PpuMode::HBlank => {}
                PpuMode::VBlank => {}
            }
        }

        match ctx.ppu_mmio.ppu_mode() {
            PpuMode::OAMScan => {
                // Mode 2 - OAM Scan
                if self.pixel_fetcher.oam_buffer().len() < Self::OAM_BUFFER as usize {
                    let oam_id = self.oam_scans * 2 / OAM::OAM_BYTES;
                    let oam_base_addr = OamMemory::OAM_START_ADDRESS + (oam_id * 4) as u16;
                    let oam_byte_idx0 = ((self.oam_scans * 2) % OAM::OAM_BYTES);
                    let oam_byte_idx1 = oam_byte_idx0 + 1;

                    // Get OAM Byte 0/1 with oam_scans even, OAM Byte 2/3 if odd
                    self.oam_loading[oam_byte_idx0 as usize] = ctx.oam_mmio.read(oam_base_addr + oam_byte_idx0 as u16);
                    self.oam_loading[oam_byte_idx1 as usize] = ctx.oam_mmio.read(oam_base_addr + oam_byte_idx1 as u16);

                    // 4 Bytes = 1 OAM - Push it to OAM Buffer
                    if !(self.oam_loading.len() < OAM::OAM_BYTES as usize) {
                        let oam = OAM::new(
                            self.oam_loading[0],
                            self.oam_loading[1],
                            self.oam_loading[2],
                            self.oam_loading[3],
                            Some(oam_id)
                        );

                        let obj_height = lcdc_view.obj_size as u8;
                        let ly = ctx.ppu_mmio.ly();
                        if (ly >= oam.y()) && (ly < (oam.y() + obj_height)) {
                            self.pixel_fetcher.push_oam_buffer(oam);
                        }
                    }
                }
                self.oam_scans = (self.oam_scans + 1) & Self::OAM_BUFFER;
            }
            PpuMode::Drawing => {
                // Mode 3 - Drawing Pixels
                self.pixel_fetcher.tick(bus, ctx);

                // Penalities
                if self.dots_penalties_counter < self.dots_penalties {
                    self.dots_penalties_counter += 1;
                } else {
                    self.screen_dot += 1;
                }
            }
            PpuMode::HBlank => {
                // Mode 0 - HBlank
            }
            PpuMode::VBlank => {
                // Mode 1 - VBlank
            }
        }

        // todo!();

        self.dot = (self.dot + 1) % Self::COLUMN_DOTS;
        if self.dot == 0 {
            ctx.ppu_mmio.next_ly();
            if ctx.ppu_mmio.ly() == Self::SCREEN_LINES as u8 || ctx.ppu_mmio.ly() == 0 {
                self.switch_mode = true;
            }
        } else if self.dot == Self::OAM_SCAN_DOTS {
            self.switch_mode = true;
        } else if self.screen_dot >= Self::SCREEN_COLUMNS as u8 {
            self.switch_mode = true;
        }
    }
}

impl Default for PPU {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PPU {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PPU {{ dot: {} }}",
            self.dot,
        )
    }
}

pub struct PpuCtx {
    pub ppu: PPU,
    pub lcd: LCD,
    pub mmio: PpuMmio,
}
