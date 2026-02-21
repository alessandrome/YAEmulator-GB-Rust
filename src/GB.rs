pub mod cpu;
pub mod ppu;
pub mod APU;
pub mod cartridge;
pub mod input;
pub mod memory;
pub mod bus;
pub mod types;
pub mod traits;
pub mod utils;
mod timer;
mod interrupt;
pub mod DMA;

use crate::GB::cartridge::addresses as cartridge_addresses;
use crate::GB::input::{GBInputButtonsBits, GBInputDPadBits};
use crate::GB::bus::MmioContext;
use crate::GB::memory::wram::WRAM;
use traits::Tick;


#[cfg(feature = "debug")]
fn debug_print(args: std::fmt::Arguments) {
    println!("{}", args);
}

#[cfg(not(feature = "debug"))]
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
    pub wram: WRAM,
    // pub bios: BIOS, // todo!("Add Bios")
    pub cpu_ctx: cpu::CpuCtx,
    ppu_ctx: ppu::PpuCtx,
    dma_ctx: DMA::DmaCtx,
    apu: APU::APU,
    pub input: input::GBInput,
    cartridge: Option<cartridge::Cartridge>,
    cycles: u64, // Number to cycle needed to complete current CPU instruction. cpu.cycle() is skipped if different from 0
    cycles_overflows: u64, // Number of time cycles has overflowed
}

impl GB {
    pub const SYSTEM_FREQUENCY_CLOCK: u32 = 4_194_304;

    pub fn new(bios: Option<String>) -> Self {
        let inputs = input::GBInput {
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

        match bios {
            None => {}
            Some(bios) => {
                rom.load_bios(&bios);
                is_booting = true;
            }
        }

        Self {
            is_booting,
            bus: bus::Bus::new(),
            cpu_ctx: cpu::CpuCtx {
                cpu: cpu::CPU::new(),
                mmio: cpu::cpu_mmio::CpuMmio::new()
            },
            ppu_ctx: ppu::PpuCtx {
                ppu: ppu::PPU::new(),
                mmio: ppu::ppu_mmio::PpuMmio::new()
            },
            dma_ctx: DMA::DmaCtx {
                dma: DMA::DMA::new(),
                mmio: DMA::dma_mmio::DmaMmio::new()
            },
            wram: WRAM::new(),
            cartridge: None,
            input: input::GBInput::default(),
            apu: APU::APU::new(),
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

    pub fn insert_cartridge(&mut self, path: &String) {
        let cartridge = cartridge::Cartridge::new((*path).clone());
        match cartridge {
            Ok(c) => {
                self.set_cartridge(Rc::new(RefCell::new(Option::from(c))));
            }
            Err(_) => {
                self.set_cartridge(Rc::new(RefCell::new(None)));
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
            ppu_mmio: &mut self.ppu_ctx.mmio,
            dma_mmio: &mut self.dma_ctx.mmio,
            wram: &mut self.wram,
        };

        // Tick every component
        self.apu.tick();
        self.ppu_ctx.ppu.tick(&mut self.bus, &mut ctx);
        self.dma_ctx.dma.tick(&mut self.bus, &mut ctx);
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

    pub fn press_dpad(&mut self, dpad: GBInputDPadBits, pressed: bool) {
        let mut input = self.input.borrow_mut();
        match dpad {
            GBInputDPadBits::Right => {
                input.right = pressed;
            }
            GBInputDPadBits::Left => {
                input.left = pressed;
            }
            GBInputDPadBits::Up => {
                input.up = pressed;
            }
            GBInputDPadBits::Down => {
                input.down = pressed;
            }
        }

        // Update IF (Interrupt Flags) for button pressed
        if pressed {
            let interrupt_flags = self.memory.borrow().read(addresses::interrupt::IF as u16);
            self.memory.borrow_mut().write(
                addresses::interrupt::IF as u16,
                interrupt_flags | interrupts::InterruptFlagsMask::JoyPad,
            );
        }
    }

    pub fn press_button(&mut self, dpad: GBInputButtonsBits, pressed: bool) {
        match dpad {
            GBInputButtonsBits::A => {
                self.input.a = pressed;
            }
            GBInputButtonsBits::B => {
                self.input.b = pressed;
            }
            GBInputButtonsBits::Select => {
                self.input.select = pressed;
            }
            GBInputButtonsBits::Start => {
                self.input.start = pressed;
            }
        }
    }

    pub fn set_use_boot(&mut self, use_boot: bool) {
        self.is_booting = use_boot;
        self.cpu.registers.set_pc(cartridge_addresses::ENTRY_POINT as u16);
    }

    pub fn get_cartridge(&self) -> &Option<cartridge::Cartridge> {
        &self.cartridge
    }

    pub fn get_bios(&self) -> &BIOS {
        &self.bios
    }

    pub fn get_frame_string(&self, doubled: bool) -> String {
        self.ppu_ctx.get_frame_string(doubled)
    }
}

impl Default for GB {
    fn default() -> Self {
       Self::new(None)
    }
}
