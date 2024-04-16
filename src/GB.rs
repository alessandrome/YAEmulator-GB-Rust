use std::cell::RefCell;
use std::rc::Rc;
use crate::GB::memory::{RAM, ROM};

pub mod registers;
pub mod instructions;
pub mod CPU;
pub mod memory;
pub mod PPU;


#[cfg(feature = "debug")]
fn debug_print(args: std::fmt::Arguments) {
    println!("{}", args);
}

#[cfg(not(feature = "debug"))]
fn debug_print(_args: std::fmt::Arguments) {
    // Do nothing
}

const SYSTEM_FREQUENCY_CLOCK: u64 = 1_048_576;

pub struct GB {
    pub memory: Rc<RefCell<RAM>>,
    pub rom: ROM,
    pub cpu: CPU::CPU,
    pub ppu: PPU::PPU,
}

impl GB {
    pub fn new(bios: String) -> Self{
        let mut ram = RAM::new();
        let ram_ref = Rc::new(RefCell::new(ram));
        let mut rom = ROM::new();
        rom.load_bios(&bios);
        Self {
            cpu: CPU::CPU::new(Rc::clone(&ram_ref)),
            ppu: PPU::PPU::new(Rc::clone(&ram_ref)),
            memory: ram_ref,
            rom,
        }
    }

    pub fn boot(&mut self) {
        self.cpu.ram.boot_load(&self.rom);
        self.cpu.registers.set_pc(0);
    }
}

impl Default for GB {
    fn default() -> Self {
        let ram = RAM::new();
        let ram_ref = Rc::new(RefCell::new(ram));
        Self {
            cpu: CPU::CPU::new(Rc::clone(&ram_ref)),
            ppu: PPU::PPU::new(Rc::clone(&ram_ref)),
            memory: ram_ref,
            rom: ROM::new(),
        }
    }
}