use crate::GB::ppu::tile::{GbPaletteId};
use ppu_mode::PpuMode;
use std::fmt;
use std::fmt::Formatter;
use crate::GB::bus::{Bus, BusDevice, MmioContext};
use crate::GB::memory::oam_memory::OamMemory;
use crate::GB::ppu::ppu_mmio::PpuMmio;
use crate::GB::ppu::oam::{OAM};
use crate::GB::traits::Tick;
use crate::GB::types::Byte;
use lcd::LCD;

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
pub mod tile_line;

const SCREEN_LINES: u16 = 144;
const SCREEN_COLUMNS: u16 = 160;
const OAM_BUFFER: u8 = 10;

pub struct PPU {
    frame: Box<[GbPaletteId; SCREEN_LINES as usize * SCREEN_COLUMNS as usize]>,
    bg_fetcher: pixel_fetcher::BackgroundFetcher,
    sprite_fetcher: pixel_fetcher::SpriteFetcher,
    oam_loading: Vec<Byte>,
    oam_scans: u8,
    discarding_pixels: u8,
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
            bg_fetcher: pixel_fetcher::BackgroundFetcher::new(),
            sprite_fetcher: pixel_fetcher::SpriteFetcher::new(),
            oam_loading: Vec::with_capacity(OAM::OAM_BYTES as usize),
            oam_scans: 0,
            discarding_pixels: 0,
            dot: 0,
            dots_penalties: 0,
            dots_penalties_counter: 0,
            screen_dot: 0,
            switch_mode: false,
        }
    }
}

impl Tick for PPU {
    fn tick(&mut self, bus: &mut Bus, ctx: &mut MmioContext) {
        let stat_view = ctx.ppu_mmio.stat_view();
        let lcdc_view = ctx.ppu_mmio.lcdc_view();

        // Ticking and update as needed PPU mode in PPU Context
        ctx.ppu_mmio.tick(self.switch_mode);
        self.switch_mode = false;

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
                PpuMode::Drawing => {
                    ctx.ppu_mmio.sort_oam_buffer();
                    self.discarding_pixels = ctx.ppu_mmio.scx() & 7;
                }
                PpuMode::HBlank => {}
                PpuMode::VBlank => {}
            }
        }

        match ctx.ppu_mmio.ppu_mode() {
            PpuMode::OAMScan => {
                // Mode 2 - OAM Scan
                if ctx.ppu_mmio.oam_buffer().len() < Self::OAM_BUFFER as usize {
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
                            ctx.ppu_mmio.push_oam_buffer(oam);
                        }
                    }
                }
                self.oam_scans = (self.oam_scans + 1) & Self::OAM_BUFFER;
            }
            PpuMode::Drawing => {
                // Mode 3 - Drawing Pixels
                let discard_pixels = self.discarding_pixels > 0;
                if discard_pixels {
                    if ctx.ppu_mmio.pop_bg_pixel().is_some() {
                        self.discarding_pixels -= 1;
                    }
                } else {
                    todo!("LCD Tick - Mix & Push pixel to screen (if BG ready!)");
                }

                self.bg_fetcher.tick(bus, ctx);

                // Penalities - Now are already included in Pixel FIFO Behavior
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
