use crate::GB::instructions;
use crate::GB::registers;
use crate::GB::RAM;

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
        self.ram.read(self.registers.get_and_inc_pc())
    }

    pub fn decode(opcode: &u8, cb_opcode: bool) -> Option<&'static instructions::Instruction> {
        let opcode_usize = *opcode as usize;
        if cb_opcode {
            return instructions::OPCODES_CB[opcode_usize]
        }
        instructions::OPCODES[opcode_usize]
    }

    pub fn execute_next(&mut self) {
        self.opcode = self.fetch_next();
        let instruction = Self::decode(&self.opcode, false);
        match (instruction) {
            Some(ins) => {
                self.cycles += (ins.execute)(&ins, self);
            },
            None => {
                println!("UNKNOWN Opcode '{:#04x}'", self.opcode);
            }
        }
    }
}
