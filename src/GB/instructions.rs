use crate::GB::CPU::CPU;
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
            cpu.registers.set_half_carry_flag(cpu.registers.get_b() < original_b);
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
            cpu.registers.set_half_carry_flag(cpu.registers.get_b() > original_b);
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
            // TODO: Test
            cpu.registers.set_carry_flag((cpu.registers.get_a() & 0b1000_0000) != 0);
            cpu.registers.set_a(cpu.registers.get_a().wrapping_shl(1) | cpu.registers.get_a().wrapping_shr(7));
            cpu.registers.set_zero_flag(true);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_negative_flag(true);
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
            cpu.ram.write_wram(imm16_address, (cpu.registers.get_sp() & 0xFF) as u8);
            cpu.ram.write_wram(imm16_address + 1, (cpu.registers.get_sp() >> 8) as u8);
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
            let pre_h = cpu.registers.get_h();
            let pre_l = cpu.registers.get_l();
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(cpu.registers.get_bc()));
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(cpu.registers.get_l() < pre_l);
            cpu.registers.set_carry_flag(cpu.registers.get_h() < pre_h);
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
            cpu.registers.set_half_carry_flag(original_c > cpu.registers.get_c());
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
            cpu.registers.set_half_carry_flag(original_c < cpu.registers.get_c());
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
            cpu.registers.set_half_carry_flag(cpu.registers.get_d() < original_d);
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
