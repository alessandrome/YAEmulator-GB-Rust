use crate::GB::CPU::CPU;
use crate::GB::debug_print;
use crate::GB::registers::{FlagBits, Flags};

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub name: &'static str,
    pub cycles: u8,
    pub size: u8,
    pub flags: &'static [FlagBits],
    pub execute: fn(&Instruction, &mut CPU) -> u64, // Return number on M-Cycles needed to execute
}

impl Instruction {
    pub fn new(opcode: u8) -> Option<&'static Self> {
        OPCODES[opcode as usize]
    }
}

const fn create_opcodes() -> [Option<&'static Instruction>; 256] {
    let mut opcodes = [None; 256];
    opcodes[0x00] = Some(&Instruction {
        opcode: 0x00,
        name: "NOP",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // NOP Do nothing
            opcode.cycles as u64
        },
    });
    opcodes[0x01] = Some(&Instruction {
        opcode: 0x01,
        name: "LD BC, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte_1 = cpu.fetch_next();
            let byte_2 = cpu.fetch_next();
            let mut dual_byte = byte_1 as u16 & 0xFF;
            dual_byte = dual_byte | (byte_2 as u16) << 8;
            cpu.registers.set_bc(dual_byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x02] = Some(&Instruction {
        opcode: 0x02,
        name: "LD [BC], A",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_bc(), cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x03] = Some(&Instruction {
        opcode: 0x03,
        name: "INC BC",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_bc(cpu.registers.get_bc() + 1);
            opcode.cycles as u64
        },
    });
    opcodes[0x04] = Some(&Instruction {
        opcode: 0x04,
        name: "INC B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_b = cpu.registers.get_b();
            cpu.registers.set_b(cpu.registers.get_b().wrapping_add(1));
            // Write flags (This could be calculated and place and just made a single functions call to set Flag register)
            cpu.registers.set_half_carry_flag((original_b & 0x0F) == 0x0F);
            cpu.registers.set_zero_flag(cpu.registers.get_b() == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x05] = Some(&Instruction {
        opcode: 0x05,
        name: "DEC B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_b = cpu.registers.get_b();
            cpu.registers.set_b(cpu.registers.get_b().wrapping_sub(1));
            // Write flags
            cpu.registers.set_half_carry_flag((original_b & 0x0F) == 0);
            cpu.registers.set_zero_flag(cpu.registers.get_b() == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x06] = Some(&Instruction {
        opcode: 0x06,
        name: "LD B, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_b(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x07] = Some(&Instruction {
        opcode: 0x07,
        name: "RLCA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_carry_flag((cpu.registers.get_a() & 0b1000_0000) != 0);
            cpu.registers.set_a(cpu.registers.get_a().wrapping_shl(1) | cpu.registers.get_a().wrapping_shr(7));
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x08] = Some(&Instruction {
        opcode: 0x08,
        name: "LD [a16], SP",
        cycles: 5,
        size: 3,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte_low_addr = cpu.fetch_next();
            let byte_high_addr = cpu.fetch_next();
            let imm16_address: u16 = (byte_high_addr as u16) << 8 | (byte_low_addr as u16) & 0xFF;
            cpu.ram.write(imm16_address, (cpu.registers.get_sp() & 0xFF) as u8);
            cpu.ram.write(imm16_address + 1, (cpu.registers.get_sp() >> 8) as u8);
            opcode.cycles as u64
        },
    });
    opcodes[0x09] = Some(&Instruction {
        opcode: 0x09,
        name: "ADD HL, BC",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // Half carry is checked on bit 11 of HL (or bit 3 of H)
            let pre_h = cpu.registers.get_h();
            let pre_l = cpu.registers.get_l();
            cpu.registers.set_negative_flag(false);
            if (cpu.registers.get_bc() != 0) {
                cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(cpu.registers.get_bc()));
                cpu.registers.set_half_carry_flag((cpu.registers.get_h() & 0x0F) < (pre_h & 0x0F));
                cpu.registers.set_carry_flag(cpu.registers.get_h() < pre_h);
            } else {
                cpu.registers.set_half_carry_flag(false);
                cpu.registers.set_carry_flag(false);
            }
            opcode.cycles as u64
        },
    });
    opcodes[0x0A] = Some(&Instruction {
        opcode: 0x0A,
        name: "LD A, [BC]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.ram.read(cpu.registers.get_bc()));
            opcode.cycles as u64
        },
    });
    opcodes[0x0B] = Some(&Instruction {
        opcode: 0x0B,
        name: "DEC BC",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_bc(cpu.registers.get_bc().wrapping_sub(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x0C] = Some(&Instruction {
        opcode: 0x0C,
        name: "INC C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_c = cpu.registers.get_c();
            cpu.registers.set_c(cpu.registers.get_c().wrapping_add(1));
            cpu.registers.set_zero_flag(cpu.registers.get_c() == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((original_c & 0x0F) == 0x0F);
            opcode.cycles as u64
        },
    });
    opcodes[0x0D] = Some(&Instruction {
        opcode: 0x0D,
        name: "DEC C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_c = cpu.registers.get_c();
            cpu.registers.set_c(cpu.registers.get_c().wrapping_sub(1));
            cpu.registers.set_zero_flag(cpu.registers.get_c() == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((original_c & 0x0F) == 0);
            opcode.cycles as u64
        },
    });
    opcodes[0x0E] = Some(&Instruction {
        opcode: 0x0E,
        name: "LD C, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_c(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x0F] = Some(&Instruction {
        opcode: 0x0F,
        name: "RRCA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            cpu.registers.set_carry_flag(cpu.registers.get_a() & 1 != 0);
            cpu.registers.set_a((cpu.registers.get_a().wrapping_shr(1)) | cpu.registers.get_a().wrapping_shl(7));
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x10] = Some(&Instruction {
        opcode: 0x10,
        name: "STOP imm8",
        cycles: 1,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: implement
            let byte = cpu.fetch_next();
            opcode.cycles as u64
        },
    });
    opcodes[0x11] = Some(&Instruction {
        opcode: 0x11,
        name: "LD DE, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte_1 = cpu.fetch_next();
            let byte_2 = cpu.fetch_next();
            let mut dual_byte = byte_1 as u16 & 0xFF;
            dual_byte = dual_byte | (byte_2 as u16) << 8;
            cpu.registers.set_de(dual_byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x12] = Some(&Instruction {
        opcode: 0x12,
        name: "LD [DE], A",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_de(), cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x13] = Some(&Instruction {
        opcode: 0x13,
        name: "INC DE",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_de(cpu.registers.get_de() + 1);
            opcode.cycles as u64
        },
    });
    opcodes[0x14] = Some(&Instruction {
        opcode: 0x14,
        name: "INC D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_d = cpu.registers.get_d();
            cpu.registers.set_b(cpu.registers.get_d().wrapping_add(1));
            // Write flags (This could be calculated and place and just made a single functions call to set Flag register)
            cpu.registers.set_half_carry_flag((cpu.registers.get_d() & 0x0F) < (original_d & 0x0F));
            cpu.registers.set_zero_flag(cpu.registers.get_d() == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x15] = Some(&Instruction {
        opcode: 0x15,
        name: "DEC D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_d = cpu.registers.get_d();
            cpu.registers.set_d(cpu.registers.get_d().wrapping_sub(1));
            // Write flags
            cpu.registers.set_half_carry_flag(cpu.registers.get_d() > original_d);
            cpu.registers.set_zero_flag(cpu.registers.get_d() == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x16] = Some(&Instruction {
        opcode: 0x16,
        name: "LD D, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_d(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x17] = Some(&Instruction {
        opcode: 0x17,
        name: "RLA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let original_carry_flag: u8 = cpu.registers.get_carry_flag() as u8;
            cpu.registers.set_carry_flag((cpu.registers.get_a() & 0b1000_0000) != 0);
            cpu.registers.set_a(cpu.registers.get_a().wrapping_shl(1) | original_carry_flag);
            cpu.registers.set_zero_flag(true);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x18] = Some(&Instruction {
        opcode: 0x18,
        name: "JR e8",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i16;
            cpu.registers.set_pc(cpu.registers.get_pc() -  if byte < 0 {byte.abs()} else {byte} as u16);
            opcode.cycles as u64
        },
    });
    opcodes[0x19] = Some(&Instruction {
        opcode: 0x19,
        name: "ADD HL, DE",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let pre_h = cpu.registers.get_h();
            let pre_l = cpu.registers.get_l();
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(cpu.registers.get_de()));
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(cpu.registers.get_l() < pre_l);
            cpu.registers.set_carry_flag(cpu.registers.get_h() < pre_h);
            opcode.cycles as u64
        },
    });
    opcodes[0x1A] = Some(&Instruction {
        opcode: 0x1A,
        name: "LD A, [DE]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.ram.read(cpu.registers.get_de()));
            opcode.cycles as u64
        },
    });
    opcodes[0x1B] = Some(&Instruction {
        opcode: 0x1B,
        name: "DEC DE",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_de(cpu.registers.get_de().wrapping_sub(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x1C] = Some(&Instruction {
        opcode: 0x1C,
        name: "INC E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_e = cpu.registers.get_e();
            cpu.registers.set_e(cpu.registers.get_e().wrapping_add(1));
            cpu.registers.set_zero_flag(cpu.registers.get_e() == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(original_e > cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x1D] = Some(&Instruction {
        opcode: 0x1D,
        name: "DEC E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_e = cpu.registers.get_e();
            cpu.registers.set_e(cpu.registers.get_e().wrapping_sub(1));
            cpu.registers.set_zero_flag(cpu.registers.get_e() == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(original_e < cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x1E] = Some(&Instruction {
        opcode: 0x1E,
        name: "LD E, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_e(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x1F] = Some(&Instruction {
        opcode: 0x1F,
        name: "RRA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let original_carry_flag: u8 = cpu.registers.get_carry_flag() as u8;
            cpu.registers.set_carry_flag(cpu.registers.get_a() & 1 != 0);
            cpu.registers.set_a((cpu.registers.get_a().wrapping_shr(1)) | original_carry_flag.wrapping_shl(7));
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x20] = Some(&Instruction {
        opcode: 0x20,
        name: "JR NZ, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i16;
            if !cpu.registers.get_zero_flag() {
                cpu.registers.set_pc(cpu.registers.get_pc() - if byte < 0 { byte.abs() } else { byte } as u16);
                return opcode.cycles as u64;
            }
            2
        },
    });
    opcodes[0x21] = Some(&Instruction {
        opcode: 0x21,
        name: "LD HL, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte_1 = cpu.fetch_next();
            let byte_2 = cpu.fetch_next();
            let mut dual_byte = byte_1 as u16 & 0xFF;
            dual_byte = dual_byte | (byte_2 as u16) << 8;
            cpu.registers.set_hl(dual_byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x22] = Some(&Instruction {
        opcode: 0x22,
        name: "LD [HL+], A", // Sometimes HL+ is named as HLI (HL Increment)
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_a());
            cpu.registers.set_hl(cpu.registers.get_hl() + 1);
            opcode.cycles as u64
        },
    });
    opcodes[0x23] = Some(&Instruction {
        opcode: 0x23,
        name: "INC HL",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_hl(cpu.registers.get_hl() + 1);
            opcode.cycles as u64
        },
    });
    opcodes[0x24] = Some(&Instruction {
        opcode: 0x24,
        name: "INC H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_h = cpu.registers.get_h();
            cpu.registers.set_b(cpu.registers.get_h().wrapping_add(1));
            cpu.registers.set_half_carry_flag((cpu.registers.get_h() & 0x0F) < (original_h & 0x0F));
            cpu.registers.set_zero_flag(cpu.registers.get_h() == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x30] = Some(&Instruction {
        opcode: 0x20,
        name: "JR NZ, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i16;
            if !cpu.registers.get_carry_flag() {
                cpu.registers.set_pc(cpu.registers.get_pc() - if byte < 0 { byte.abs() } else { byte } as u16);
                return opcode.cycles as u64;
            }
            2
        },
    });
    opcodes[0x31] = Some(&Instruction {
        opcode: 0x31,
        name: "LD SP, imm16",
        cycles: 3,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte_1 = cpu.fetch_next();
            let byte_2 = cpu.fetch_next();
            let mut dual_byte = byte_1 as u16 & 0xFF;
            dual_byte = dual_byte | (byte_2 as u16) << 8;
            cpu.registers.set_sp(dual_byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x32] = Some(&Instruction {
        opcode: 0x32,
        name: "LD [HL-], A", // Sometimes HL- is named as HLD (HL Decrement)
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_a());
            cpu.registers.set_hl(cpu.registers.get_hl() - 1);
            opcode.cycles as u64
        },
    });
    opcodes[0x33] = Some(&Instruction {
        opcode: 0x33,
        name: "INC SP",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_sp(cpu.registers.get_sp() + 1);
            opcode.cycles as u64
        },
    });
    opcodes[0xCB] = Some(&Instruction {
        opcode: 0xCB,
        name: "CB SUBSET",
        cycles: 0,
        size: 0,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // Implement CB subset will be executed on next fetching & execute
            opcode.cycles as u64
        },
    });
    opcodes
}

const fn create_cb_opcodes() -> [Option<&'static Instruction>; 256] {
    let mut opcodes = [None; 256];
    // opcodes[0x00] = Some(&Instruction {
    //     opcode: 0x00,
    //     name: "NOP",
    //     cycles: 1,
    //     size: 1,
    //     flags: &[],
    //     execute: fn a(cpu: &CPU) -> u8,
    // });
    // opcodes[0x01] = Some(&Instruction {
    //     opcode: 0x01,
    //     name: "LD BC, d16",
    //     cycles: 3,
    //     size: 3,
    //     flags: &[],
    //     execute: fn a(cpu: &CPU) -> u8,
    // });
    // opcodes[0xCB] = Some(&Instruction {
    //     opcode: 0xCB,
    //     name: "LD BC, d16",
    //     cycles: 3,
    //     size: 3,
    //     flags: &[],
    //     execute: fn a(cpu: &CPU) -> u8{},
    // });
    opcodes
}

pub const OPCODES: [Option<&'static Instruction>; 256] = create_opcodes();

pub const OPCODES_CB: [Option<&'static Instruction>; 256] = create_cb_opcodes();


#[cfg(test)]
mod test {
    use crate::GB::CPU::CPU;
    use crate::GB::RAM;
    use crate::GB::RAM::WRAM_ADDRESS;

    #[test]
    fn test_0x00_nop() {
        let mut cpu = CPU::new();
        let registers_copy = cpu.registers;
        let program: Vec<u8> = vec![0x00, 0x00];
        cpu.load(&program);
        cpu.execute_next();
        cpu.execute_next();
        // NOP should not alter any register
        assert_eq!(registers_copy.get_a(), cpu.registers.get_a());
        assert_eq!(registers_copy.get_b(), cpu.registers.get_b());
        assert_eq!(registers_copy.get_c(), cpu.registers.get_c());
        assert_eq!(registers_copy.get_d(), cpu.registers.get_d());
        assert_eq!(registers_copy.get_e(), cpu.registers.get_e());
        assert_eq!(registers_copy.get_f(), cpu.registers.get_f());
        assert_eq!(registers_copy.get_h(), cpu.registers.get_h());
        assert_eq!(registers_copy.get_l(), cpu.registers.get_l());
    }

    #[test]
    fn test_0x01_ld_bc_imm16() {
        let test_value: u16 = 0xC05A;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x01, 0x5A, 0xC0];
        cpu.load(&program);
        cpu.execute_next();
        // NOP should not alter any register
        assert_eq!(cpu.registers.get_b(), 0xC0);
        assert_eq!(cpu.registers.get_c(), 0x5A);
        assert_eq!(cpu.registers.get_bc(), test_value);
    }

    #[test]
    fn test_0x02_ld__bc__a() {
        let test_value: u8 = 0xF4;
        let test_address: u16 = RAM::WRAM_ADDRESS as u16 + 0x0500;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x02];
        cpu.load(&program);
        cpu.registers.set_a(test_value);
        cpu.registers.set_bc(test_address);
        cpu.execute_next();
        // NOP should not alter any register
        assert_eq!(cpu.registers.get_a(), test_value);
        assert_eq!(cpu.ram.read(test_address), test_value);
    }

    #[test]
    fn test_0x03_inc_bc() {
        let test_value: u16 = 0xC0F4;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x03];
        cpu.load(&program);
        cpu.registers.set_bc(test_value - 1);
        cpu.execute_next();
        // NOP should not alter any register
        assert_eq!(cpu.registers.get_b(), 0xC0);
        assert_eq!(cpu.registers.get_c(), 0xF4);
        assert_eq!(cpu.registers.get_bc(), test_value);
    }

    #[test]
    fn test_0x04_inc_b() {
        //No Flags
        let test_value_1: u8 = 0b1111_0100;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x04];
        cpu_1.load(&program_1);
        cpu_1.registers.set_b(test_value_1 - 1);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        let program_2: Vec<u8> = vec![0x04];
        cpu_2.load(&program_2);
        cpu_2.registers.set_b(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_b(), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_2: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_2);
        cpu_2.registers.set_b(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_b(), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x05_dec_b() {
        //No Flags
        let test_value_1: u8 = 0xF4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x05];
        cpu_1.load(&program_1);
        cpu_1.registers.set_b(test_value_1 + 1);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_2: u8 = 0x00;
        let mut cpu_2 = CPU::new();
        let program_2: Vec<u8> = vec![0x05];
        cpu_2.load(&program_2);
        cpu_2.registers.set_b(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_b(), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags Z
        let test_value_3: u8 = 0x00;
        let mut cpu_3 = CPU::new();
        let program_3: Vec<u8> = vec![0x05];
        cpu_3.load(&program_3);
        cpu_3.registers.set_b(test_value_3 + 1);
        cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_b(), test_value_3);
        assert_eq!(cpu_3.registers.get_zero_flag(), true);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_4: u8 = 0xF0;
        cpu_3 = CPU::new();
        cpu_3.load(&program_3);
        cpu_3.registers.set_b(test_value_4);
        cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_b(), test_value_4 - 1);
        assert_eq!(cpu_3.registers.get_zero_flag(), false);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x06_ld_b_imm8() {
        //No Flags
        let test_value_1: u8 = 0xCD;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x06, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cpu_1.registers.get_b(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x07_rlca() {
        //No Flags
        let test_value_1: u8 = 0b1000_1000;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x07, 0x07];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cpu_1.registers.get_a(), 0b0001_0001);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_a(), 0b0010_0010);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    #[test]
    fn test_0x08_ld__a16__sp() {
        //No Flags
        let test_value_1: u16 = 0xBD89;
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x89;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x08, 0x89, (test_address_1 >> 8) as u8];
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let cycles = cpu_1.execute_next();
        // Check address and data are correctly used
        assert_eq!(cycles, 5);
        assert_eq!(cpu_1.registers.get_sp(), test_value_1);
        assert_eq!(cpu_1.ram.read(test_address_1), 0x89);
        assert_eq!(cpu_1.ram.read(test_address_1 + 1), (test_value_1 >> 8) as u8);
    }

    #[test]
    fn test_0x09_add_hl_bc() {
        //No Flags
        let mut test_value_1: u16 = 0xBD89;
        let mut test_value_2: u16 = 0x1029;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x09];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_bc(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x7000;
        test_value_2 = 0x9000;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_bc(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), 0);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        // H flags on ADD HL, rr should be on only carrying from bit 11 (check is made on H of HL)
        test_value_1 = 0x1070;
        test_value_2 = 0x1090;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_bc(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x1700;
        test_value_2 = 0x1900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_bc(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x9700;
        test_value_2 = 0x7900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_bc(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1.wrapping_add(test_value_2));
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x0a_ld_a__bc_() {
        let mut test_value_1: u8 = 0xBD;
        let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x0A];
        cpu_1.load(&program);
        cpu_1.registers.set_bc(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        cpu_1.registers.set_a(0x11); // Sure different from expected value
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.ram.read(test_address_1), test_value_1);
        assert_eq!(cpu_1.registers.get_bc(), test_address_1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
    }

    #[test]
    fn test_0x0b_dec_bc() {
        //No Flags
        let mut test_value_1: u16 = 0xBD89;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x0B];
        cpu_1.load(&program);
        cpu_1.registers.set_bc(test_value_1 + 1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_bc(), test_value_1);
    }

    #[test]
    fn test_0x0c_inc_c() {
        //No Flags
        let test_value_1: u8 = 0b1111_0100;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x0C];
        cpu_1.load(&program_1);
        cpu_1.registers.set_c(test_value_1 - 1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_c(test_value_2);
        cycles = cpu_2.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_2.registers.get_c(), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_2: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_c(test_value_2);
        cycles = cpu_2.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_2.registers.get_c(), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x0d_dec_c() {
        //No Flags
        let test_value_1: u8 = 0xF4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x0D];
        cpu_1.load(&program_1);
        cpu_1.registers.set_c(test_value_1 + 1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_2: u8 = 0x00;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_c(test_value_2);
        cycles = cpu_2.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_2.registers.get_c(), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags Z
        let test_value_3: u8 = 0x00;
        let mut cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_c(test_value_3 + 1);
        cycles = cpu_3.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_3.registers.get_c(), test_value_3);
        assert_eq!(cpu_3.registers.get_zero_flag(), true);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_4: u8 = 0xF0;
        cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_c(test_value_4);
        cycles = cpu_3.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_3.registers.get_c(), test_value_4 - 1);
        assert_eq!(cpu_3.registers.get_zero_flag(), false);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x0e_ld_c_imm8() {
        //No Flags
        let test_value_1: u8 = 0xD4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x0E, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_c(0xAA);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_c(), test_value_1);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[cfg(test)]
    fn test_0x10_stop() {
        // TODO: Study and implement STOP function
    }

    #[test]
    fn test_0x11_ld_de_imm16() {
        let test_value: u16 = 0xC05A;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x11, 0x5A, 0xC0];
        cpu.load(&program);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_d(), 0xC0);
        assert_eq!(cpu.registers.get_e(), 0x5A);
        assert_eq!(cpu.registers.get_de(), test_value);
    }

    #[test]
    fn test_0x12_ld__de__a() {
        let test_value: u8 = 0xF4;
        let test_address: u16 = RAM::WRAM_ADDRESS as u16 + 0x0500;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x12];
        cpu.load(&program);
        cpu.registers.set_a(test_value);
        cpu.registers.set_de(test_address);
        cpu.ram.write(test_address, 0x00);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_a(), test_value);
        assert_eq!(cpu.registers.get_de(), test_address);
        assert_eq!(cpu.ram.read(test_address), test_value);
    }

    #[test]
    fn test_0x13_inc_de() {
        let test_value: u16 = 0xC0F4;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x13];
        cpu.load(&program);
        cpu.registers.set_de(test_value - 1);
        cpu.execute_next();
        // NOP should not alter any register
        assert_eq!(cpu.registers.get_d(), 0xC0);
        assert_eq!(cpu.registers.get_e(), 0xF4);
        assert_eq!(cpu.registers.get_de(), test_value);
    }

    #[test]
    fn test_0x14_inc_d() {
        //No Flags
        let test_value_1: u8 = 0b1111_0100;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x14];
        cpu_1.load(&program_1);
        cpu_1.registers.set_d(test_value_1 - 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_d(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_d(), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_2: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_d(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_d(), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x15_dec_d() {
        //No Flags
        let test_value_1: u8 = 0xF4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x15];
        cpu_1.load(&program_1);
        cpu_1.registers.set_d(test_value_1 + 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_2: u8 = 0x00;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_d(test_value_2);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_d(), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags Z
        let test_value_3: u8 = 0x00;
        let mut cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_d(test_value_3 + 1);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_d(), test_value_3);
        assert_eq!(cpu_3.registers.get_zero_flag(), true);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_4: u8 = 0xF0;
        cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_d(test_value_4);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_d(), test_value_4 - 1);
        assert_eq!(cpu_3.registers.get_zero_flag(), false);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x16_ld_d_imm8() {
        //No Flags
        let test_value_1: u8 = 0xCD;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x16, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        let cycles = cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x21_ld_hl_imm16() {
        let test_value: u16 = 0xC05A;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x21, 0x5A, 0xC0];
        cpu.load(&program);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_h(), 0xC0);
        assert_eq!(cpu.registers.get_l(), 0x5A);
        assert_eq!(cpu.registers.get_hl(), test_value);
    }

    #[test]
    fn test_0x22_ld__hli__a() {
        let test_value: u8 = 0xF4;
        let test_address: u16 = RAM::WRAM_ADDRESS as u16 + 0x0500;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x22];
        cpu.load(&program);
        cpu.registers.set_a(test_value);
        cpu.registers.set_hl(test_address);
        cpu.ram.write(test_address, 0x00);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_a(), test_value);
        assert_eq!(cpu.registers.get_hl(), test_address + 1);
        assert_eq!(cpu.ram.read(test_address), test_value);
    }

    #[test]
    fn test_0x23_inc_de() {
        let test_value: u16 = 0xC0F4;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x23];
        cpu.load(&program);
        cpu.registers.set_hl(test_value - 1);
        cpu.execute_next();
        assert_eq!(cpu.registers.get_h(), 0xC0);
        assert_eq!(cpu.registers.get_l(), 0xF4);
        assert_eq!(cpu.registers.get_hl(), test_value);
    }

    #[test]
    fn test_0x24_inc_h() {
        // No Flags
        let test_value_1: u8 = 0b1111_0100;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x24];
        cpu_1.load(&program_1);
        cpu_1.registers.set_h(test_value_1 - 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_h(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_d(), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_2: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_h(test_value_2);
        cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_d(), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x25_dec_h() {
        //No Flags
        let test_value_1: u8 = 0xF4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x25];
        cpu_1.load(&program_1);
        cpu_1.registers.set_d(test_value_1 + 1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 1);
        assert_eq!(cpu_1.registers.get_d(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_2: u8 = 0x00;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_d(test_value_2);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.registers.get_d(), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags Z
        let test_value_3: u8 = 0x00;
        let mut cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_d(test_value_3 + 1);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_d(), test_value_3);
        assert_eq!(cpu_3.registers.get_zero_flag(), true);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_4: u8 = 0xF0;
        cpu_3 = CPU::new();
        cpu_3.load(&program_1);
        cpu_3.registers.set_d(test_value_4);
        cycle = cpu_3.execute_next();
        assert_eq!(cpu_3.registers.get_d(), test_value_4 - 1);
        assert_eq!(cpu_3.registers.get_zero_flag(), false);
        assert_eq!(cpu_3.registers.get_negative_flag(), true);
        assert_eq!(cpu_3.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x26_ld_h_imm8() {
        //No Flags
        let test_value_1: u8 = 0xCD;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x26, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        let cycles = cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x31_ld_sp_imm16() {
        let test_value: u16 = 0xC05A;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x31, 0x5A, 0xC0];
        cpu.load(&program);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_sp(), test_value);
    }

    #[test]
    fn test_0x32_ld__hld__a() {
        let test_value: u8 = 0xF4;
        let test_address: u16 = RAM::WRAM_ADDRESS as u16 + 0x0500;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x32];
        cpu.load(&program);
        cpu.registers.set_a(test_value);
        cpu.registers.set_hl(test_address);
        cpu.ram.write(test_address, 0x00);
        let cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_a(), test_value);
        assert_eq!(cpu.registers.get_hl(), test_address - 1);
        assert_eq!(cpu.ram.read(test_address), test_value);
    }

    #[test]
    fn test_0x33_inc_de() {
        let test_value: u16 = 0xC0F4;
        let mut cpu = CPU::new();
        let program: Vec<u8> = vec![0x33];
        cpu.load(&program);
        cpu.registers.set_sp(test_value - 1);
        cpu.execute_next();
        assert_eq!(cpu.registers.get_sp(), test_value);
    }

    #[test]
    fn test_0x34_inc__hl_() {
        // No Flags
        let test_value_1: u8 = 0b1111_0100;
        let test_address = WRAM_ADDRESS as u16 + 0x50;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x34];
        cpu_1.load(&program_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 3);
        assert_eq!(cpu_1.ram.read(test_address), test_value_1 + 1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z/H
        let test_value_2: u8 = 0xFF;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_hl(test_address);
        cpu_2.ram.write(test_address, test_value_2);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.ram.read(test_address), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Flags H
        let test_value_3: u8 = 0x0F;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_hl(test_address);
        cpu_2.ram.write(test_address, test_value_3);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.ram.read(test_address), 0x10);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x35_dec__hl_() {
        // No Flags
        let test_value_1: u8 = 0b1111_0100;
        let test_address = WRAM_ADDRESS as u16 + 0x50;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x35];
        cpu_1.load(&program_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_1);
        let mut cycle = cpu_1.execute_next();
        assert_eq!(cycle, 3);
        assert_eq!(cpu_1.ram.read(test_address), test_value_1 - 1);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);

        // Flags Z
        let test_value_2: u8 = 0x01;
        let mut cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_hl(test_address);
        cpu_2.ram.write(test_address, test_value_2);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.ram.read(test_address), 0);
        assert_eq!(cpu_2.registers.get_zero_flag(), true);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), false);

        // Flags H
        let test_value_3: u8 = 0xF0;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_hl(test_address);
        cpu_2.ram.write(test_address, test_value_3);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.ram.read(test_address), test_value_3 - 1);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Test Underflow
        let test_value_4: u8 = 0x00;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_hl(test_address);
        cpu_2.ram.write(test_address, test_value_3);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.ram.read(test_address), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), false);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);
    }

    #[test]
    fn test_0x36_ld__hl__imm8() {
        //No Flags
        let test_value_1: u8 = 0xCD;
        let test_address: u16 = WRAM_ADDRESS as u16 + 0x88;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x36, test_value_1];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, 0x00);
        let cycles = cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cycles, 3);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        assert_eq!(cpu_1.ram.read(test_address), test_value_1);
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }
}