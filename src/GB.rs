use std::cell::{Ref, RefCell};
use std::io::Error;
use std::rc::Rc;
use crate::GB::cartridge::{Cartridge, UseCartridge};
use crate::GB::memory::{RAM};
use crate::GB::memory::BIOS::BIOS;
use crate::GB::cartridge::addresses as cartridge_addresses;
use crate::GB::CPU::CPU_CLOCK_SPEED;
use crate::GB::memory::addresses::OAM_AREA_ADDRESS;
use crate::GB::PPU::constants::OAM_NUMBERS;
use crate::GB::PPU::oam::OAM_BYTE_SIZE;

pub mod registers;
pub mod instructions;
pub mod CPU;
pub mod memory;
pub mod PPU;
pub mod cartridge;
pub mod input;
pub mod audio;


#[cfg(feature = "debug")]
fn debug_print(args: std::fmt::Arguments) {
    println!("{}", args);
}

#[cfg(not(feature = "debug"))]
fn debug_print(_args: std::fmt::Arguments) {
    // Do nothing
}

const SYSTEM_FREQUENCY_CLOCK: u64 = 1_048_576;
const CYCLES_PER_FRAME: u64 = CPU_CLOCK_SPEED / 60;

pub struct GB {
    is_booting: bool,
    pub memory: Rc<RefCell<RAM>>,
    pub bios: BIOS,
    pub cpu: CPU::CPU,
    pub ppu: PPU::PPU,
    pub input: Rc<RefCell<input::GBInput>>,
    cartridge: Rc<RefCell<Option<Cartridge>>>,
    pub cpu_cycles: u64, // Number to cycle needed to complete current CPU instruction. cpu.cycle() is skipped if different from 0
}

impl GB {
    pub fn new(bios: Option<String>) -> Self{
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
            cpu_cycles: 0,
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
        let a = self.memory.borrow().read(0xFF80);
        if !(self.cpu_cycles > 0) {
            self.cpu_cycles = self.cpu.execute_next();
            if self.cpu.dma_transfer {
                self.dma_transfer();
            }
        }
        self.cpu_cycles -= 1;
        self.ppu.cycle();
        self.cpu.interrupt();
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

    pub fn dma_transfer(&mut self) {
        let start_address = (self.memory.borrow().read(memory::registers::DMA) as u16) << 8;
        let mut mem = self.memory.borrow_mut();
        for i in 0..OAM_NUMBERS as u16 {
            let oam_from_addr = start_address + i * OAM_BYTE_SIZE as u16;
            let oam_to_addr = OAM_AREA_ADDRESS as u16 + i * OAM_BYTE_SIZE as u16;
            let val =  mem.read(oam_from_addr);
            mem.write(oam_to_addr, val);
            let val =  mem.read(oam_from_addr + 1);
            mem.write(oam_to_addr + 1, val);
            let val =  mem.read(oam_from_addr + 2);
            mem.write(oam_to_addr + 2, val);
            let val =  mem.read(oam_from_addr + 3);
            mem.write(oam_to_addr + 3, val);
        }
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
