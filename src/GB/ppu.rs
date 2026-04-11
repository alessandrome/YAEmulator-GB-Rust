use crate::GB::ppu::tile::{GbColor, GbPaletteId};
use ppu_mode::PpuMode;
use std::fmt;
use std::fmt::Formatter;
use crate::GB::bus::{Bus, BusDevice, MmioContextWrite};
use crate::GB::memory::oam_memory::OamMemory;
use crate::GB::ppu::ppu_mmio::PpuMmio;
use crate::GB::ppu::oam::{OAM};
use crate::GB::traits::Tick;
use crate::GB::types::Byte;
use lcd::LCD;
use crate::GB::cpu::registers::interrupt_registers::InterruptFlagsMask;
use crate::GB::ppu::pixel::{PixelFifo, PixelFifoPaletteRegister};
use crate::GB::ppu::pixel_fetcher::{BgFetchingMode, PixelFetcherState};

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

#[derive(Copy, Clone, Debug, PartialEq)]
enum PpuFetchingMode {
    FetchBg,
    FetchSprite,
}

pub struct PPU {
    frame: Box<[GbPaletteId; SCREEN_LINES as usize * SCREEN_COLUMNS as usize]>,
    fetching_mode: PpuFetchingMode,
    bg_fetcher: pixel_fetcher::BackgroundFetcher,
    sprite_fetcher: pixel_fetcher::SpriteFetcher,
    oam_loading: Vec<Byte>,
    oam_scans: u8,
    discarding_pixels: u8,
    dot: u16,
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
            fetching_mode: PpuFetchingMode::FetchBg,
            bg_fetcher: pixel_fetcher::BackgroundFetcher::new(),
            sprite_fetcher: pixel_fetcher::SpriteFetcher::new(),
            oam_loading: Vec::with_capacity(OAM::OAM_BYTES as usize),
            oam_scans: 0,
            discarding_pixels: 0,
            dot: 0,
            screen_dot: 0,
            switch_mode: false,
        }
    }

    pub fn pixel_mixer(obj_pixel: Option<&PixelFifo>, bg_pixel: &PixelFifo) -> PixelFifo {
        match obj_pixel {
            None => bg_pixel.clone(),
            Some(obj_pixel) => {
                if obj_pixel.color_id() == GbPaletteId::Id0 {
                    return bg_pixel.clone();
                }
                if obj_pixel.priority() && bg_pixel.color_id() != GbPaletteId::Id0 {
                    return bg_pixel.clone();
                }
                obj_pixel.clone()
            }
        }
    }
}

