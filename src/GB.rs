use crate::GB::cartridge::addresses as cartridge_addresses;
use crate::GB::cartridge::{Cartridge, UseCartridge};
use crate::GB::input::{GBInputButtonsBits, GBInputDPadBits};
use crate::GB::memory::addresses::{self, OAM_AREA_ADDRESS};
use crate::GB::memory::BIOS::BIOS;
use crate::GB::memory::{interrupts, RAM};
use crate::GB::CPU::CPU_CLOCK_SPEED;
use crate::GB::PPU::constants::OAM_NUMBERS;
use crate::GB::PPU::oam::OAM_BYTE_SIZE;
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use std::time::Instant;
use crate::GB::memory::interrupts::InterruptFlagsMask;

pub mod CPU;
pub mod PPU;
pub mod APU;
pub mod cartridge;
pub mod input;
pub mod instructions;
pub mod memory;
pub mod bus;
pub mod types;
pub mod traits;
pub mod utils;

#[cfg(feature = "debug")]
fn debug_print(args: std::fmt::Arguments) {
    println!("{}", args);
}

#[cfg(not(feature = "debug"))]
fn debug_print(_args: std::fmt::Arguments) {
    // Do nothing
}

pub const SYSTEM_FREQUENCY_CLOCK: u64 = 1_048_576;
pub const CYCLES_PER_FRAME: u64 = CPU_CLOCK_SPEED / 60;
pub const FRAME_TIME: f64 = 1_f64 / 60_f64;

// #[derive()]
pub struct GB {
    is_booting: bool,
    pub memory: Rc<RefCell<RAM>>,
    pub bios: BIOS,
    pub cpu: CPU::CPU,
    pub ppu: PPU::PPU,
    pub input: Rc<RefCell<input::GBInput>>,
    cartridge: Rc<RefCell<Option<Cartridge>>>,
    // pub cpu_cycles: u64, // Number to cycle needed to complete current CPU instruction. cpu.cycle() is skipped if different from 0
}

impl GB {
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
        let inputs_ref = Rc::new(RefCell::new(inputs));

        let mut ram = RAM::new(Rc::clone(&inputs_ref));
        let ram_ref = Rc::new(RefCell::new(ram));
        let cartridge_ref = Rc::new(RefCell::new(None));
        let cpu = CPU::CPU::new(Rc::clone(&ram_ref));
        let mut rom = BIOS::new();
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
            cpu,
            ppu: PPU::PPU::new(Rc::clone(&ram_ref)),
            memory: ram_ref,
            bios: rom,
            cartridge: cartridge_ref,
            // cpu_cycles: 0,
            input: inputs_ref,
        }
    }

    pub fn boot(&mut self) {
        self.is_booting = true;
        self.memory.borrow_mut().boot_load(&self.bios);
        self.cpu.registers.set_pc(0);
    }

    pub fn insert_cartridge(&mut self, path: &String) {
        let cartridge = Cartridge::new((*path).clone());
        match cartridge {
            Ok(c) => {
                self.set_cartridge(Rc::new(RefCell::new(Option::from(c))));
            }
            Err(_) => {
                self.set_cartridge(Rc::new(RefCell::new(None)));
            }
        }
    }

    pub fn cycle(&mut self) {
        // let time = Instant::now();
        // Execute next only if it hasn't to wait more executing instruction cycles
        // if !(self.cpu_cycles > 0) {
        //     self.cpu_cycles = self.cpu.execute_next();
        //     if self.cpu.
        // {
        //         self.dma_transfer();
        //     }
        // }
        // if self.cpu_cycles > 0 {
        //     self.cpu_cycles -= 1;
        // }
        // if self.cpu_cycles == 0 {
        //     self.cpu.interrupt();
        // }
        self.cpu.execute_next();
        if self.cpu.dma_transfer {
            self.dma_transfer();
        }
        self.ppu.cycle();
        // println!("{:?}", time.elapsed());
    }

    // fn check_interrupt(&mut self) {
    //     if self.cpu.ime {
    //         let memory_borrow = self.memory.borrow();
    //         let interrupt_flags = memory_borrow.read(addresses::interrupt::IF as u16);
    //         let ie = memory_borrow.read(addresses::interrupt::IE as u16);
    //         match ie | interrupt_flags {
    //             x if x & memory::interrupts::InterruptFlagsMask::VBlank != 0 => {
    //
    //             }
    //             x if x & memory::interrupts::InterruptFlagsMask::LCD != 0 => {
    //
    //             }
    //             _ => {}
    //         }
    //     }
    // }

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
        let mut input = self.input.borrow_mut();
        match dpad {
            GBInputButtonsBits::A => {
                input.a = pressed;
            }
            GBInputButtonsBits::B => {
                input.b = pressed;
            }
            GBInputButtonsBits::Select => {
                input.select = pressed;
            }
            GBInputButtonsBits::Start => {
                input.start = pressed;
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

    pub fn get_cartridge(&self) -> Ref<'_, Option<Cartridge>> {
        self.cartridge.borrow()
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

impl UseCartridge for GB {
    fn set_cartridge(&mut self, rom: Rc<RefCell<Option<Cartridge>>>) {
        self.cpu.set_cartridge(Rc::clone(&rom));
        self.memory.borrow_mut().set_cartridge(Rc::clone(&rom));
        self.cartridge = rom;
    }
}
