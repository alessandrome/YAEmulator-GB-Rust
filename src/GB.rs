pub mod cpu;
pub mod ppu;
pub mod apu;
pub mod cartridge;
pub mod joypad;
pub mod memory;
pub mod bus;
pub mod types;
pub mod traits;
pub mod utils;
mod timer;
mod interrupt;
pub mod dma;

use crate::GB::cartridge::addresses as cartridge_addresses;
use crate::GB::joypad::{JoypadButtonsBits, JoypadDPadBits};
use crate::GB::bus::MmioContext;
use traits::Tick;
use crate::GB::ppu::PPU;
use crate::GB::ppu::tile::GbColor;

#[cfg(feature = "debug")]
#[inline]
fn debug_print(args: std::fmt::Arguments) {
    println!("{}", args);
}

#[cfg(not(feature = "debug"))]
#[inline]
fn debug_print(_args: std::fmt::Arguments) {
    // Do nothing
}

pub const SYSTEM_FREQUENCY_CLOCK: u64 = 1_048_576;
pub const CYCLES_PER_FRAME: u64 = SYSTEM_FREQUENCY_CLOCK / 60;
pub const FRAME_TIME: f64 = 1_f64 / 60_f64;

// #[derive()]
pub struct GB {
    is_booting: bool,
    bus: bus::Bus,
    pub wram: memory::wram::WRAM,
    pub oam_memory: memory::oam_memory::OamMemory,
    // pub bios: BIOS, // todo!("Add Bios")
    cpu_ctx: cpu::CpuCtx,
    ppu_ctx: ppu::PpuCtx,
    dma_ctx: dma::DmaCtx,
    apu_ctx: apu::ApuCtx,
    input: joypad::Joypad,
    cartridge: Option<cartridge::Cartridge>,
    cycles: u64, // Number to cycle needed to complete current CPU instruction. cpu.cycle() is skipped if different from 0
    cycles_overflows: u64, // Number of time cycles has overflowed
}

impl GB {
    pub const SYSTEM_FREQUENCY_CLOCK: u32 = 4_194_304;

    pub fn new(bios: Option<String>) -> Self {
        let inputs = joypad::Joypad {
            a: false,
            b: false,
            start: false,
            select: false,
            up: false,
            down: false,
            left: false,
            right: false,
        };

        // Todo!("Add Bios")
        // let mut rom = BIOS::new();
        let mut is_booting = false;

        // match bios {
        //     None => {}
        //     Some(bios) => {
        //         rom.load_bios(&bios);
        //         is_booting = true;
        //     }
        // }

        Self {
            is_booting,
            bus: bus::Bus::new(),
            cpu_ctx: cpu::CpuCtx {
                cpu: cpu::CPU::new(),
                mmio: cpu::cpu_mmio::CpuMmio::new()
            },
            ppu_ctx: ppu::PpuCtx {
                ppu: ppu::PPU::new(),
                lcd: ppu::lcd::LCD::new(),
                mmio: ppu::ppu_mmio::PpuMmio::new()
            },
            apu_ctx: apu::ApuCtx {
                apu: apu::APU::new(),
                mmio: apu::apu_mmio::ApuMmio::new(),
            },
            dma_ctx: dma::DmaCtx {
                dma: dma::DMA::new(),
                mmio: dma::dma_mmio::DmaMmio::new()
            },
            oam_memory: memory::oam_memory::OamMemory::new(),
            wram: memory::wram::WRAM::new(),
            cartridge: None,
            input: joypad::Joypad::default(),
            cycles: 0,
            cycles_overflows: 0,
        }
    }

    // fn with_bus<T>(&mut self, f: impl FnOnce(&mut MmioContext) -> T) -> T {
    //     let mut ctx = MmioContext {
    //         cpu: &mut self.cpu,
    //         apu: &mut self.apu,
    //         dma: &mut self.dma,
    //         wram: &mut self.wram,
    //     };
    //     f(&mut ctx)
    // }

    pub fn boot(&mut self) {
        // self.is_booting = true;
        // self.memory.borrow_mut().boot_load(&self.bios);
        // self.cpu.registers.set_pc(0);
    }

    pub fn insert_cartridge(&mut self, path: &String) -> Result<(), std::io::Error> {
        let cartridge = cartridge::Cartridge::new((*path).clone());
        match cartridge {
            Ok(c) => {
                self.cartridge = Option::from(c);
                Ok(())
            }
            Err(err) => {
                self.cartridge = Option::None;
                Err(err)
            }
        }
    }

    /**
    A single T-Cycle ticking
    */
    pub fn tick(&mut self) {
        // let time = Instant::now();
        let mut ctx = MmioContext {
            cpu_mmio: &mut self.cpu_ctx.mmio,
            rom_mmio: &mut self.cartridge,
            ppu_mmio: &mut self.ppu_ctx.mmio,
            apu_mmio: &mut self.apu_ctx.mmio,
            dma_mmio: &mut self.dma_ctx.mmio,
            oam_mmio: &mut self.oam_memory,
            wram_mmio: &mut self.wram,
        };

        // Tick every component
        self.apu_ctx.apu.tick(&mut self.bus, &mut ctx);
        self.ppu_ctx.ppu.tick(&mut self.bus, &mut ctx);
        self.dma_ctx.dma.tick(&mut self.bus, &mut ctx);
        self.ppu_ctx.lcd.tick(&mut self.bus, &mut ctx);
        self.cpu_ctx.cpu.tick(&mut self.bus, &mut ctx);

        // if self.cpu.dma_transfer {
        //     self.dma_transfer();
        // }
        // println!("{:?}", time.elapsed());
        // Update cycles
        self.cycles = self.cycles.wrapping_add(1);
        if self.cycles == 0 {
            self.cycles_overflows = self.cycles_overflows.wrapping_add(1);
        }
    }

    pub fn press_dpad(&mut self, dpad: JoypadDPadBits, pressed: bool) {
        match dpad {
            JoypadDPadBits::Right => {
                self.input.right = pressed;
            }
            JoypadDPadBits::Left => {
                self.input.left = pressed;
            }
            JoypadDPadBits::Up => {
                self.input.up = pressed;
            }
            JoypadDPadBits::Down => {
                self.input.down = pressed;
            }
        }
    }

    pub fn press_button(&mut self, dpad: JoypadButtonsBits, pressed: bool) {
        match dpad {
            JoypadButtonsBits::A => {
                self.input.a = pressed;
            }
            JoypadButtonsBits::B => {
                self.input.b = pressed;
            }
            JoypadButtonsBits::Select => {
                self.input.select = pressed;
            }
            JoypadButtonsBits::Start => {
                self.input.start = pressed;
            }
        }
    }

    pub fn set_use_boot(&mut self, use_boot: bool) {
        self.is_booting = use_boot;
        self.cpu_ctx.cpu.registers.set_pc(cartridge_addresses::ENTRY_POINT as u16);
    }

    pub fn get_cartridge(&self) -> &Option<cartridge::Cartridge> {
        &self.cartridge
    }

    // pub fn get_bios(&self) -> &BIOS {
    //     &self.bios
    // }

    pub fn get_frame(&self) -> &[GbColor; PPU::SCREEN_PIXELS as usize] {
        self.ppu_ctx.lcd.screen()
    }
}

impl Default for GB {
    fn default() -> Self {
       Self::new(None)
    }
}
