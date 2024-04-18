use std::cell::{Ref, RefCell};
use std::io::Error;
use std::rc::Rc;
use crate::GB::cartridge::{Cartridge, UseCartridge};
use crate::GB::memory::{RAM, ROM};

pub mod registers;
pub mod instructions;
pub mod CPU;
pub mod memory;
pub mod PPU;
pub mod cartridge;


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
    is_booting: bool,
    pub memory: Rc<RefCell<RAM>>,
    pub rom: ROM,
    pub cpu: CPU::CPU,
    pub ppu: PPU::PPU,
    cartridge: Rc<RefCell<Option<Cartridge>>>
}

impl GB {
    pub fn new(bios: String) -> Self{
        let mut ram = RAM::new();
        let ram_ref = Rc::new(RefCell::new(ram));
        let cartridge_ref = Rc::new(RefCell::new(None));
        let cpu = CPU::CPU::new(Rc::clone(&ram_ref));
        let mut rom = ROM::new();
        rom.load_bios(&bios);
        Self {
            is_booting: true,
            cpu,
            ppu: PPU::PPU::new(Rc::clone(&ram_ref)),
            memory: ram_ref,
            rom,
            cartridge: cartridge_ref,
        }
    }

    pub fn boot(&mut self) {
        self.is_booting = true;
        self.cpu.ram.boot_load(&self.rom);
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
        let mut cycles = 0;
        cycles = self.cpu.execute_next();
    }

    pub fn get_cartridge(&self) -> Ref<'_, Option<Cartridge>> {
        self.cartridge.borrow()
    }
}

impl Default for GB {
    fn default() -> Self {
        let ram = RAM::new();
        let ram_ref = Rc::new(RefCell::new(ram));
        let cartridge_ref = Rc::new(RefCell::new(None));
        let cpu = CPU::CPU::new(Rc::clone(&ram_ref));
        Self {
            is_booting: false,
            cpu,
            ppu: PPU::PPU::new(Rc::clone(&ram_ref)),
            memory: ram_ref,
            rom: ROM::new(),
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
