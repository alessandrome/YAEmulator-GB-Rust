pub mod registers;
pub mod timer;

use std::cell::RefCell;
use std::rc::Rc;
use crate::GB::{instructions, SYSTEM_FREQUENCY_CLOCK};
use crate::GB::cartridge::{Cartridge, UseCartridge};
use crate::GB::memory::{self, addresses, interrupts, RAM, UseMemory, USER_PROGRAM_ADDRESS};
use crate::GB::memory::interrupts::InterruptFlagsMask;


pub const CPU_CLOCK_MULTIPLIER: u64 = 4;
pub const CPU_CLOCK_SPEED: u64 = SYSTEM_FREQUENCY_CLOCK * CPU_CLOCK_MULTIPLIER; // In Hz - 4 Time System Clock
pub const DIVIDER_FREQUENCY: u64 = 16384; // Divider Update Frequency in Hz
pub const CPU_INTERRUPT_CYCLES: u64 = 5; // Number of cycle to manage a requested Interrupt

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::GB::CPU::CPU;
    use crate::GB::input::GBInput as GBInput;
    use crate::GB::memory::{RAM, UseMemory, WRAM_ADDRESS, WRAM_SIZE};

    #[test]
    fn cpu_new_8bit_registers() {
        let inputs = GBInput::default();
        let inputs_ref = Rc::new(RefCell::new(inputs));
        let memory_ref = Rc::new(RefCell::new(RAM::new(Rc::clone(&inputs_ref))));
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
        let inputs = GBInput::default();
        let inputs_ref = Rc::new(RefCell::new(inputs));
        let memory_ref = Rc::new(RefCell::new(RAM::new(Rc::clone(&inputs_ref))));
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
        let inputs = GBInput::default();
        let inputs_ref = Rc::new(RefCell::new(inputs));
        let memory_ref = Rc::new(RefCell::new(RAM::new(Rc::clone(&inputs_ref))));
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
        let inputs = GBInput::default();
        let inputs_ref = Rc::new(RefCell::new(inputs));
        let memory_ref = Rc::new(RefCell::new(RAM::new(Rc::clone(&inputs_ref))));
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
    pub registers: registers::core::Registers,
    pub ime: bool,  // Interrupt Master Enable - True if you want to enable and intercept interrupts
    pub opcode: u8,  // Running Instruction Opcode
    pub cycles: u64,  // Total Cycles Count
    pub left_cycles: u64,  // Left Cycles to complete currently executing instruction
    pub div_timer_cycles: u64,  // Total Cycles Count
    pub timer_cycles: u64,  // Total Cycles Count
    pub timer_enabled: bool,
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
            registers: registers::core::Registers::new(),
            ime: false,
            opcode: 0,
            cycles: 0,
            left_cycles: 0,
            div_timer_cycles: 0,
            timer_cycles: 0,
            timer_enabled: false,
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
                if self.left_cycles == 0 {
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
                    self.left_cycles = cycles;
                }
            }
        }
        self.cycles += 1;
        self.left_cycles -= 1;
        if self.left_cycles == 0 {
            if self.interrupt().0 {
                self.left_cycles = CPU_INTERRUPT_CYCLES;
            }
        }
        self.update_timers(1);
        1
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
                self.registers.set_pc(self.interrupt_routine_addr);
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

    /// Update the timer DIV and TIMA based on cycle count. Enabled IF Timer flag when TIMA overflows.
    pub fn update_timers(&mut self, cycles: u8) {
        let mut memory_mut = self.memory.borrow_mut();
        self.div_timer_cycles += 1;
        if (self.div_timer_cycles % timer::M64_CLOCK_CYCLES) == 0 {
            let (new_div, div_overflow) = memory_mut.read(memory::registers::DIV).overflowing_add(cycles);
            memory_mut.write(memory::registers::DIV, new_div);
            self.div_timer_cycles = 0;
        }

        let tac = memory_mut.read(memory::registers::TAC);
        if (tac & timer::TACMask::Enabled) != 0 {
            if !self.timer_enabled {
                // Timer has just been re-enabled, resetting timer cycles count
                self.timer_cycles = 0;
                self.timer_enabled = true;
            }
            let mut mode_cycles: u64 = 0;
            match (tac & timer::TACMask::TimerClock) {
                timer::M256_CLOCK_MODE => { mode_cycles = timer::M256_CLOCK_CYCLES; }
                timer::M4_CLOCK_MODE => { mode_cycles = timer::M4_CLOCK_CYCLES; }
                timer::M16_CLOCK_MODE => { mode_cycles = timer::M16_CLOCK_CYCLES; }
                _ => { mode_cycles = timer::M64_CLOCK_CYCLES; }
            }
            self.timer_cycles += 1;

            // Increment and managed TIMA overflow
            for _ in 0..(self.timer_cycles / mode_cycles) {
                let (new_tima, overflowed) = memory_mut.read(memory::registers::TIMA).overflowing_add(1);
                if overflowed {
                    let tma = memory_mut.read(memory::registers::TMA);
                    memory_mut.write(memory::registers::TIMA, tma);
                    let interrupts = memory_mut.read(memory::registers::IF);
                    memory_mut.write(memory::registers::IF, interrupts | InterruptFlagsMask::Timer);
                } else {
                    memory_mut.write(memory::registers::TIMA, new_tima);
                }
            }
        } else {
            self.timer_enabled = false;
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
