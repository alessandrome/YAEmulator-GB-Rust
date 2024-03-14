use crate::GB::instructions;
use crate::GB::registers;
use crate::GB::RAM;

pub struct CPU {
    pub registers: registers::Registers,
    pub ram: RAM::RAM,
}

impl CPU {
    pub fn fetch_next(&mut self) -> u8 {
        self.ram.read(self.registers.get_and_inc_pc())
    }
    pub fn decode(opcode: &u8, cb_opcode: bool) -> Option<&'static instructions::Instruction> {
        if cb_opcode {
            instructions::OPCODES_CB[opcode]
        }
        instructions::OPCODES[opcode]
    }
}
