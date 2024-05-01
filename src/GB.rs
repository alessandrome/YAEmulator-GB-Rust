use std::cell::{Ref, RefCell};
use std::io::Error;
use std::rc::Rc;
use crate::GB::cartridge::{Cartridge, UseCartridge};
use crate::GB::memory::{RAM};
use crate::GB::memory::BIOS::BIOS;
use crate::GB::cartridge::addresses as cartridge_addresses;

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
    pub bios: BIOS,
    pub cpu: CPU::CPU,
    pub ppu: PPU::PPU,
    cartridge: Rc<RefCell<Option<Cartridge>>>,
    pub cpu_cycles: u64, // Number to cycle needed to complete current CPU instruction. cpu.cycle() is skipped if different from 0
}

impl GB {
    pub fn new(bios: Option<String>) -> Self{
        let mut ram = RAM::new();
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
            cpu_cycles: 0,
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
        if !(self.cpu_cycles > 0) {
            self.cpu_cycles = self.cpu.execute_next();
        }
        self.cpu_cycles -= 1;
        self.ppu.cycle();
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
            bios: BIOS::new(),
            cartridge: cartridge_ref,
            cpu_cycles: 0,
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
