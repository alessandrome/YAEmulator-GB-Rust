use crate::GB::memory::ROM;

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
    pub cpu: CPU::CPU,
    pub rom: ROM,
}

impl GB {
    pub fn new(bios: String) -> Self{
        let mut rom = ROM::new();
        rom.load_bios(&bios);
        Self {
            cpu: CPU::CPU::new(),
            rom: rom
        }
    }

    pub fn boot(&mut self) {
        self.cpu.ram.boot_load(&self.rom);
        self.cpu.registers.set_pc(0);
    }
}

impl Default for GB {
    fn default() -> Self {
        Self {
            rom: ROM::new(),
            cpu: CPU::CPU::new()
        }
    }
}