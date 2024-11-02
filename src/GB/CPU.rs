use std::cell::RefCell;
use std::rc::Rc;
use crate::GB::{instructions, SYSTEM_FREQUENCY_CLOCK};
use crate::GB::cartridge::{Cartridge, UseCartridge};
use crate::GB::registers;
use crate::GB::memory::{self, addresses, interrupts, RAM, UseMemory, USER_PROGRAM_ADDRESS};
use crate::GB::memory::interrupts::InterruptFlagsMask;


pub const CPU_CLOCK_MULTIPLIER: u64 = 4;
pub const CPU_CLOCK_SPEED: u64 = SYSTEM_FREQUENCY_CLOCK * CPU_CLOCK_MULTIPLIER; // In Hz - 4 Time System Clock
pub const DIVIDER_FREQUENCY: u64 = 16384; // Divider Update Frequency in Hz

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::GB::CPU::CPU;
    use crate::GB::memory::{RAM, UseMemory, WRAM_ADDRESS, WRAM_SIZE};

    #[test]
    fn cpu_new_8bit_registers() {
        let memory_ref = Rc::new(RefCell::new(RAM::new()));
        let cpu = CPU::new(memory_ref);
        assert_eq!(cpu.registers.get_a(), 0);
        assert_eq!(cpu.registers.get_f(), 0);
        assert_eq!(cpu.registers.get_b(), 0);
        assert_eq!(cpu.registers.get_c(), 0);
        assert_eq!(cpu.registers.get_d(), 0);
        assert_eq!(cpu.registers.get_e(), 0);
        assert_eq!(cpu.registers.get_h(), 0);
        assert_eq!(cpu.registers.get_l(), 0);
    }

    #[test]
    fn cpu_new_16bit_registers() {
        let memory_ref = Rc::new(RefCell::new(RAM::new()));
        let cpu = CPU::new(memory_ref);
        assert_eq!(cpu.registers.get_af(), 0);
        assert_eq!(cpu.registers.get_bc(), 0);
        assert_eq!(cpu.registers.get_de(), 0);
        assert_eq!(cpu.registers.get_hl(), 0);
        assert_eq!(cpu.registers.get_sp(), (WRAM_ADDRESS + WRAM_SIZE - 1) as u16);
        assert_eq!(cpu.registers.get_pc(), 0);
    }

    #[test]
    fn cpu_new_16_8bit_registers() {
        // 16 Bit register should be 0 as the compound of low register is 0 (and should not be altered by access of 8bit register)
        let memory_ref = Rc::new(RefCell::new(RAM::new()));
        let cpu = CPU::new(memory_ref);
        assert_eq!(cpu.registers.get_a(), 0);
        assert_eq!(cpu.registers.get_f(), 0);
        assert_eq!(cpu.registers.get_b(), 0);
        assert_eq!(cpu.registers.get_c(), 0);
        assert_eq!(cpu.registers.get_d(), 0);
        assert_eq!(cpu.registers.get_e(), 0);
        assert_eq!(cpu.registers.get_h(), 0);
        assert_eq!(cpu.registers.get_l(), 0);
        assert_eq!(cpu.registers.get_af(), 0);
        assert_eq!(cpu.registers.get_bc(), 0);
        assert_eq!(cpu.registers.get_de(), 0);
        assert_eq!(cpu.registers.get_hl(), 0);
        assert_eq!(cpu.registers.get_sp(), (WRAM_ADDRESS + WRAM_SIZE - 1) as u16);
        assert_eq!(cpu.registers.get_pc(), 0);
    }

    #[test]
    fn cpu_push_n_pop() {
        let memory_ref = Rc::new(RefCell::new(RAM::new()));
        let mut cpu = CPU::new(memory_ref);
        let start_sp = cpu.registers.get_sp();
        let test_value: u8 = 0x81;
        cpu.push(test_value);
        assert_eq!(cpu.registers.get_sp(), start_sp - 1);
        assert_eq!(cpu.read_memory(start_sp), test_value);

        let popped_val = cpu.pop();
        assert_eq!(cpu.registers.get_sp(), start_sp);
        assert_eq!(popped_val, test_value);
    }
}

pub struct CPU {
    pub registers: registers::Registers,
    pub ime: bool,  // Interrupt Master Enable - True if you want to enable and intercept interrupts
    pub opcode: u8,  // Running Instruction Opcode
    pub cycles: u64,  // Total Cycles Count
    pub divider_counter: u8,  // Total Cycles Count
    pub dma_transfer: bool,  // True When DMA RAM to VRAM is enabled
    pub memory: Rc<RefCell<RAM>>,
    cartridge: Rc<RefCell<Option<Cartridge>>>,
    pub interrupt_routine_cycle: Option<u8>,
    interrupt_routine_addr: u16,
    pub interrupt_type: InterruptFlagsMask
}

impl CPU {
    pub fn new(memory: Rc<RefCell<RAM>>) -> Self {
        Self {
            registers: registers::Registers::new(),
            ime: false,
            opcode: 0,
            cycles: 0,
            divider_counter: 0,
            dma_transfer: false,
            memory,
            cartridge: Rc::new(RefCell::new(None)),
            interrupt_routine_cycle: None,
            interrupt_routine_addr: 0xFFFF,
            interrupt_type: InterruptFlagsMask::VBlank
        }
    }
    
    pub fn fetch_next(&mut self) -> u8 {
        let addr = self.registers.get_and_inc_pc();
        self.read_memory(addr)
    }

