use crate::GB::cartridge::addresses as cartridge_addresses;
use crate::GB::input::{GBInputButtonsBits, GBInputDPadBits};
use crate::GB::memory::addresses::{self, OAM_AREA_ADDRESS};
use crate::GB::memory::BIOS::BIOS;
use crate::GB::memory::{interrupts, RAM};
use crate::GB::PPU::constants::OAM_NUMBERS;
use crate::GB::PPU::oam::OAM_BYTE_SIZE;
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use std::time::Instant;
use crate::GB::bus::BusContext;
use crate::GB::memory::interrupts::InterruptFlagsMask;
use crate::GB::memory::wram::WRAM;

pub mod CPU;
pub mod PPU;
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
pub struct GB<'a> {
    is_booting: bool,
    bus: bus::Bus,
    pub wram: WRAM,
    // pub bios: BIOS, // todo!("Add Bios")
    pub cpu: CPU::CPU<'a>,
    pub ppu: PPU::PPU,
    dma: DMA::DMA,
    apu: APU::APU,
    pub input: input::GBInput,
    cartridge: Option<cartridge::Cartridge>,
    cycles: u64, // Number to cycle needed to complete current CPU instruction. cpu.cycle() is skipped if different from 0
    cycles_overflows: u64, // Number of time cycles has overflowed
}

impl<'a> GB<'a> {
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
            cpu: CPU::CPU::new(),
            ppu: PPU::PPU::new(Rc::clone(&ram_ref)),
            dma: DMA::DMA::new(),
            wram: WRAM::new(),
            cartridge: None,
            input: input::GBInput::default(),
            apu: APU::APU::new(),
            cycles: 0,
            cycles_overflows: 0,
        }
    }

    fn with_bus<T>(&mut self, f: impl FnOnce(&mut BusContext) -> T) -> T {
        let mut ctx = BusContext {
            cpu: &mut self.cpu,
            apu: &mut self.apu,
            dma: &mut self.dma,
            wram: &mut self.wram,
        };
        f(&mut ctx)
    }

    pub fn boot(&mut self) {
        self.is_booting = true;
        self.memory.borrow_mut().boot_load(&self.bios);
        self.cpu.registers.set_pc(0);
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
        let mut ctx = BusContext {
            cpu: &mut self.cpu,
            apu: &mut self.apu,
            dma: &mut self.dma,
            wram: &mut self.wram,
        };

        // Tick every component
        self.ppu.tick();
        self.apu.tick();
        self.ppu.tick();
        self.dma.tick(&mut self.bus, &mut ctx);
        self.cpu.tick(&mut self.bus, &mut ctx);
        
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

        // Update IF (Interrupt Flags) for button pressed
        if pressed {
            let interrupt_flags = self.memory.borrow().read(addresses::interrupt::IF as u16);
            self.memory.borrow_mut().write(
                addresses::interrupt::IF as u16,
                interrupt_flags | interrupts::InterruptFlagsMask::JoyPad,
            );
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
        self.ppu.get_frame_string(doubled)
    }

    pub fn dma_transfer(&mut self) {
        let start_address = (self.memory.borrow().read(memory::registers::DMA) as u16) << 8;
        let mut mem = self.memory.borrow_mut();
        for i in 0..OAM_NUMBERS as u16 {
            let oam_from_addr = start_address + i * OAM_BYTE_SIZE as u16;
            let oam_to_addr = OAM_AREA_ADDRESS as u16 + i * OAM_BYTE_SIZE as u16;
            let val = mem.read(oam_from_addr);
            mem.write(oam_to_addr, val);
            let val = mem.read(oam_from_addr + 1);
            mem.write(oam_to_addr + 1, val);
            let val = mem.read(oam_from_addr + 2);
            mem.write(oam_to_addr + 2, val);
            let val = mem.read(oam_from_addr + 3);
            mem.write(oam_to_addr + 3, val);
        }
        self.cpu.dma_transfer = false;
    }

    pub fn is_managing_interrupt(&self) -> (InterruptFlagsMask, Option<u8>) {
        (self.cpu.interrupt_type, self.cpu.interrupt_routine_cycle)
    }

    pub fn is_cpu_managing_interrupt(&self) -> bool {
        match self.cpu.interrupt_routine_cycle {
            Some(_) => { true }
            _ => false
        }
    }

    pub fn cpu_interrupt_type(&self) -> interrupts::InterruptFlagsMask {
        self.cpu.interrupt_type
    }

    pub fn cpu_left_instruction_cycles(&self) -> u64 {
        self.cpu.left_cycles
    }
}

impl Default for GB {
    fn default() -> Self {
        let inputs_ref = Rc::new(RefCell::new(input::GBInput::default()));
        let ram = RAM::new(Rc::clone(&inputs_ref));
        let ram_ref = Rc::new(RefCell::new(ram));
        let cartridge_ref = Rc::new(RefCell::new(None));
        let cpu = CPU::CPU::new(Rc::clone(&ram_ref));
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
        Self {
            is_booting: false,
            cpu,
            ppu: PPU::PPU::new(Rc::clone(&ram_ref)),
            memory: ram_ref,
            bios: BIOS::new(),
            input: inputs_ref,
            cartridge: cartridge_ref,
        }
    }
}
