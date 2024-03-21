use crate::GB::instructions;
use crate::GB::registers;
use crate::GB::RAM;

#[cfg(test)]
mod test {
    use crate::GB::CPU::CPU;

    #[test]
    fn cpu_new_8bit_registers() {
        let cpu = CPU::new();
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
        let cpu = CPU::new();
        assert_eq!(cpu.registers.get_af(), 0);
        assert_eq!(cpu.registers.get_bc(), 0);
        assert_eq!(cpu.registers.get_de(), 0);
        assert_eq!(cpu.registers.get_hl(), 0);
        assert_eq!(cpu.registers.get_sp(), 0);
        assert_eq!(cpu.registers.get_pc(), 0);
    }

    #[test]
    fn cpu_new_16_8bit_registers() {
        // 16 Bit register should be 0 as the compound of low register is 0 (and should not be altered by access of 8bit register)
        let cpu = CPU::new();
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
        assert_eq!(cpu.registers.get_sp(), 0);
        assert_eq!(cpu.registers.get_pc(), 0);
    }
}

pub struct CPU {
    pub registers: registers::Registers,
    pub ram: RAM::RAM,
    pub opcode: u8,     // Running Instruction Opcode
    pub cycles: u64     // Total Cycles Count
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: registers::Registers::new(),
            ram: RAM::RAM::new(),
            opcode: 0,
            cycles: 0,
        }
    }
    
    pub fn fetch_next(&mut self) -> u8 {
        self.ram.read_user_program(self.registers.get_and_inc_pc())
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
            self.ram.write_user_program(addr, *byte);
            addr += 1;
        }
        self.registers.set_pc(0);
    }
}