    pub fn decode(opcode: u8, cb_opcode: bool) -> Option<&'static instructions::Instruction> {
        let opcode_usize = opcode as usize;
        if cb_opcode {
            return instructions::OPCODES_CB[opcode_usize]
        }
        instructions::OPCODES[opcode_usize]
    }

    pub fn execute_next(&mut self) -> u64 {
        let mut cycles: u64 = 1;
        match self.interrupt_routine_cycle {
            Some(_) => {
                cycles = self.interrupt_routine();
            }
            None => {
                let cb_subset = self.opcode == 0xCB;
                self.opcode = self.fetch_next();
                let instruction = Self::decode(self.opcode, cb_subset);
                match (instruction) {
                    Some(ins) => {
                        cycles = (ins.execute)(&ins, self);
                    },
                    None => {
                        println!("UNKNOWN Opcode '{:#04x}'", self.opcode);
                    }
                }
                if !cb_subset {
                    self.cycles += cycles;
                }
            }
        }
        cycles
    }

    pub fn load(&mut self, data: &Vec<u8>) {
        let mut addr: u16 = 0;
        for byte in data {
            self.write_memory(USER_PROGRAM_ADDRESS as u16 + addr, *byte);
            addr += 1;
        }
        self.registers.set_pc(USER_PROGRAM_ADDRESS as u16);
    }

    /// Check and jump to requested interrupt address after take a snapshot of status on stack.
    /// 
    /// Interrupt bit has priority from lower bit to higher (bit 0 has the higher priority).
    pub fn interrupt(&mut self) -> (bool, InterruptFlagsMask, Option<u8>) {
        let mut interrupt_found = false;
        if self.ime {
            let flags = interrupts::Interrupts::new(self.read_memory(memory::registers::IF));
            let enabled_flags = interrupts::Interrupts::new(self.read_memory(memory::registers::IE));
            if enabled_flags.v_blank && flags.v_blank {
                // Bit 0
                self.interrupt_routine_addr = memory::interrupts::INTERRUPT_VBLANK_ADDR;
                self.interrupt_type = InterruptFlagsMask::VBlank;
            } else if enabled_flags.lcd && flags.lcd {
                // Bit 1
                self.interrupt_routine_addr = memory::interrupts::INTERRUPT_STAT_ADDR;
                self.interrupt_type = InterruptFlagsMask::LCD;
            } else if enabled_flags.timer && flags.timer {
                // Bit 2
                self.interrupt_routine_addr = memory::interrupts::INTERRUPT_TIMER_ADDR;
                self.interrupt_type = InterruptFlagsMask::Timer;
            } else if enabled_flags.serial && flags.serial {
                // Bit 3
                self.interrupt_routine_addr = memory::interrupts::INTERRUPT_SERIAL_ADDR;
                self.interrupt_type = InterruptFlagsMask::Serial;
            } else if enabled_flags.joy_pad && flags.joy_pad {
                // Bit 4
                self.interrupt_routine_addr = memory::interrupts::INTERRUPT_JOYPAD_ADDR;
                self.interrupt_type = InterruptFlagsMask::JoyPad;
            }
            if self.interrupt_routine_addr < 0x70 /* Arbitrary greater than max int addr */ {
                self.ime = false;
                self.interrupt_routine_cycle = Some(0);
                interrupt_found = true;
            }
        }
        (interrupt_found, self.interrupt_type, self.interrupt_routine_cycle)
    }

    /// CPU should run this only when "interrupt_routine_cycle" is not None. Every service routine last 5 cycles.
    fn interrupt_routine(&mut self) -> u64 {
        let routine_cycle = self.interrupt_routine_cycle.unwrap();
        let mut cycles: u64 = 1;
        match routine_cycle {
            0 | 1 => {
                // NOP, just increment routine cycle count
            }
            2 => {
                self.push((self.registers.get_pc() >> 8) as u8);
                self.push((self.registers.get_pc() & 0xFF) as u8);
                cycles = 2;
            }
            v if v >= 4 => {
                let if_register = self.memory.borrow().read(addresses::interrupt::IF as u16);
                self.memory.borrow_mut().write(addresses::interrupt::IF as u16, if_register & !self.interrupt_type);
                self.interrupt_routine_cycle = None;
                self.interrupt_routine_addr = 0xFFFF;
            }
            _ => {}
        }
        match self.interrupt_routine_cycle {
            None => {}
            _ => {self.interrupt_routine_cycle = Some(routine_cycle + 1);}
        }
        cycles
    }

    /*
        CPU Push 1-byte using SP register (to not confuse with instruction PUSH r16, that PUSH in a 2-bytes value from a double-register)
     */
    pub fn push(&mut self, byte: u8) {
        self.write_memory(self.registers.get_sp(), byte);
        self.registers.set_sp(self.registers.get_sp() - 1);
    }

    /*
        CPU Pop 1-byte using SP register (to not confuse with instruction POP r16, that pop out a 2-bytes value to put in a double-register)
     */
    pub fn pop(&mut self) -> u8 {
        self.registers.set_sp(self.registers.get_sp() + 1);
        self.read_memory(self.registers.get_sp())
    }

    pub fn update_divider(&mut self, cycles: u64) {
        let cycles_per_update = CPU_CLOCK_SPEED / DIVIDER_FREQUENCY;

        self.cycles += cycles;
        while self.cycles >= cycles_per_update {
            self.divider_counter = self.divider_counter.wrapping_add(1);
            self.cycles -= cycles_per_update;
        }
    }
}

impl UseMemory for CPU {
    fn read_memory(&self, address: u16) -> u8 {
        self.memory.borrow().read(address)
    }

    fn write_memory(&self, address: u16, data: u8) {
        self.memory.borrow_mut().write(address, data);
    }
}

impl UseCartridge for CPU {
    fn set_cartridge(&mut self, rom: Rc<RefCell<Option<Cartridge>>>) {
        self.cartridge = rom;
    }
}