impl Tick for PPU {
    fn tick(&mut self, bus: &mut Bus, ctx: &mut MmioContextWrite) {
        let lcdc_view = ctx.ppu_mmio.lcdc_view();
        // Run PPU only if it is enabled
        if lcdc_view.lcd_enabled {
            let stat_view = ctx.ppu_mmio.stat_view();

            // Ticking and update as needed PPU mode in PPU Context
            ctx.ppu_mmio.tick(self.switch_mode);
            self.switch_mode = false;

            // Mode is changing
            if ctx.ppu_mmio.prev_ppu_mode() != ctx.ppu_mmio.ppu_mode() {
                match ctx.ppu_mmio.ppu_mode() {
                    PpuMode::OAMScan => {
                        // Clearing OAM buffer & Pixel FIFOs
                        self.oam_scans = 0;
                        self.oam_loading.clear();
                        self.dot = 0;
                        ctx.ppu_mmio.clear_bg_fifo();
                        ctx.ppu_mmio.clear_obj_fifo();
                        ctx.ppu_mmio.reset_lx();
                        match ctx.ppu_mmio.prev_ppu_mode() {
                            PpuMode::VBlank => {
                                self.bg_fetcher.reset_frame();
                            }
                            _ => {
                                self.bg_fetcher.reset_line();
                            }
                        }
                    }
                    PpuMode::Drawing => {
                        // Prepare OAM Buffer for Pixel FIFOs and check how many pixels discard
                        ctx.ppu_mmio.sort_oam_buffer();
                        self.fetching_mode = PpuFetchingMode::FetchBg;
                        let lcdc = ctx.ppu_mmio.lcdc_view();
                        let wx = ctx.ppu_mmio.wx();
                        let wy = ctx.ppu_mmio.wy();
                        let lx = ctx.ppu_mmio.lx();
                        let ly = ctx.ppu_mmio.ly();
                        if lcdc.bg_win_enabled && wx != 0 && wx.saturating_sub(7) == lx && ly >= wy {
                            self.discarding_pixels = 7 - wx;
                        } else {
                            self.discarding_pixels = ctx.ppu_mmio.scx() & 7;
                        }
                    }
                    PpuMode::HBlank => {}
                    PpuMode::VBlank => {
                        ctx.cpu_mmio.interrupt_registers_mut().set_if_bit(InterruptFlagsMask::VBlank);
                    }
                }
            }

            match ctx.ppu_mmio.ppu_mode() {
                PpuMode::OAMScan => {
                    // Mode 2 - OAM Scan
                    if ctx.ppu_mmio.oam_buffer().len() < Self::OAM_BUFFER as usize {
                        let oam_id = self.oam_scans;
                        let oam_base_addr = OamMemory::OAM_START_ADDRESS + (oam_id * 4) as u16;
                        let oam_byte_idx0 = self.oam_loading.len();
                        let oam_byte_idx1 = oam_byte_idx0 + 1;

                        // Get OAM Byte 0/1 with oam_scans even, OAM Byte 2/3 if odd
                        self.oam_loading.push(ctx.oam_mmio.read(oam_base_addr + oam_byte_idx0 as u16));
                        self.oam_loading.push(ctx.oam_mmio.read(oam_base_addr + oam_byte_idx1 as u16));

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
                            let adj_ly = ctx.ppu_mmio.ly() + 16;
                            if (adj_ly >= oam.y()) && (adj_ly < (oam.y() + obj_height)) {
                                ctx.ppu_mmio.push_oam_buffer(oam);
                            }
                            self.oam_loading.clear();
                            self.oam_scans = self.oam_scans + 1;
                        }
                    }
                }
                PpuMode::Drawing => {
                    // Mode 3 - Drawing Pixels
                    if self.bg_fetcher.fetching_mode() == BgFetchingMode::Bg {
                        let lcdc = ctx.ppu_mmio.lcdc_view();
                        let wx = ctx.ppu_mmio.wx();
                        let wy = ctx.ppu_mmio.wy();
                        let lx = ctx.ppu_mmio.lx();
                        let ly = ctx.ppu_mmio.ly();
                        if lcdc.bg_win_enabled && lcdc.window_enabled && wx.saturating_sub(7) == lx && ly >= wy {
                            self.bg_fetcher.set_window_mode();
                            ctx.ppu_mmio.clear_obj_fifo();
                        }
                    }

                    if self.bg_fetcher.fetching_mode() == BgFetchingMode::Bg {
                        let sprite_oam = ctx.ppu_mmio.oam_buffer().first();
                        match sprite_oam {
                            None => {}
                            Some(oam) => {
                                if self.fetching_mode == PpuFetchingMode::FetchBg && oam.x().saturating_sub(8) == ctx.ppu_mmio.lx() {
                                    // Sprite Pixel fetching starts only if OBJs are enabled
                                    if ctx.ppu_mmio.lcdc_view().obj_enabled {
                                        self.fetching_mode = PpuFetchingMode::FetchSprite;
                                        self.bg_fetcher.reset_cycle();
                                    }
                                }
                            }
                        }
                    }

                    match self.fetching_mode {
                        PpuFetchingMode::FetchBg => {
                            self.bg_fetcher.tick(bus, ctx);
                            // Mix BG & Sprite pixel if BG is ready and set pixel color to stream
                            if !ctx.ppu_mmio.bg_fifo().is_empty() {
                                let discard_pixels = self.discarding_pixels > 0;
                                let obj_pixel = ctx.ppu_mmio.pop_obj_pixel();
                                if discard_pixels {
                                    // Discard pixels on new drawing line as needed
                                    self.discarding_pixels -= 1;
                                } else {
                                    // Streaming pixel for LCD
                                    let mixed_pixel_fifo = Self::pixel_mixer(
                                        obj_pixel.as_ref(),
                                        &ctx.ppu_mmio.pop_bg_pixel().unwrap(),
                                    );
                                    // Get color by palette
                                    let color: GbColor;
                                    match mixed_pixel_fifo.palette() {
                                        PixelFifoPaletteRegister::Bgp => {
                                            color = ctx.ppu_mmio.bgp_view().color(mixed_pixel_fifo.color_id());
                                        }
                                        PixelFifoPaletteRegister::Obp0 => {
                                            color = ctx.ppu_mmio.obp0_view().color(mixed_pixel_fifo.color_id());
                                        }
                                        PixelFifoPaletteRegister::Obp1 => {
                                            color = ctx.ppu_mmio.obp1_view().color(mixed_pixel_fifo.color_id());
                                        }
                                    }
                                    ctx.ppu_mmio.stream_pixel(color);
                                    ctx.ppu_mmio.next_lx();
                                }
                            }
                        }
                        PpuFetchingMode::FetchSprite => {
                            self.sprite_fetcher.tick(bus, ctx);
                            if self.sprite_fetcher.state() == PixelFetcherState::FetchTileT1 {
                                self.fetching_mode = PpuFetchingMode::FetchBg;
                            }
                        }
                    }

                    // Penalities - Now already included in Pixel FIFO Behavior!
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
            } else if (ctx.ppu_mmio.lx() >= Self::SCREEN_COLUMNS as u8) && (ctx.ppu_mmio.ppu_mode() == PpuMode::Drawing) {
                self.switch_mode = true;
            }
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
