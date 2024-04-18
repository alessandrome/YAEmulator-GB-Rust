use std::cell::RefCell;
use std::rc::Rc;
use crate::GB::{instructions, SYSTEM_FREQUENCY_CLOCK};
use crate::GB::cartridge::Cartridge;
use crate::GB::registers;
use crate::GB::memory::{self, RAM, UseMemory, USER_PROGRAM_ADDRESS};


const CPU_CLOCK_MULTIPLIER: u64 = 4;
const CPU_CLOCK_SPEED: u64 = SYSTEM_FREQUENCY_CLOCK * CPU_CLOCK_MULTIPLIER; // In Hz - 4 Time System Clock
const DIVIDER_FREQUENCY: u64 = 16384; // Divider Update Frequency in Hz

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::GB::CPU::CPU;
    use crate::GB::memory::{RAM, WRAM_ADDRESS, WRAM_SIZE};

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
        assert_eq!(cpu.ram.read(start_sp), test_value);

        let popped_val = cpu.pop();
        assert_eq!(cpu.registers.get_sp(), start_sp);
        assert_eq!(popped_val, test_value);
    }
}

pub struct CPU {
    pub registers: registers::Registers,
    pub ram: memory::RAM,
    pub ime: bool,      // Interrupt Master Enable - True if you want to enable and intercept interrupts
    pub opcode: u8,     // Running Instruction Opcode
    pub cycles: u64,     // Total Cycles Count
    pub divider_counter: u8,     // Total Cycles Count
    pub memory: Rc<RefCell<RAM>>,
    cartridge: Rc<RefCell<Option<Cartridge>>>,
}

impl CPU {
    pub fn new(memory: Rc<RefCell<RAM>>) -> Self {
        Self {
            registers: registers::Registers::new(),
            ram: memory::RAM::new(),
            ime: false,
            opcode: 0,
            cycles: 0,
            divider_counter: 0,
            memory,
            cartridge: Rc::new(RefCell::new(None)),
        }
    }
    
    pub fn fetch_next(&mut self) -> u8 {
        self.ram.read(self.registers.get_and_inc_pc())
    }

    pub fn decode(opcode: &u8, cb_opcode: bool) -> Option<&'static instructions::Instruction> {
        let opcode_usize = *opcode as usize;
        if cb_opcode {
            return instructions::OPCODES_CB[opcode_usize]
        }
        instructions::OPCODES[opcode_usize]
    }

    pub fn execute_next(&mut self) -> u64{
        let cb_subset = self.opcode == 0xCB;
        self.opcode = self.fetch_next();
        let instruction = Self::decode(&self.opcode, cb_subset);
        let mut cycles: u64 = 1;
        match (instruction) {
            Some(ins) => {
                cycles = (ins.execute)(&ins, self);
            },
            None => {
                println!("UNKNOWN Opcode '{:#04x}'", self.opcode);
            }
        }
        self.cycles += cycles;
        cycles
    }

    pub fn load(&mut self, data: &Vec<u8>) {
        let mut addr: u16 = 0;
        for byte in data {
            self.ram.write(USER_PROGRAM_ADDRESS as u16 + addr, *byte);
            addr += 1;
        }
        self.registers.set_pc(USER_PROGRAM_ADDRESS as u16);
    }

    /*
        CPU Push 1-byte using SP register (to not confuse with instruction PUSH r16, that PUSH in a 2-bytes value from a double-register)
     */
    pub fn push(&mut self, byte: u8) {
        self.ram.write(self.registers.get_sp(), byte);
        self.registers.set_sp(self.registers.get_sp() - 1);
    }

    /*
        CPU Pop 1-byte using SP register (to not confuse with instruction POP r16, that pop out a 2-bytes value to put in a double-register)
     */
    pub fn pop(&mut self) -> u8 {
        self.registers.set_sp(self.registers.get_sp() + 1);
        self.ram.read(self.registers.get_sp())
    }

    pub fn update_divider(&mut self, cycles: u64) {
        let cycles_per_update = CPU_CLOCK_SPEED / DIVIDER_FREQUENCY;

        self.cycles += cycles;
        while self.cycles >= cycles_per_update {
            self.divider_counter = self.divider_counter.wrapping_add(1);
            self.cycles -= cycles_per_update;
        }
    }

    pub fn set_cartridge(&mut self, rom: Rc<RefCell<Option<Cartridge>>>) {
        self.cartridge = rom;
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
