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

const fn daa(mut a: u8, mut flags: u8) -> (u8, u8) {
    // Code recovered inspired by other gits, but not sure if precise as expected from GB Docs
    let mut adjust = 0;
    let mut carry_flag = (FlagBits::C as u8) & flags != 0;
    let mut half_carry_flag = (FlagBits::H as u8) & flags != 0;

    if half_carry_flag {
        adjust |= 0x06;
    }
    if carry_flag || a > 0x99 {
        adjust |= 0x60;
    }

    // Edit register A for CDB representation
    if (FlagBits::N as u8) & flags == 0 {
        if (a & 0x0F )> 9 {
            adjust |= 0x06;
        }

        if a > 0x99 {
            adjust |= 0x60;
        }

        a = a.wrapping_add(adjust);
    } else {
        a = a.wrapping_sub(adjust);
    }

    carry_flag = (adjust & 0x60) != 0;
    let zero_flag = a == 0;

    // Impostare i flag appropriati
    if carry_flag {
        flags |= (FlagBits::C as u8);
    } else {
        flags &= !(FlagBits::C as u8);
    }

    if zero_flag {
        flags |= (FlagBits::Z as u8);
    } else {
        flags &= !(FlagBits::Z as u8);
    }

    flags &= !(FlagBits::H as u8);

    // Settare i nuovi valori dei flag
    // flags può essere un riferimento mutabile a un altro registro che contiene i flags
    // quindi è necessario modificarlo come desiderato
    (a, flags)
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
            cpu.registers.set_d(cpu.registers.get_d().wrapping_add(1));
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
            cpu.registers.set_half_carry_flag((cpu.registers.get_d() & 0x0F) > (original_d & 0x0F));
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
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x18] = Some(&Instruction {
        opcode: 0x18,
        name: "JR e8",
        cycles: 3,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
            if byte > 0 {
                cpu.registers.set_pc(cpu.registers.get_pc().wrapping_add(byte as u16));
            } else {
                cpu.registers.set_pc(cpu.registers.get_pc().wrapping_sub((byte as i16).abs() as u16));
            }
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
            cpu.registers.set_half_carry_flag((cpu.registers.get_h() & 0x0F) < (pre_h & 0x0F));
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
            cpu.registers.set_half_carry_flag((cpu.registers.get_e() & 0x0F) < (original_e & 0x0F));
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
            cpu.registers.set_half_carry_flag((cpu.registers.get_e() & 0x0F) > (original_e & 0x0F));
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
            let byte = cpu.fetch_next() as i8;
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
            cpu.registers.set_h(cpu.registers.get_h().wrapping_add(1));
            cpu.registers.set_half_carry_flag((cpu.registers.get_h() & 0x0F) < (original_h & 0x0F));
            cpu.registers.set_zero_flag(cpu.registers.get_h() == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x25] = Some(&Instruction {
        opcode: 0x25,
        name: "DEC H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_h = cpu.registers.get_h();
            cpu.registers.set_h(cpu.registers.get_h().wrapping_sub(1));
            // Write flags
            cpu.registers.set_half_carry_flag((original_h & 0x0F) == 0);
            cpu.registers.set_zero_flag(cpu.registers.get_h() == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x26] = Some(&Instruction {
        opcode: 0x26,
        name: "LD H, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_h(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x27] = Some(&Instruction {
        opcode: 0x27,
        name: "DAA",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let (a, flags) = daa(cpu.registers.get_a(), cpu.registers.get_f());
            cpu.registers.set_a(a);
            cpu.registers.set_f(flags);
            opcode.cycles as u64
        },
    });
    opcodes[0x28] = Some(&Instruction {
        opcode: 0x28,
        name: "JR Z, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
            if cpu.registers.get_zero_flag() {
                cpu.registers.set_pc(cpu.registers.get_pc() - if byte < 0 { byte.abs() } else { byte } as u16);
                return opcode.cycles as u64;
            }
            2
        },
    });
    opcodes[0x29] = Some(&Instruction {
        opcode: 0x29,
        name: "ADD HL, HL",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let pre_h = cpu.registers.get_h();
            let pre_l = cpu.registers.get_l();
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(cpu.registers.get_hl()));
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((cpu.registers.get_h() & 0x0F) < (pre_h & 0x0F));
            cpu.registers.set_carry_flag(cpu.registers.get_h() < pre_h);
            opcode.cycles as u64
        },
    });
    opcodes[0x2A] = Some(&Instruction {
        opcode: 0x3A,
        name: "LD A, [HL+]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.ram.read(cpu.registers.get_hl()));
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x2B] = Some(&Instruction {
        opcode: 0x2B,
        name: "DEC HL",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_sub(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x2C] = Some(&Instruction {
        opcode: 0x2C,
        name: "INC L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_l = cpu.registers.get_l();
            cpu.registers.set_l(cpu.registers.get_l().wrapping_add(1));
            cpu.registers.set_half_carry_flag((cpu.registers.get_l() & 0x0F) < (original_l & 0x0F));
            cpu.registers.set_zero_flag(cpu.registers.get_l() == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x2D] = Some(&Instruction {
        opcode: 0x2D,
        name: "DEC L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_l = cpu.registers.get_l();
            cpu.registers.set_l(cpu.registers.get_l().wrapping_sub(1));
            // Write flags
            cpu.registers.set_half_carry_flag((original_l & 0x0F) == 0);
            cpu.registers.set_zero_flag(cpu.registers.get_l() == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x2E] = Some(&Instruction {
        opcode: 0x2E,
        name: "LD L, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_l(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x2F] = Some(&Instruction {
        opcode: 0x2F,
        name: "CPL",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(!cpu.registers.get_a());
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x30] = Some(&Instruction {
        opcode: 0x20,
        name: "JR NC, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
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
    opcodes[0x34] = Some(&Instruction {
        opcode: 0x34,
        name: "INC [HL]",
        cycles: 3,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_hl_ram = cpu.ram.read(cpu.registers.get_hl());
            cpu.ram.write(cpu.registers.get_hl(), original_hl_ram.wrapping_add(1));
            cpu.registers.set_half_carry_flag((cpu.ram.read(cpu.registers.get_hl()) & 0x0F) < (original_hl_ram & 0x0F));
            cpu.registers.set_zero_flag(cpu.ram.read(cpu.registers.get_hl()) == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x35] = Some(&Instruction {
        opcode: 0x35,
        name: "DEC [HL]",
        cycles: 3,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_byte = cpu.ram.read(cpu.registers.get_hl());
            cpu.ram.write(cpu.registers.get_hl(), original_byte.wrapping_sub(1));
            cpu.registers.set_half_carry_flag((cpu.ram.read(cpu.registers.get_hl()) & 0x0F) > (original_byte & 0x0F));
            cpu.registers.set_zero_flag(cpu.ram.read(cpu.registers.get_hl()) == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x36] = Some(&Instruction {
        opcode: 0x36,
        name: "LD [HL], imm8",
        cycles: 3,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.ram.write(cpu.registers.get_hl(), byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x37] = Some(&Instruction {
        opcode: 0x37,
        name: "SCF",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_carry_flag(true);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x38] = Some(&Instruction {
        opcode: 0x38,
        name: "JR C, e8",
        cycles: 3, // 2 Cycles if condition doesn't match
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: Test
            let byte = cpu.fetch_next() as i8;
            if cpu.registers.get_carry_flag() {
                cpu.registers.set_pc(cpu.registers.get_pc() - if byte < 0 { byte.abs() } else { byte } as u16);
                return opcode.cycles as u64;
            }
            2
        },
    });
    opcodes[0x39] = Some(&Instruction {
        opcode: 0x39,
        name: "ADD HL, SP",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let pre_h = cpu.registers.get_h();
            let pre_l = cpu.registers.get_l();
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(cpu.registers.get_sp()));
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((cpu.registers.get_h() & 0x0F) < (pre_h & 0x0F));
            cpu.registers.set_carry_flag(cpu.registers.get_h() < pre_h);
            opcode.cycles as u64
        },
    });
    opcodes[0x3A] = Some(&Instruction {
        opcode: 0x3A,
        name: "LD A, [HL-]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.ram.read(cpu.registers.get_hl()));
            cpu.registers.set_hl(cpu.registers.get_hl().wrapping_sub(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x3B] = Some(&Instruction {
        opcode: 0x3B,
        name: "DEC SP",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_sp(cpu.registers.get_sp().wrapping_sub(1));
            opcode.cycles as u64
        },
    });
    opcodes[0x3C] = Some(&Instruction {
        opcode: 0x3C,
        name: "INC A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_a = cpu.registers.get_a();
            cpu.registers.set_a(cpu.registers.get_a().wrapping_add(1));
            cpu.registers.set_half_carry_flag((cpu.registers.get_a() & 0x0F) < (original_a & 0x0F));
            cpu.registers.set_zero_flag(cpu.registers.get_a() == 0);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0x3D] = Some(&Instruction {
        opcode: 0x3D,
        name: "DEC A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let original_a = cpu.registers.get_a();
            cpu.registers.set_a(cpu.registers.get_a().wrapping_sub(1));
            // Write flags
            cpu.registers.set_half_carry_flag((original_a & 0x0F) == 0);
            cpu.registers.set_zero_flag(cpu.registers.get_a() == 0);
            cpu.registers.set_negative_flag(true);
            opcode.cycles as u64
        },
    });
    opcodes[0x3E] = Some(&Instruction {
        opcode: 0x3E,
        name: "LD A, imm8",
        cycles: 2,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            cpu.registers.set_a(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0x3F] = Some(&Instruction {
        opcode: 0x3F,
        name: "CCF",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(!cpu.registers.get_carry_flag());
            opcode.cycles as u64
        },
    });
    opcodes[0x40] = Some(&Instruction {
        opcode: 0x40,
        name: "LD B, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x41] = Some(&Instruction {
        opcode: 0x41,
        name: "LD B, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x42] = Some(&Instruction {
        opcode: 0x42,
        name: "LD B, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x43] = Some(&Instruction {
        opcode: 0x43,
        name: "LD B, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x44] = Some(&Instruction {
        opcode: 0x44,
        name: "LD B, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x45] = Some(&Instruction {
        opcode: 0x45,
        name: "LD B, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x46] = Some(&Instruction {
        opcode: 0x46,
        name: "LD B, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x47] = Some(&Instruction {
        opcode: 0x47,
        name: "LD B, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_b(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x48] = Some(&Instruction {
        opcode: 0x48,
        name: "LD C, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x49] = Some(&Instruction {
        opcode: 0x49,
        name: "LD C, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x4A] = Some(&Instruction {
        opcode: 0x4A,
        name: "LD C, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x4B] = Some(&Instruction {
        opcode: 0x4B,
        name: "LD C, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x4C] = Some(&Instruction {
        opcode: 0x4C,
        name: "LD C, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x4D] = Some(&Instruction {
        opcode: 0x4D,
        name: "LD C, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x4E] = Some(&Instruction {
        opcode: 0x4E,
        name: "LD C, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x4F] = Some(&Instruction {
        opcode: 0x4F,
        name: "LD C, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_c(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x50] = Some(&Instruction {
        opcode: 0x50,
        name: "LD D, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x51] = Some(&Instruction {
        opcode: 0x51,
        name: "LD D, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x52] = Some(&Instruction {
        opcode: 0x52,
        name: "LD D, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x53] = Some(&Instruction {
        opcode: 0x53,
        name: "LD D, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x54] = Some(&Instruction {
        opcode: 0x54,
        name: "LD D, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x55] = Some(&Instruction {
        opcode: 0x55,
        name: "LD D, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x56] = Some(&Instruction {
        opcode: 0x56,
        name: "LD D, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x57] = Some(&Instruction {
        opcode: 0x57,
        name: "LD D, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_d(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x58] = Some(&Instruction {
        opcode: 0x58,
        name: "LD E, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x59] = Some(&Instruction {
        opcode: 0x59,
        name: "LD E, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x5A] = Some(&Instruction {
        opcode: 0x5A,
        name: "LD E, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x5B] = Some(&Instruction {
        opcode: 0x5B,
        name: "LD D, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x5C] = Some(&Instruction {
        opcode: 0x5C,
        name: "LD E, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x5D] = Some(&Instruction {
        opcode: 0x5D,
        name: "LD E, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x5E] = Some(&Instruction {
        opcode: 0x5E,
        name: "LD E, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x5F] = Some(&Instruction {
        opcode: 0x5F,
        name: "LD E, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_e(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x60] = Some(&Instruction {
        opcode: 0x60,
        name: "LD H, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x61] = Some(&Instruction {
        opcode: 0x61,
        name: "LD H, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x62] = Some(&Instruction {
        opcode: 0x62,
        name: "LD H, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x63] = Some(&Instruction {
        opcode: 0x63,
        name: "LD H, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x64] = Some(&Instruction {
        opcode: 0x64,
        name: "LD H, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x65] = Some(&Instruction {
        opcode: 0x65,
        name: "LD H, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x66] = Some(&Instruction {
        opcode: 0x66,
        name: "LD H, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x67] = Some(&Instruction {
        opcode: 0x67,
        name: "LD H, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_h(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x68] = Some(&Instruction {
        opcode: 0x68,
        name: "LD L, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x69] = Some(&Instruction {
        opcode: 0x69,
        name: "LD L, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x6A] = Some(&Instruction {
        opcode: 0x6A,
        name: "LD L, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x6B] = Some(&Instruction {
        opcode: 0x6B,
        name: "LD L, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x6C] = Some(&Instruction {
        opcode: 0x6C,
        name: "LD L, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x6D] = Some(&Instruction {
        opcode: 0x6D,
        name: "LD L, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x6E] = Some(&Instruction {
        opcode: 0x6E,
        name: "LD L, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x6F] = Some(&Instruction {
        opcode: 0x6F,
        name: "LD L, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_l(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x70] = Some(&Instruction {
        opcode: 0x70,
        name: "LD [HL], B",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x71] = Some(&Instruction {
        opcode: 0x71,
        name: "LD [HL], C",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x72] = Some(&Instruction {
        opcode: 0x72,
        name: "LD [HL], D",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x73] = Some(&Instruction {
        opcode: 0x73,
        name: "LD [HL], E",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x74] = Some(&Instruction {
        opcode: 0x74,
        name: "LD [HL], H",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x75] = Some(&Instruction {
        opcode: 0x75,
        name: "LD [HL], L",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x76] = Some(&Instruction {
        opcode: 0x76,
        name: "HALT",  // LD [HL], [HL] Decode as HALT instruction on DMG/GBC
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            // TODO: implement
            opcode.cycles as u64
        },
    });
    opcodes[0x77] = Some(&Instruction {
        opcode: 0x77,
        name: "LD [HL], A",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.ram.write(cpu.registers.get_hl(), cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x78] = Some(&Instruction {
        opcode: 0x78,
        name: "LD A, B",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_b());
            opcode.cycles as u64
        },
    });
    opcodes[0x79] = Some(&Instruction {
        opcode: 0x79,
        name: "LD A, C",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0x7A] = Some(&Instruction {
        opcode: 0x7A,
        name: "LD A, D",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_d());
            opcode.cycles as u64
        },
    });
    opcodes[0x7B] = Some(&Instruction {
        opcode: 0x7B,
        name: "LD A, E",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0x7C] = Some(&Instruction {
        opcode: 0x7C,
        name: "LD A, H",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_h());
            opcode.cycles as u64
        },
    });
    opcodes[0x7D] = Some(&Instruction {
        opcode: 0x7D,
        name: "LD A, L",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0x7E] = Some(&Instruction {
        opcode: 0x7E,
        name: "LD A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.ram.read(cpu.registers.get_hl()));
            opcode.cycles as u64
        },
    });
    opcodes[0x7F] = Some(&Instruction {
        opcode: 0x7F,
        name: "LD A, A",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_a(cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0x80] = Some(&Instruction {
        opcode: 0x80,
        name: "ADD A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_b());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x81] = Some(&Instruction {
        opcode: 0x81,
        name: "ADD A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_c());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x82] = Some(&Instruction {
        opcode: 0x82,
        name: "ADD A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_d());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x83] = Some(&Instruction {
        opcode: 0x83,
        name: "ADD A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_e());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x84] = Some(&Instruction {
        opcode: 0x84,
        name: "ADD A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_h());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x85] = Some(&Instruction {
        opcode: 0x85,
        name: "ADD A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.registers.get_l());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x86] = Some(&Instruction {
        opcode: 0x86,
        name: "ADD A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(cpu.ram.read(cpu.registers.get_hl()));
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x87] = Some(&Instruction {
        opcode: 0x87,
        name: "ADD A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(old_value);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x88] = Some(&Instruction {
        opcode: 0x88,
        name: "ADC A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_b());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x89] = Some(&Instruction {
        opcode: 0x88,
        name: "ADC A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_c());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8A] = Some(&Instruction {
        opcode: 0x8A,
        name: "ADC A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_d());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8B] = Some(&Instruction {
        opcode: 0x8B,
        name: "ADC A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_e());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8C] = Some(&Instruction {
        opcode: 0x8C,
        name: "ADC A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_h());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8D] = Some(&Instruction {
        opcode: 0x8D,
        name: "ADC A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_l());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8E] = Some(&Instruction {
        opcode: 0x8E,
        name: "ADC A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.ram.read(cpu.registers.get_hl()));
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x8F] = Some(&Instruction {
        opcode: 0x8F,
        name: "ADC A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(cpu.registers.get_a());
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x90] = Some(&Instruction {
        opcode: 0x90,
        name: "SUB A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_b());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x91] = Some(&Instruction {
        opcode: 0x91,
        name: "SUB A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_c());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x92] = Some(&Instruction {
        opcode: 0x92,
        name: "SUB A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_d());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x93] = Some(&Instruction {
        opcode: 0x93,
        name: "SUB A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_e());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x94] = Some(&Instruction {
        opcode: 0x94,
        name: "SUB A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_h());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x95] = Some(&Instruction {
        opcode: 0x95,
        name: "SUB A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_l());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x96] = Some(&Instruction {
        opcode: 0x96,
        name: "SUB A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.ram.read(cpu.registers.get_hl()));
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x97] = Some(&Instruction {
        opcode: 0x97,
        name: "SUB A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_a());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x98] = Some(&Instruction {
        opcode: 0x98,
        name: "SBC A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_b());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x99] = Some(&Instruction {
        opcode: 0x99,
        name: "SBC A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_c());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9A] = Some(&Instruction {
        opcode: 0x9A,
        name: "SBC A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_d());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9B] = Some(&Instruction {
        opcode: 0x9B,
        name: "SBC A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_e());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9C] = Some(&Instruction {
        opcode: 0x9C,
        name: "SBC A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_h());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9D] = Some(&Instruction {
        opcode: 0x9D,
        name: "SBC A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_l());
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9E] = Some(&Instruction {
        opcode: 0x9E,
        name: "SBC A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.ram.read(cpu.registers.get_hl()));
            let new_value = intra_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0x9F] = Some(&Instruction {
        opcode: 0x9F,
        name: "SBC A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let intra_value = old_value.wrapping_sub(cpu.registers.get_a());
            let new_value = intra_value.wrapping_sub(intra_value)
                .wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(
                ((new_value & 0x0F) > (intra_value & 0x0F)) || ((intra_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xA0] = Some(&Instruction {
        opcode: 0xA0,
        name: "AND A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_b();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA1] = Some(&Instruction {
        opcode: 0xA1,
        name: "AND A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_c();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA2] = Some(&Instruction {
        opcode: 0xA2,
        name: "AND A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_d();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA3] = Some(&Instruction {
        opcode: 0xA3,
        name: "AND A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_e();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA4] = Some(&Instruction {
        opcode: 0xA4,
        name: "AND A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_h();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA5] = Some(&Instruction {
        opcode: 0xA5,
        name: "AND A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.registers.get_l();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA6] = Some(&Instruction {
        opcode: 0xA6,
        name: "AND A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.ram.read(cpu.registers.get_hl());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA7] = Some(&Instruction {
        opcode: 0xA7,
        name: "AND A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & old_value;
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA8] = Some(&Instruction {
        opcode: 0xA8,
        name: "XOR A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_b();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xA9] = Some(&Instruction {
        opcode: 0xA9,
        name: "XOR A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_c();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAA] = Some(&Instruction {
        opcode: 0xAA,
        name: "XOR A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_d();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAB] = Some(&Instruction {
        opcode: 0xAB,
        name: "XOR A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_e();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAC] = Some(&Instruction {
        opcode: 0xAC,
        name: "XOR A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_h();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAD] = Some(&Instruction {
        opcode: 0xAD,
        name: "XOR A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.registers.get_l();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAE] = Some(&Instruction {
        opcode: 0xAE,
        name: "XOR A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.ram.read(cpu.registers.get_hl());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xAF] = Some(&Instruction {
        opcode: 0xAF,
        name: "XOR A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ old_value;
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB0] = Some(&Instruction {
        opcode: 0xB0,
        name: "OR A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_b();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB1] = Some(&Instruction {
        opcode: 0xB1,
        name: "OR A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_c();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB2] = Some(&Instruction {
        opcode: 0xB2,
        name: "OR A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_d();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB3] = Some(&Instruction {
        opcode: 0xB3,
        name: "OR A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_e();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB4] = Some(&Instruction {
        opcode: 0xB4,
        name: "OR A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_h();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB5] = Some(&Instruction {
        opcode: 0xB5,
        name: "OR A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.registers.get_l();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB6] = Some(&Instruction {
        opcode: 0xB6,
        name: "OR A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.ram.read(cpu.registers.get_hl());
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB7] = Some(&Instruction {
        opcode: 0xB7,
        name: "OR A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | old_value;
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xB8] = Some(&Instruction {
        opcode: 0xB8,
        name: "CP A, B",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_b());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xB9] = Some(&Instruction {
        opcode: 0xB9,
        name: "CP A, C",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_c());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBA] = Some(&Instruction {
        opcode: 0xBA,
        name: "CP A, D",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_d());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBB] = Some(&Instruction {
        opcode: 0xBB,
        name: "CP A, E",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_e());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBC] = Some(&Instruction {
        opcode: 0xBC,
        name: "CP A, H",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_h());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBD] = Some(&Instruction {
        opcode: 0xBD,
        name: "CP A, L",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_l());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBE] = Some(&Instruction {
        opcode: 0xBE,
        name: "CP A, [HL]",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.ram.read(cpu.registers.get_hl()));
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xBF] = Some(&Instruction {
        opcode: 0xBF,
        name: "CP A, A",
        cycles: 1,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(cpu.registers.get_a());
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xC0] = Some(&Instruction {
        opcode: 0xC0,
        name: "RET NZ",
        cycles: 5,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            if !cpu.registers.get_zero_flag() {
                let mut return_address: u16 = 0x00;
                return_address |= cpu.pop() as u16;
                return_address |= (cpu.pop() as u16) << 8;
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            2
        },
    });
    opcodes[0xC1] = Some(&Instruction {
        opcode: 0xC1,
        name: "POP BC",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut byte = cpu.pop();
            cpu.registers.set_c(byte);
            byte = cpu.pop();
            cpu.registers.set_b(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0xC2] = Some(&Instruction {
        opcode: 0xC2,
        name: "JP NZ, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_address: u16 = 0x00;
            return_address |= cpu.fetch_next() as u16;
            return_address |= (cpu.fetch_next() as u16) << 8;
            if !cpu.registers.get_zero_flag() {
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xC3] = Some(&Instruction {
        opcode: 0xC3,
        name: "JP imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_address: u16 = 0x00;
            return_address |= cpu.fetch_next() as u16;
            return_address |= (cpu.fetch_next() as u16) << 8;
            cpu.registers.set_pc(return_address);
            opcode.cycles as u64
        },
    });
    opcodes[0xC4] = Some(&Instruction {
        opcode: 0xC4,
        name: "CALL NZ, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut call_address: u16 = 0x00;
            call_address |= cpu.fetch_next() as u16;
            call_address |= (cpu.fetch_next() as u16) << 8;
            if !cpu.registers.get_zero_flag() {
                let return_address = cpu.registers.get_pc();
                cpu.push((return_address >> 8) as u8);
                cpu.push((return_address & 0xFF) as u8);
                cpu.registers.set_pc(call_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xC5] = Some(&Instruction {
        opcode: 0xC5,
        name: "PUSH BC",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.push(cpu.registers.get_b());
            cpu.push(cpu.registers.get_c());
            opcode.cycles as u64
        },
    });
    opcodes[0xC6] = Some(&Instruction {
        opcode: 0xC6,
        name: "ADD A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_add(byte);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) < (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xC7] = Some(&Instruction {
        opcode: 0xC7,
        name: "RST $00",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xC8] = Some(&Instruction {
        opcode: 0xC8,
        name: "RET Z",
        cycles: 5,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            if cpu.registers.get_zero_flag() {
                let mut return_addr = cpu.pop() as u16;
                return_addr |= (cpu.pop() as u16) << 8;
                cpu.registers.set_pc(return_addr);
                return opcode.cycles as u64
            }
            2
        },
    });
    opcodes[0xC9] = Some(&Instruction {
        opcode: 0xC9,
        name: "RET",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_addr = cpu.pop() as u16;
            return_addr |= (cpu.pop() as u16) << 8;
            cpu.registers.set_pc(return_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xCA] = Some(&Instruction {
        opcode: 0xCA,
        name: "JP Z, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_address: u16 = 0x00;
            return_address |= cpu.fetch_next() as u16;
            return_address |= (cpu.fetch_next() as u16) << 8;
            if cpu.registers.get_zero_flag() {
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            3
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
            cpu.execute_next()
        },
    });
    opcodes[0xCC] = Some(&Instruction {
        opcode: 0xCC,
        name: "CALL Z, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut call_address: u16 = 0x00;
            call_address |= cpu.fetch_next() as u16;
            call_address |= (cpu.fetch_next() as u16) << 8;
            if cpu.registers.get_zero_flag() {
                cpu.push((cpu.registers.get_pc() >> 8) as u8);
                cpu.push((cpu.registers.get_pc() & 0xFF) as u8);
                cpu.registers.set_pc(call_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xCD] = Some(&Instruction {
        opcode: 0xCD,
        name: "CALL imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut call_address: u16 = 0x00;
            call_address |= cpu.fetch_next() as u16;
            call_address |= (cpu.fetch_next() as u16) << 8;
            cpu.push((cpu.registers.get_pc() >> 8) as u8);
            cpu.push((cpu.registers.get_pc() & 0xFF) as u8);
            cpu.registers.set_pc(call_address);
            return opcode.cycles as u64
        },
    });
    opcodes[0xCE] = Some(&Instruction {
        opcode: 0xCE,
        name: "ADC A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_add(byte);
            let new_value = intermediate_value.wrapping_add(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) < (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) < (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value < old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xCF] = Some(&Instruction {
        opcode: 0xCF,
        name: "RST $08",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xD0] = Some(&Instruction {
        opcode: 0xD0,
        name: "RET CZ",
        cycles: 5,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            if !cpu.registers.get_carry_flag() {
                let mut return_address: u16 = 0x00;
                return_address |= cpu.pop() as u16;
                return_address |= (cpu.pop() as u16) << 8;
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            2
        },
    });
    opcodes[0xD1] = Some(&Instruction {
        opcode: 0xD1,
        name: "POP DE",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut byte = cpu.pop();
            cpu.registers.set_e(byte);
            byte = cpu.pop();
            cpu.registers.set_d(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0xD2] = Some(&Instruction {
        opcode: 0xD2,
        name: "JP NC, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_address: u16 = 0x00;
            return_address |= cpu.fetch_next() as u16;
            return_address |= (cpu.fetch_next() as u16) << 8;
            if !cpu.registers.get_carry_flag() {
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xD4] = Some(&Instruction {
        opcode: 0xD4,
        name: "CALL NC, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut call_address: u16 = 0x00;
            call_address |= cpu.fetch_next() as u16;
            call_address |= (cpu.fetch_next() as u16) << 8;
            if !cpu.registers.get_carry_flag() {
                let return_address = cpu.registers.get_pc();
                cpu.push((return_address >> 8) as u8);
                cpu.push((return_address & 0xFF) as u8);
                cpu.registers.set_pc(call_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xD5] = Some(&Instruction {
        opcode: 0xD5,
        name: "PUSH DE",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.push(cpu.registers.get_d());
            cpu.push(cpu.registers.get_e());
            opcode.cycles as u64
        },
    });
    opcodes[0xD6] = Some(&Instruction {
        opcode: 0xD6,
        name: "SUB A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(byte);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xD7] = Some(&Instruction {
        opcode: 0xD7,
        name: "RST $10",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xD8] = Some(&Instruction {
        opcode: 0xD8,
        name: "RET C",
        cycles: 5,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            if cpu.registers.get_carry_flag() {
                let mut return_addr = cpu.pop() as u16;
                return_addr |= (cpu.pop() as u16) << 8;
                cpu.registers.set_pc(return_addr);
                return opcode.cycles as u64
            }
            2
        },
    });
    opcodes[0xD9] = Some(&Instruction {
        opcode: 0xD9,
        name: "RETI",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_addr = cpu.pop() as u16;
            return_addr |= (cpu.pop() as u16) << 8;
            cpu.registers.set_pc(return_addr);
            // TODO: Also need to restore Interrupt Enable flag to pre-interrupt status
            opcode.cycles as u64
        },
    });
    opcodes[0xDA] = Some(&Instruction {
        opcode: 0xDA,
        name: "JP C, imm16",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut return_address: u16 = 0x00;
            return_address |= cpu.fetch_next() as u16;
            return_address |= (cpu.fetch_next() as u16) << 8;
            if cpu.registers.get_carry_flag() {
                cpu.registers.set_pc(return_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xDC] = Some(&Instruction {
        opcode: 0xDC,
        name: "CALL C, imm16",
        cycles: 6,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut call_address: u16 = 0x00;
            call_address |= cpu.fetch_next() as u16;
            call_address |= (cpu.fetch_next() as u16) << 8;
            if cpu.registers.get_carry_flag() {
                cpu.push((cpu.registers.get_pc() >> 8) as u8);
                cpu.push((cpu.registers.get_pc() & 0xFF) as u8);
                cpu.registers.set_pc(call_address);
                return opcode.cycles as u64
            }
            3
        },
    });
    opcodes[0xDE] = Some(&Instruction {
        opcode: 0xDE,
        name: "SBC A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            let old_value = cpu.registers.get_a();
            let intermediate_value = old_value.wrapping_sub(byte);
            let new_value = intermediate_value.wrapping_sub(cpu.registers.get_carry_flag() as u8);
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag(((new_value & 0x0F) > (intermediate_value & 0x0F)) || ((intermediate_value & 0x0F) > (old_value & 0x0F)));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xDF] = Some(&Instruction {
        opcode: 0xDF,
        name: "RST $18",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xE0] = Some(&Instruction {
        opcode: 0xE0,
        name: "LDH [imm8], A",
        cycles: 3,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.fetch_next() as u16) & 0xFF;
            let mem_addr = 0xFF00 | byte;
            cpu.ram.write(mem_addr, cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0xE1] = Some(&Instruction {
        opcode: 0xE1,
        name: "POP HL",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut byte = cpu.pop();
            cpu.registers.set_l(byte);
            byte = cpu.pop();
            cpu.registers.set_h(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0xE2] = Some(&Instruction {
        opcode: 0xE2,
        name: "LDH [C], A",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.registers.get_c() as u16) & 0xFF;
            let mem_addr = 0xFF00 | byte;
            cpu.ram.write(mem_addr, cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0xE5] = Some(&Instruction {
        opcode: 0xE5,
        name: "PUSH HL",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.push(cpu.registers.get_h());
            cpu.push(cpu.registers.get_l());
            opcode.cycles as u64
        },
    });
    opcodes[0xE6] = Some(&Instruction {
        opcode: 0xE6,
        name: "AND A, imm8",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value & cpu.fetch_next();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(true);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xE7] = Some(&Instruction {
        opcode: 0xE7,
        name: "RST $20",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xE8] = Some(&Instruction {
        opcode: 0xE8,
        name: "ADD SP, e8",
        cycles: 4,
        size: 2,
        flags: &[FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.fetch_next() as i8) as i16;
            let old_sp = cpu.registers.get_sp();
            if byte > 0 {
                cpu.registers.set_sp(old_sp.wrapping_add(byte as u16));
                cpu.registers.set_half_carry_flag((cpu.registers.get_sp() & 0x0F00) < (old_sp & 0x0F00));
                cpu.registers.set_carry_flag(cpu.registers.get_sp() < old_sp);
            } else {
                cpu.registers.set_sp(old_sp.wrapping_sub(byte.abs() as u16));
                cpu.registers.set_half_carry_flag((cpu.registers.get_sp() & 0x0F00) > (old_sp & 0x0F00));
                cpu.registers.set_carry_flag(cpu.registers.get_sp() > old_sp);
            }
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xE9] = Some(&Instruction {
        opcode: 0xE9,
        name: "JP HL",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_pc(cpu.registers.get_hl());
            opcode.cycles as u64
        },
    });
    opcodes[0xEA] = Some(&Instruction {
        opcode: 0xEA,
        name: "LD [imm16], A",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut imm_address: u16 = 0x00;
            imm_address |= cpu.fetch_next() as u16 & 0xFF;
            imm_address |= (cpu.fetch_next() as u16) << 8;
            cpu.ram.write(imm_address, cpu.registers.get_a());
            opcode.cycles as u64
        },
    });
    opcodes[0xEE] = Some(&Instruction {
        opcode: 0xEE,
        name: "XOR A, imm8",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value ^ cpu.fetch_next();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xEF] = Some(&Instruction {
        opcode: 0xEF,
        name: "RST $28",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xF0] = Some(&Instruction {
        opcode: 0xF0,
        name: "LDH A, [imm8]",
        cycles: 3,
        size: 2,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.fetch_next() as u16) & 0xFF;
            let mem_addr = 0xFF00 | byte;
            cpu.registers.set_a(cpu.ram.read(mem_addr));
            opcode.cycles as u64
        },
    });
    opcodes[0xF1] = Some(&Instruction {
        opcode: 0xF1,
        name: "POP AF",
        cycles: 3,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut byte = cpu.pop();
            cpu.registers.set_f(byte);
            byte = cpu.pop();
            cpu.registers.set_a(byte);
            opcode.cycles as u64
        },
    });
    opcodes[0xF2] = Some(&Instruction {
        opcode: 0xF2,
        name: "LDH A, [C]",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.registers.get_c() as u16) & 0xFF;
            let mem_addr = 0xFF00 | byte;
            cpu.registers.set_a(cpu.ram.read(mem_addr));
            opcode.cycles as u64
        },
    });
    opcodes[0xF3] = Some(&Instruction {
        opcode: 0xF3,
        name: "DI",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.interrupts_enabled = false;
            opcode.cycles as u64
        },
    });
    opcodes[0xF5] = Some(&Instruction {
        opcode: 0xF5,
        name: "PUSH AF",
        cycles: 3,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.push(cpu.registers.get_a());
            cpu.push(cpu.registers.get_f());
            opcode.cycles as u64
        },
    });
    opcodes[0xF6] = Some(&Instruction {
        opcode: 0xF6,
        name: "OR A, imm8",
        cycles: 2,
        size: 1,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let old_value = cpu.registers.get_a();
            let new_value = old_value | cpu.fetch_next();
            cpu.registers.set_a(new_value);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(false);
            cpu.registers.set_half_carry_flag(false);
            cpu.registers.set_carry_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xF7] = Some(&Instruction {
        opcode: 0xF7,
        name: "RST $30",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes[0xF8] = Some(&Instruction {
        opcode: 0xF8,
        name: "LD HL, SP + e8",
        cycles: 3,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = (cpu.fetch_next() as i8) as i16;
            let mut temp_sp = cpu.registers.get_sp();
            if byte > 0 {
                temp_sp = temp_sp.wrapping_add(byte as u16);
                cpu.registers.set_half_carry_flag((temp_sp & 0x0F00) < (cpu.registers.get_sp() & 0x0F00));
                cpu.registers.set_carry_flag(temp_sp < cpu.registers.get_sp());
            } else {
                temp_sp = temp_sp.wrapping_sub(byte.abs() as u16);
                cpu.registers.set_half_carry_flag((temp_sp & 0x0F00) > (cpu.registers.get_sp() & 0x0F00));
                cpu.registers.set_carry_flag(temp_sp > cpu.registers.get_sp());
            }
            cpu.registers.set_hl(temp_sp);
            cpu.registers.set_zero_flag(false);
            cpu.registers.set_negative_flag(false);
            opcode.cycles as u64
        },
    });
    opcodes[0xF9] = Some(&Instruction {
        opcode: 0xF9,
        name: "LD SP, HL",
        cycles: 2,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.registers.set_sp(cpu.registers.get_hl());
            opcode.cycles as u64
        },
    });
    opcodes[0xFA] = Some(&Instruction {
        opcode: 0xFA,
        name: "LD A, [imm16]",
        cycles: 4,
        size: 3,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let mut imm_address: u16 = 0x00;
            imm_address |= cpu.fetch_next() as u16 & 0xFF;
            imm_address |= (cpu.fetch_next() as u16) << 8;
            cpu.registers.set_a(cpu.ram.read(imm_address));
            opcode.cycles as u64
        },
    });
    opcodes[0xFB] = Some(&Instruction {
        opcode: 0xFB,
        name: "EI",
        cycles: 1,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            cpu.interrupts_enabled = true;
            opcode.cycles as u64
        },
    });
    opcodes[0xFE] = Some(&Instruction {
        opcode: 0xFE,
        name: "CP A, imm8",
        cycles: 2,
        size: 2,
        flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let byte = cpu.fetch_next();
            let old_value = cpu.registers.get_a();
            let new_value = old_value.wrapping_sub(byte);
            cpu.registers.set_zero_flag(new_value == 0);
            cpu.registers.set_negative_flag(true);
            cpu.registers.set_half_carry_flag((new_value & 0x0F) > (old_value & 0x0F));
            cpu.registers.set_carry_flag(new_value > old_value);
            opcode.cycles as u64
        },
    });
    opcodes[0xFF] = Some(&Instruction {
        opcode: 0xFF,
        name: "RST $30",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
            let return_addr = cpu.registers.get_pc();
            let interrupt_addr = ((opcode.opcode & 0b00_111_000) >> 3) as u16 * 8;
            cpu.push((return_addr >> 8) as u8);
            cpu.push((return_addr & 0xFF) as u8);
            cpu.registers.set_pc(interrupt_addr);
            opcode.cycles as u64
        },
    });
    opcodes
}

const fn create_cb_opcodes() -> [Option<&'static Instruction>; 256] {
    macro_rules! rlc {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1) | cpu.registers.get_carry_flag() as u8;
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.ram.read(cpu.registers.$get_reg());
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1) | cpu.registers.get_carry_flag() as u8;
                    cpu.ram.write(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }
    macro_rules! rl {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1) | old_carry;
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.ram.read(cpu.registers.$get_reg());
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1) | old_carry as u8;
                    cpu.ram.write(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }
    macro_rules! sla {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1);
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.ram.read(cpu.registers.$get_reg());
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b1000_0000) != 0);
                    let new_val = old_val.wrapping_shl(1);
                    cpu.ram.write(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }

    macro_rules! rrc {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1) | ((cpu.registers.get_carry_flag() as u8) << 7);
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.ram.read(cpu.registers.$get_reg());
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1) | ((cpu.registers.get_carry_flag() as u8) << 7);
                    cpu.ram.write(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }
    macro_rules! rr {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1) | (old_carry << 7);
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.ram.read(cpu.registers.$get_reg());
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1) | (old_carry << 7);
                    cpu.ram.write(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }
    macro_rules! sra {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1);
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.ram.read(cpu.registers.$get_reg());
                    let old_carry = cpu.registers.get_carry_flag() as u8;
                    cpu.registers.set_carry_flag((old_val & 0b0000_0001) != 0);
                    let new_val = old_val.wrapping_shr(1);
                    cpu.ram.write(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }

    macro_rules! swap {
        ($opcode:expr, $name:expr, r8, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 2,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.registers.$get_reg();
                    let old_low_nibble =  old_val & 0x0F;
                    let old_high_nibble =  old_val & 0xF0;
                    let new_val = (old_low_nibble << 4) | (old_high_nibble >> 4);
                    cpu.registers.$set_reg(new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    cpu.registers.set_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
        ($opcode:expr, $name:expr, ar16, $set_reg:ident, $get_reg:ident) => {
            Some(&Instruction {
                opcode: $opcode,
                name: $name,
                cycles: 4,
                size: 2,
                flags: &[FlagBits::Z, FlagBits::N, FlagBits::H, FlagBits::C],
                execute: |opcode: &Instruction, cpu: &mut CPU| -> u64 {
                    let old_val = cpu.ram.read(cpu.registers.$get_reg());
                    let old_low_nibble =  old_val & 0x0F;
                    let old_high_nibble =  old_val & 0xF0;
                    let new_val = (old_low_nibble << 4) | (old_high_nibble >> 4);
                    cpu.ram.write(cpu.registers.$get_reg(), new_val);
                    cpu.registers.set_zero_flag(new_val == 0);
                    cpu.registers.set_negative_flag(false);
                    cpu.registers.set_half_carry_flag(false);
                    cpu.registers.set_carry_flag(false);
                    opcode.cycles as u64
                }
            })
        };
    }

    let mut opcodes = [None; 256];
    opcodes[0x00] = rlc!(0x00, "RLC B", r8, set_b, get_b);
    opcodes[0x01] = rlc!(0x01, "RLC C", r8, set_c, get_c);
    opcodes[0x02] = rlc!(0x02, "RLC D", r8, set_d, get_d);
    opcodes[0x03] = rlc!(0x03, "RLC E", r8, set_e, get_e);
    opcodes[0x04] = rlc!(0x04, "RLC H", r8, set_h, get_h);
    opcodes[0x05] = rlc!(0x05, "RLC L", r8, set_l, get_l);
    opcodes[0x06] = rlc!(0x06, "RLC [HL]", ar16, set_hl, get_hl);
    opcodes[0x07] = rlc!(0x07, "RLC A", r8, set_a, get_a);

    opcodes[0x08] = rrc!(0x08, "RRC B", r8, set_b, get_b);
    opcodes[0x09] = rrc!(0x09, "RRC C", r8, set_c, get_c);
    opcodes[0x0A] = rrc!(0x0a, "RRC D", r8, set_d, get_d);
    opcodes[0x0B] = rrc!(0x0b, "RRC E", r8, set_e, get_e);
    opcodes[0x0C] = rrc!(0x0c, "RRC H", r8, set_h, get_h);
    opcodes[0x0D] = rrc!(0x0d, "RRC L", r8, set_l, get_l);
    opcodes[0x0E] = rrc!(0x0e, "RRC [HL]", ar16, set_hl, get_hl);
    opcodes[0x0F] = rrc!(0x0f, "RRC A", r8, set_a, get_a);

    opcodes[0x10] = rl!(0x10, "RL B", r8, set_b, get_b);
    opcodes[0x11] = rl!(0x11, "RL C", r8, set_c, get_c);
    opcodes[0x12] = rl!(0x12, "RL D", r8, set_d, get_d);
    opcodes[0x13] = rl!(0x13, "RL E", r8, set_e, get_e);
    opcodes[0x14] = rl!(0x14, "RL H", r8, set_h, get_h);
    opcodes[0x15] = rl!(0x15, "RL L", r8, set_l, get_l);
    opcodes[0x16] = rl!(0x16, "RL [HL]", ar16, set_hl, get_hl);
    opcodes[0x17] = rl!(0x17, "RL A", r8, set_a, get_a);

    opcodes[0x18] = rr!(0x18, "RR B", r8, set_b, get_b);
    opcodes[0x19] = rr!(0x19, "RR C", r8, set_c, get_c);
    opcodes[0x1A] = rr!(0x1a, "RR D", r8, set_d, get_d);
    opcodes[0x1B] = rr!(0x1b, "RR E", r8, set_e, get_e);
    opcodes[0x1C] = rr!(0x1c, "RR H", r8, set_h, get_h);
    opcodes[0x1D] = rr!(0x1d, "RR L", r8, set_l, get_l);
    opcodes[0x1E] = rr!(0x1e, "RR [HL]", ar16, set_hl, get_hl);
    opcodes[0x1F] = rr!(0x1f, "RR A", r8, set_a, get_a);

    opcodes[0x20] = sla!(0x20, "SLA B", r8, set_b, get_b);
    opcodes[0x21] = sla!(0x21, "SLA C", r8, set_c, get_c);
    opcodes[0x22] = sla!(0x22, "SLA D", r8, set_d, get_d);
    opcodes[0x23] = sla!(0x23, "SLA E", r8, set_e, get_e);
    opcodes[0x24] = sla!(0x24, "SLA H", r8, set_h, get_h);
    opcodes[0x25] = sla!(0x25, "SLA L", r8, set_l, get_l);
    opcodes[0x26] = sla!(0x26, "SLA [HL]", ar16, set_hl, get_hl);
    opcodes[0x27] = sla!(0x27, "SLA A", r8, set_a, get_a);

    opcodes[0x28] = sra!(0x28, "SRA B", r8, set_b, get_b);
    opcodes[0x29] = sra!(0x29, "SRA C", r8, set_c, get_c);
    opcodes[0x2A] = sra!(0x2a, "SRA D", r8, set_d, get_d);
    opcodes[0x2B] = sra!(0x2b, "SRA E", r8, set_e, get_e);
    opcodes[0x2C] = sra!(0x2c, "SRA H", r8, set_h, get_h);
    opcodes[0x2D] = sra!(0x2d, "SRA L", r8, set_l, get_l);
    opcodes[0x2E] = sra!(0x2e, "SRA [HL]", ar16, set_hl, get_hl);
    opcodes[0x2F] = sra!(0x2f, "SRA A", r8, set_a, get_a);

    opcodes[0x30] = swap!(0x30, "SWAP B", r8, set_b, get_b);
    opcodes[0x31] = swap!(0x31, "SWAP C", r8, set_c, get_c);
    opcodes[0x32] = swap!(0x32, "SWAP D", r8, set_d, get_d);
    opcodes[0x33] = swap!(0x33, "SWAP E", r8, set_e, get_e);
    opcodes[0x34] = swap!(0x34, "SWAP H", r8, set_h, get_h);
    opcodes[0x35] = swap!(0x35, "SWAP L", r8, set_l, get_l);
    opcodes[0x36] = swap!(0x36, "SWAP [HL]", ar16, set_hl, get_hl);
    opcodes[0x37] = swap!(0x37, "SWAP A", r8, set_a, get_a);
    opcodes
}

pub const OPCODES: [Option<&'static Instruction>; 256] = create_opcodes();

pub const OPCODES_CB: [Option<&'static Instruction>; 256] = create_cb_opcodes();


#[cfg(test)]
mod test {
    use crate::GB::CPU::CPU;
    use crate::GB::RAM;
    use crate::GB::RAM::{USER_PROGRAM_ADDRESS, WRAM_ADDRESS};

    macro_rules! test_flags {
        ($cpu:ident, $zero:expr, $negative:expr, $half:expr, $carry:expr) => {
            assert_eq!($cpu.registers.get_zero_flag(), $zero);
            assert_eq!($cpu.registers.get_negative_flag(), $negative);
            assert_eq!($cpu.registers.get_half_carry_flag(), $half);
            assert_eq!($cpu.registers.get_carry_flag(), $carry);
        };
    }

    macro_rules! test_ld_r8 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident, $set_reg_from:ident, $get_reg_from:ident) => {
            #[test]
            fn $func() {
                let test_value: u8 = 0xC4;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg_to(0x00);
                cpu.registers.$set_reg_from(test_value);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), test_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };
        // Rule when Source R = Destination R
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let test_value: u8 = 0xC4;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg(test_value);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg(), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_ld_r16 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident, $set_reg_from:ident, $get_reg_from:ident) => {
            #[test]
            fn $func() {
                let test_value: u16 = 0xE5C4;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg_to(0x00);
                cpu.registers.$set_reg_from(test_value);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_to(), test_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };
        // Rule when Source R = Destination R
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let test_value: u16 = 0xD5C4;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg(test_value);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg(), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_ld_r16_imm16 {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident, $get_reg_high:ident, $get_reg_low:ident) => {
            #[test]
            fn $func() {
                let test_value: u16 = 0xC05A;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode, 0x5A, 0xC0];
                cpu.load(&program);
                let registers_copy = cpu.registers;
                cpu.registers.$set_reg(0);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.$get_reg_high(), 0xC0);
                assert_eq!(cpu.registers.$get_reg_low(), 0x5A);
                assert_eq!(cpu.registers.$get_reg(), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    registers_copy.get_zero_flag(),
                    registers_copy.get_negative_flag(),
                    registers_copy.get_half_carry_flag(),
                    registers_copy.get_carry_flag()
                );
            }
        };
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let test_value: u16 = 0xC05A;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode, 0x5A, 0xC0];
                cpu.load(&program);
                let registers_copy = cpu.registers;
                cpu.registers.$set_reg(0);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.$get_reg(), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    registers_copy.get_zero_flag(),
                    registers_copy.get_negative_flag(),
                    registers_copy.get_half_carry_flag(),
                    registers_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_ld_r8_imm8 {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let test_value: u8 = 0x5A;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode, 0x5A];
                cpu.load(&program);
                let registers_copy = cpu.registers;
                cpu.registers.$set_reg(0);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg(), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    registers_copy.get_zero_flag(),
                    registers_copy.get_negative_flag(),
                    registers_copy.get_half_carry_flag(),
                    registers_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_ld_ar16_r8 {
        ($opcode:expr, $func:ident, $set_reg_addr:ident, $get_reg_addr:ident, $set_reg_from:ident, $get_reg_from:ident) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0xC4;
                let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
                let mut cpu = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg_from(test_value_1);
                cpu.registers.$set_reg_addr(test_address_1);
                cpu.ram.write(test_address_1, 0x00);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_1);
                assert_eq!(cpu.registers.$get_reg_addr(), test_address_1);
                assert_eq!(cpu.ram.read(test_address_1), test_value_1);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_ld_imm16_r8 {
        ($opcode:expr, $func:ident, $set_from:ident, $get_from:ident) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0xC4;
                let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
                let test_address_1_low: u8 = (test_address_1 & 0xFF) as u8;
                let test_address_1_high: u8 = (test_address_1 >> 8) as u8;
                let mut cpu = CPU::new();
                let program_1: Vec<u8> = vec![$opcode, test_address_1_low, test_address_1_high];
                cpu.load(&program_1);
                let register_copy = cpu.registers;
                cpu.registers.$set_from(test_value_1);
                cpu.ram.write(test_address_1, 0x00);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu.registers.$get_from(), test_value_1);
                assert_eq!(cpu.ram.read(test_address_1), test_value_1);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_ld_r8_imm16 {
        ($opcode:expr, $func:ident, $set_to:ident, $get_to:ident) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0xC4;
                let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
                let test_address_1_low: u8 = (test_address_1 & 0xFF) as u8;
                let test_address_1_high: u8 = (test_address_1 >> 8) as u8;
                let mut cpu = CPU::new();
                let program_1: Vec<u8> = vec![$opcode, test_address_1_low, test_address_1_high];
                cpu.load(&program_1);
                let register_copy = cpu.registers;
                cpu.registers.$set_to(0);
                cpu.ram.write(test_address_1, test_value_1);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu.registers.$get_to(), test_value_1);
                assert_eq!(cpu.ram.read(test_address_1), test_value_1);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_ld_ar16 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident, $set_reg_addr:ident, $get_reg_addr:ident) => {
            #[test]
            fn $func() {
                let test_value: u8 = 0xC4;
                let test_address: u16 = WRAM_ADDRESS as u16 + 0x15B;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg_to(0x00);
                cpu.registers.$set_reg_addr(test_address);
                cpu.ram.write(cpu.registers.$get_reg_addr(), test_value);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_to(), test_value);
                assert_eq!(cpu.registers.$get_reg_addr(), test_address);
                assert_eq!(cpu.ram.read(test_address), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_ldh_r8_imm8 {
        // $byte_is_src = true -> LDH A, [imm8] else -> LDH [imm8], A
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident, $byte_is_src:expr) => {
            #[test]
            fn $func() {
                let test_value: u8 = 0xD8;
                let test_addr_low: u8 = 0x5A;
                let test_addr: u16 = 0xFF00 | (test_addr_low as u16 & 0xFF);
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode, 0x5A];
                cpu.load(&program);
                let registers_copy = cpu.registers;
                cpu.ram.write(test_addr, if $byte_is_src {test_value} else {0});
                cpu.registers.$set_reg(if !$byte_is_src {test_value} else {0});
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.$get_reg(), test_value);
                assert_eq!(cpu.ram.read(test_addr), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    registers_copy.get_zero_flag(),
                    registers_copy.get_negative_flag(),
                    registers_copy.get_half_carry_flag(),
                    registers_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_ldh_r8_r8 {
        // $rtl = true -> LDH r8, [r'8] else -> LDH [r'8], r8
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident, $set_reg_2:ident, $get_reg_2:ident, $rtl:expr) => {
            #[test]
            fn $func() {
                let test_value: u8 = 0xD8;
                let test_addr_low: u8 = 0x5A;
                let test_addr: u16 = 0xFF00 | (test_addr_low as u16 & 0xFF);
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let registers_copy = cpu.registers;
                cpu.ram.write(test_addr, if $rtl {test_value} else {0});
                cpu.registers.$set_reg_2(test_addr_low);
                cpu.registers.$set_reg(if !$rtl {test_value} else {0});
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg(), test_value);
                assert_eq!(cpu.registers.$get_reg_2(), test_addr_low);
                assert_eq!(cpu.ram.read(test_addr), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    registers_copy.get_zero_flag(),
                    registers_copy.get_negative_flag(),
                    registers_copy.get_half_carry_flag(),
                    registers_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_inc_r16 {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident, $get_reg_high:ident, $get_reg_low:ident) => {
            #[test]
            fn $func() {
                let test_value: u16 = 0xC0F4;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg(test_value - 1);
                cpu.execute_next();
                assert_eq!(cpu.registers.$get_reg_high(), 0xC0);
                assert_eq!(cpu.registers.$get_reg_low(), 0xF4);
                assert_eq!(cpu.registers.$get_reg(), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let test_value: u16 = 0xC0F4;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg(test_value - 1);
                cpu.execute_next();
                assert_eq!(cpu.registers.$get_reg(), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_dec_r16 {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident, $get_reg_high:ident, $get_reg_low:ident) => {
            #[test]
            fn $func() {
                let test_value: u16 = 0xC0F4;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg(test_value + 1);
                cpu.execute_next();
                assert_eq!(cpu.registers.$get_reg_high(), 0xC0);
                assert_eq!(cpu.registers.$get_reg_low(), 0xF4);
                assert_eq!(cpu.registers.$get_reg(), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };

        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let test_value: u16 = 0xC0F4;
                let mut cpu = CPU::new();
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg(test_value + 1);
                cpu.execute_next();
                assert_eq!(cpu.registers.$get_reg(), test_value);
                // Flags untouched
                test_flags!(
                    cpu,
                    register_copy.get_zero_flag(),
                    register_copy.get_negative_flag(),
                    register_copy.get_half_carry_flag(),
                    register_copy.get_carry_flag()
                );
            }
        };
    }

    macro_rules! test_inc_r8 {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                //No Flags
                let mut test_value_1: u8 = 0b1111_0100;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                cpu_1.registers.$set_reg(test_value_1 - 1);
                cpu_1.execute_next();
                assert_eq!(cpu_1.registers.$get_reg(), test_value_1);
                test_flags!(cpu_1, false, false, false, registers_copy.get_carry_flag());

                // Flags Z/H
                test_value_1 = 0xFF;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                cpu_1.registers.$set_reg(test_value_1);
                cpu_1.execute_next();
                assert_eq!(cpu_1.registers.$get_reg(), 0);
                test_flags!(cpu_1, true, false, true, registers_copy.get_carry_flag());

                // Flags H
                test_value_1 = 0x0F;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                cpu_1.registers.$set_reg(test_value_1);
                cpu_1.execute_next();
                assert_eq!(cpu_1.registers.$get_reg(), 0x10);
                test_flags!(cpu_1, false, false, true, registers_copy.get_carry_flag());
            }
        };
    }

    macro_rules! test_dec_r8 {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                //No Flags
                let mut test_value_1: u8 = 0xF4;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                cpu_1.registers.$set_reg(test_value_1 + 1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_1);
                test_flags!(cpu_1, false, true, false, registers_copy.get_carry_flag());

                // Flags H
                test_value_1 = 0x00;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                cpu_1.registers.$set_reg(test_value_1);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg(), 0xFF);
                test_flags!(cpu_1, false, true, true, registers_copy.get_carry_flag());

                // Flags Z
                test_value_1 = 0x00;
                let mut cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                cpu_1.registers.$set_reg(test_value_1 + 1);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_1);
                test_flags!(cpu_1, true, true, false, registers_copy.get_carry_flag());

                // Flags H
                test_value_1 = 0xF0;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                cpu_1.registers.$set_reg(test_value_1);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_1 - 1);
                test_flags!(cpu_1, false, true, true, registers_copy.get_carry_flag());
            }
        };
    }

    macro_rules! test_add_r8_r8 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident, $set_reg_from:ident, $get_reg_from:ident) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x16;
                let mut expected_value: u8 = test_value_1.wrapping_add(test_value_2);
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // No Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0x10;
                expected_value = 0x00;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // Z/C Flags
                test_flags!(cpu_1, true, false, false, true);

                test_value_1 = 0x0F;
                test_value_2 = 0x01;
                expected_value = 0x10;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // H Flag
                test_flags!(cpu_1, false, false, true, false);

                test_value_1 = 0xFF;
                test_value_2 = 0x01;
                expected_value = 0x00;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // Z/H/C Flag
                test_flags!(cpu_1, true, false, true, true);
            }
        };
    }

    macro_rules! test_adc_r8_r8 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident, $set_reg_from:ident, $get_reg_from:ident, $carry:expr) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x16;
                let mut expected_value: u8 = test_value_1.wrapping_add(test_value_2).wrapping_add($carry as u8);
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                cpu_1.registers.set_carry_flag($carry);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // No Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0x10 - $carry as u8;
                expected_value = 0x00;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // Z/C Flags
                test_flags!(cpu_1, true, false, false, true);

                test_value_1 = 0x0F;
                test_value_2 = 0x01;
                expected_value = 0x10 + $carry as u8;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // H Flag
                test_flags!(cpu_1, false, false, true, false);

                test_value_1 = 0xFF - $carry as u8;
                test_value_2 = 0x01;
                expected_value = 0x00;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // Z/H/C Flag
                test_flags!(cpu_1, true, false, true, true);
            }
        };
    }

    macro_rules! test_adc_r8_r8 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident, $set_reg_from:ident, $get_reg_from:ident, $carry:expr) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x16;
                let mut expected_value: u8 = test_value_1.wrapping_add(test_value_2).wrapping_add($carry as u8);
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                cpu_1.registers.set_carry_flag($carry);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // No Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0x10 - $carry as u8;
                expected_value = 0x00;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // Z/C Flags
                test_flags!(cpu_1, true, false, $carry, true);

                test_value_1 = 0x0F;
                test_value_2 = 0x01;
                expected_value = 0x10 + $carry as u8;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // H Flag
                test_flags!(cpu_1, false, false, true, false);

                test_value_1 = 0xFF - $carry as u8;
                test_value_2 = 0x01;
                expected_value = 0x00;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.$set_reg_from(test_value_2);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg_from(), test_value_2);
                // Z/H/C Flag
                test_flags!(cpu_1, true, false, true, true);
            }
        };
    }

    macro_rules! test_add_r8_imm8 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x16;
                let mut expected_value: u8 = test_value_1.wrapping_add(test_value_2);
                let mut cpu_1 = CPU::new();
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                // No Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0x10;
                expected_value = 0x00;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                // Z/C Flags
                test_flags!(cpu_1, true, false, false, true);

                test_value_1 = 0x0F;
                test_value_2 = 0x01;
                expected_value = 0x10;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                // H Flag
                test_flags!(cpu_1, false, false, true, false);

                test_value_1 = 0xFF;
                test_value_2 = 0x01;
                expected_value = 0x00;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                // Z/H/C Flag
                test_flags!(cpu_1, true, false, true, true);
            }
        };
    }

    macro_rules! test_adc_r8_imm8 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident, $carry:expr) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x16;
                let mut expected_value: u8 = test_value_1.wrapping_add(test_value_2).wrapping_add($carry as u8);
                let mut cpu_1 = CPU::new();
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.set_carry_flag($carry);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                // No Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0x10 - $carry as u8;
                expected_value = 0x00;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                // Z/C Flags
                test_flags!(cpu_1, true, false, $carry, true);

                test_value_1 = 0x0F;
                test_value_2 = 0x01;
                expected_value = 0x10 + $carry as u8;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                // H Flag
                test_flags!(cpu_1, false, false, true, false);

                test_value_1 = 0xFF - $carry as u8;
                test_value_2 = 0x01;
                expected_value = 0x00;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_to(test_value_1);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_to(), expected_value);
                // Z/H/C Flag
                test_flags!(cpu_1, true, false, true, true);
            }
        };
    }

    macro_rules! test_sbc_r8_imm8 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident, $carry:expr) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x13;
                let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2 + $carry as u8);
                let mut cpu_1 = CPU::new();
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_carry_flag($carry);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // N Flags
                test_flags!(cpu_1, false, true, false, false);

                test_value_1 = 0x10;
                test_value_2 = 0x0E;
                expected_value = test_value_1.wrapping_sub(test_value_2 + $carry as u8);
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // H Flags
                test_flags!(cpu_1, false, true, true, false);

                test_value_1 = 0x10;
                test_value_2 = 0x0F;
                expected_value = test_value_1.wrapping_sub(test_value_2).wrapping_sub($carry as u8);
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // Z/H Flag
                test_flags!(cpu_1, $carry, true, true, false);

                test_value_1 = 0x00;
                test_value_2 = 0;
                expected_value = test_value_1.wrapping_sub($carry as u8);
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_carry_flag($carry);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // H/C Flag
                test_flags!(cpu_1, !$carry, true, $carry, $carry);
            }
        };
    }

    macro_rules! test_and_a_r8 {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let mut expected_value: u8 = test_value_1 & test_value_2;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // H Flag
                test_flags!(cpu_1, false, false, true, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0100_0110;
                expected_value = 0x0;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, true, false, true, false);
            }
        };
        ($opcode:expr, $func:ident, hl) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let test_address: u16 = WRAM_ADDRESS as u16 + 0x22;
                let mut expected_value: u8 = test_value_1 & test_value_2;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // H Flag
                test_flags!(cpu_1, false, false, true, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0100_0110;
                expected_value = 0x0;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, true, false, true, false);
            }
        };
        ($opcode:expr, $func:ident, a) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut expected_value: u8 = test_value_1 & test_value_1;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // H Flag
                test_flags!(cpu_1, false, false, true, false);

                test_value_1 = 0b1010_1001;
                expected_value = test_value_1;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu_1, false, false, true, false);
            }
        };
    }

    macro_rules! test_and_a_imm8 {
        ($opcode:expr, $func:ident) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let mut expected_value: u8 = test_value_1 & test_value_2;
                let mut cpu_1 = CPU::new();
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // H Flag
                test_flags!(cpu_1, false, false, true, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0100_0110;
                program_1[1] = test_value_2;
                expected_value = 0x0;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu_1, true, false, true, false);
            }
        };
    }

    macro_rules! test_xor_a_r8 {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let mut expected_value: u8 = test_value_1 ^ test_value_2;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // H Flag
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                expected_value = 0xFF;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b1010_1001;
                expected_value = 0x0;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, true, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, hl) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let test_address: u16 = WRAM_ADDRESS as u16 + 0x22;
                let mut expected_value: u8 = test_value_1 ^ test_value_2;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // H Flag
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                expected_value = 0xFF;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b1010_1001;
                expected_value = 0x0;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, true, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, a) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut expected_value: u8 = 0x0;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // H Flag
                test_flags!(cpu_1, true, false, false, false);
            }
        };
    }

    macro_rules! test_xor_a_imm8 {
        ($opcode:expr, $func:ident) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let mut expected_value: u8 = test_value_1 ^ test_value_2;
                let mut cpu_1 = CPU::new();
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // H Flag
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                program_1[1] = test_value_2;
                expected_value = 0xFF;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b1010_1001;
                expected_value = 0x0;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu_1, true, false, false, false);
            }
        };
    }

    macro_rules! test_or_a_r8 {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let mut expected_value: u8 = test_value_1 | test_value_2;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // H Flag
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                expected_value = 0xFF;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b1010_1001;
                expected_value = test_value_1;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b0;
                test_value_2 = 0b0;
                expected_value = test_value_1;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, true, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, hl) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let test_address: u16 = WRAM_ADDRESS as u16 + 0x22;
                let mut expected_value: u8 = test_value_1 | test_value_2;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // H Flag
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                expected_value = 0xFF;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0;
                expected_value = test_value_1;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b0;
                test_value_2 = 0b0;
                expected_value = test_value_1;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu_1, true, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, a) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut expected_value: u8 = test_value_1;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // No Flag
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b0;
                expected_value = test_value_1;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // Z Flag
                test_flags!(cpu_1, true, false, false, false);
            }
        };
    }

    macro_rules! test_or_a_imm8 {
        ($opcode:expr, $func:ident) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let mut expected_value: u8 = test_value_1 | test_value_2;
                let mut cpu_1 = CPU::new();
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // No Flag
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                expected_value = 0xFF;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b1010_1001;
                expected_value = test_value_1;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu_1, false, false, false, false);

                test_value_1 = 0b0;
                test_value_2 = 0b0;
                expected_value = test_value_1;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu_1, true, false, false, false);
            }
        };
    }

    macro_rules! test_cp_a_r8 {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x11;
                let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // No Flags
                test_flags!(cpu_1, false, true, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0xF0;
                expected_value = 0x00;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // Z/N Flags
                test_flags!(cpu_1, true, true, false, false);

                test_value_1 = 0x10;
                test_value_2 = 0x01;
                expected_value = 0x0F;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // N/H Flag
                test_flags!(cpu_1, false, true, true, false);

                test_value_1 = 0x10;
                test_value_2 = 0x20;
                expected_value = test_value_1.wrapping_sub(test_value_2);
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // N/C Flag
                test_flags!(cpu_1, false, true, false, true);

                test_value_1 = 0x00;
                test_value_2 = 0x01;
                expected_value = 0xFF;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.$set_reg(test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                assert_eq!(cpu_1.registers.$get_reg(), test_value_2);
                // N/H/C Flag
                test_flags!(cpu_1, false, true, true, true);
            }
        };
        ($opcode:expr, $func:ident, hl) => {
            #[test]
            fn $func() {
                let test_address: u16 = WRAM_ADDRESS as u16 + 0x55;
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x11;
                let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // No Flags
                test_flags!(cpu_1, false, true, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0xF0;
                expected_value = 0x00;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // Z/N Flags
                test_flags!(cpu_1, true, true, false, false);

                test_value_1 = 0x10;
                test_value_2 = 0x01;
                expected_value = 0x0F;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // N/H Flag
                test_flags!(cpu_1, false, true, true, false);

                test_value_1 = 0x10;
                test_value_2 = 0x20;
                expected_value = test_value_1.wrapping_sub(test_value_2);
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // N/C Flag
                test_flags!(cpu_1, false, true, false, true);

                test_value_1 = 0x00;
                test_value_2 = 0x01;
                expected_value = 0xFF;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cpu_1.registers.set_hl(test_address);
                cpu_1.ram.write(test_address, test_value_2);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                assert_eq!(cpu_1.registers.get_hl(), test_address);
                assert_eq!(cpu_1.ram.read(test_address), test_value_2);
                // N/H/C Flag
                test_flags!(cpu_1, false, true, true, true);
            }
        };
        ($opcode:expr, $func:ident, a) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                // Z Flag
                test_flags!(cpu_1, true, true, false, false);

                test_value_1 = 0b0;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                // Z Flag
                test_flags!(cpu_1, true, true, false, false);
            }
        };
    }

    macro_rules! test_cp_a_imm8 {
        ($opcode:expr, $func:ident) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x11;
                let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
                let mut cpu_1 = CPU::new();
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                // No Flags
                test_flags!(cpu_1, false, true, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0xF0;
                expected_value = 0x00;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                // Z/N Flags
                test_flags!(cpu_1, true, true, false, false);

                test_value_1 = 0x10;
                test_value_2 = 0x01;
                expected_value = 0x0F;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                // N/H Flag
                test_flags!(cpu_1, false, true, true, false);

                test_value_1 = 0x10;
                test_value_2 = 0x20;
                expected_value = test_value_1.wrapping_sub(test_value_2);
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                // N/C Flag
                test_flags!(cpu_1, false, true, false, true);

                test_value_1 = 0x00;
                test_value_2 = 0x01;
                expected_value = 0xFF;
                program_1[1] = test_value_2;
                cpu_1 = CPU::new();
                cpu_1.load(&program_1);
                cpu_1.registers.set_a(test_value_1);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.get_a(), test_value_1);
                // N/H/C Flag
                test_flags!(cpu_1, false, true, true, true);
            }
        };
    }

    macro_rules! test_call {
        ($opcode:expr, $func:ident) => {
            #[test]
            fn $func() {
                let test_call_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x210;
                let test_call_low: u8 = (test_call_address & 0xFF) as u8;
                let test_call_high: u8 = (test_call_address >> 8) as u8;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode, test_call_low, test_call_high];
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                let return_address = cpu_1.registers.get_pc() + 3;
                let return_address_low = (return_address & 0xFF) as u8;
                let return_address_high = (return_address >> 8) as u8;
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 6);
                assert_eq!(cpu_1.registers.get_sp(), registers_copy.get_sp() - 2);
                assert_eq!(cpu_1.registers.get_pc(), test_call_address);
                assert_eq!(cpu_1.ram.read(cpu_1.registers.get_sp() + 1), return_address_low);
                assert_eq!(cpu_1.ram.read(cpu_1.registers.get_sp() + 2), return_address_high);
            }
        };
        ($opcode:expr, $func:ident, $inverse:expr, $set_flag:ident, $get_flag:ident) => {
            #[test]
            fn $func() {
                let test_call_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x210;
                let test_call_low: u8 = (test_call_address & 0xFF) as u8;
                let test_call_high: u8 = (test_call_address >> 8) as u8;
                let mut cpu = CPU::new();
                let program_1: Vec<u8> = vec![$opcode, test_call_low, test_call_high];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                if $inverse {cpu.registers.$set_flag(true)} else {cpu.registers.$set_flag(false)};
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp());
                assert_eq!(cpu.registers.get_pc(), registers_copy.get_pc() + 3);

                cpu = CPU::new();
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                let return_address = cpu.registers.get_pc() + 3;
                let return_address_low = (return_address & 0xFF) as u8;
                let return_address_high = (return_address >> 8) as u8;
                if $inverse {cpu.registers.$set_flag(false)} else {cpu.registers.$set_flag(true)};
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 6);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp() - 2);
                assert_eq!(cpu.registers.get_pc(), test_call_address);
                assert_eq!(cpu.ram.read(cpu.registers.get_sp() + 1), return_address_low);
                assert_eq!(cpu.ram.read(cpu.registers.get_sp() + 2), return_address_high);
            }
        };
    }

    macro_rules! test_ret {
        ($opcode:expr, $func:ident) => {
            #[test]
            fn $func() {
                let test_return_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x210;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                cpu_1.push((test_return_address >> 8) as u8);
                cpu_1.push((test_return_address & 0xFF) as u8);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.registers.get_sp(), registers_copy.get_sp());
                assert_eq!(cpu_1.registers.get_pc(), test_return_address);
            }
        };
        ($opcode:expr, $func:ident, $inverse:expr, $set_flag:ident, $get_flag:ident) => {
            #[test]
            fn $func() {
                let test_return_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x210;
                let mut cpu = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                cpu.push((test_return_address >> 8) as u8);
                cpu.push((test_return_address & 0xFF) as u8);
                if $inverse {cpu.registers.$set_flag(true)} else {cpu.registers.$set_flag(false)};
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp() - 2);
                assert_eq!(cpu.registers.get_pc(), registers_copy.get_pc() + 1);

                cpu = CPU::new();
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                cpu.push((test_return_address >> 8) as u8);
                cpu.push((test_return_address & 0xFF) as u8);
                if $inverse {cpu.registers.$set_flag(false)} else {cpu.registers.$set_flag(true)};
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 5);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp());
                assert_eq!(cpu.registers.get_pc(), test_return_address);
            }
        };
    }

    macro_rules! test_call {
        ($opcode:expr, $func:ident) => {
            #[test]
            fn $func() {
                let test_call_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x0C16;
                let test_call_address_high: u8 = (test_call_address >> 8) as u8;
                let test_call_address_low: u8 = (test_call_address & 0xFF) as u8;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode, test_call_address_low, test_call_address_high];
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                let test_return_address = registers_copy.get_pc() + program_1.len() as u16;
                let test_return_address_high = (test_return_address >> 8) as u8;
                let test_return_address_low = (test_return_address & 0xFF) as u8;
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 6);
                assert_eq!(cpu_1.registers.get_sp(), registers_copy.get_sp() - 2);
                assert_eq!(cpu_1.registers.get_pc(), test_call_address);
                assert_eq!(cpu_1.ram.read(cpu_1.registers.get_sp() + 1), test_return_address_low);
                assert_eq!(cpu_1.ram.read(cpu_1.registers.get_sp() + 2), test_return_address_high);
            }
        };
        ($opcode:expr, $func:ident, $inverse:expr, $set_flag:ident, $get_flag:ident) => {
            #[test]
            fn $func() {
                let test_call_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x0C16;
                let test_call_address_high: u8 = (test_call_address >> 8) as u8;
                let test_call_address_low: u8 = (test_call_address & 0xFF) as u8;
                let mut cpu = CPU::new();
                let program_1: Vec<u8> = vec![$opcode, test_call_address_low, test_call_address_high];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                if $inverse {cpu.registers.$set_flag(true)} else {cpu.registers.$set_flag(false)};
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp());
                assert_eq!(cpu.registers.get_pc(), registers_copy.get_pc() + program_1.len() as u16);

                cpu = CPU::new();
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                if $inverse {cpu.registers.$set_flag(false)} else {cpu.registers.$set_flag(true)};
                let test_return_address = registers_copy.get_pc() + program_1.len() as u16;
                let test_return_address_high = (test_return_address >> 8) as u8;
                let test_return_address_low = (test_return_address & 0xFF) as u8;
                cycles = cpu.execute_next();
                assert_eq!(cycles, 6);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp() - 2);
                assert_eq!(cpu.registers.get_pc(), test_call_address);
                assert_eq!(cpu.ram.read(cpu.registers.get_sp() + 1), test_return_address_low);
                assert_eq!(cpu.ram.read(cpu.registers.get_sp() + 2), test_return_address_high);
            }
        };
    }

    macro_rules! test_jump {
        ($opcode:expr, $func:ident) => {
            #[test]
            fn $func() {
                let test_jump_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x210;
                let test_address_high_byte: u8 = (test_jump_address >> 8) as u8;
                let test_address_low_byte: u8 = (test_jump_address & 0xFF) as u8;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![$opcode, test_address_low_byte, test_address_high_byte];
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.registers.get_pc(), test_jump_address);
            }
        };
        ($opcode:expr, $func:ident, $inverse:expr, $set_flag:ident, $get_flag:ident) => {
            #[test]
            fn $func() {
                let test_jump_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x210;
                let test_address_high_byte: u8 = (test_jump_address >> 8) as u8;
                let test_address_low_byte: u8 = (test_jump_address & 0xFF) as u8;
                let mut cpu = CPU::new();
                let program_1: Vec<u8> = vec![$opcode, test_address_low_byte, test_address_high_byte];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                if $inverse {cpu.registers.$set_flag(true)} else {cpu.registers.$set_flag(false)};
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.get_pc(), registers_copy.get_pc() + 3);

                cpu = CPU::new();
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                if $inverse {cpu.registers.$set_flag(false)} else {cpu.registers.$set_flag(true)};
                cycles = cpu.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu.registers.get_pc(), test_jump_address);
            }
        };
        ($opcode:expr, $func:ident, $set_from_reg:ident, $get_from_reg:ident) => {
            #[test]
            fn $func() {
                let test_jump_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x210;
                let mut cpu = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                cpu.registers.$set_from_reg(test_jump_address);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_pc(), test_jump_address);
                assert_eq!(cpu.registers.$get_from_reg(), test_jump_address);
            }
        };
    }

    macro_rules! test_pop {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let test_value: u16 = 0x521B;
                let test_high_byte: u8 = (test_value >> 8) as u8;
                let test_low_byte: u8 = (test_value & 0xFF) as u8;
                let mut cpu = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.$set_reg(0);
                let registers_copy = cpu.registers;
                cpu.push(test_high_byte);
                cpu.push(test_low_byte);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                if (stringify!($get_reg) == "get_af") {
                    assert_eq!(cpu.registers.$get_reg(), test_value & 0xFFF0);
                } else{
                    assert_eq!(cpu.registers.$get_reg(), test_value);
                }
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp());
            }
        };
    }

    macro_rules! test_push {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                let test_value: u16 = 0x521B;
                let test_high_byte: u8 = (test_value >> 8) as u8;
                let test_low_byte: u8 = (test_value & 0xFF) as u8;
                let mut cpu = CPU::new();
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.$set_reg(test_value);
                let registers_copy = cpu.registers;
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.$get_reg(), if stringify!($get_reg) == "get_af" {test_value & 0xFFF0} else {test_value});
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp() - 2);
                assert_eq!(cpu.ram.read(cpu.registers.get_sp() + 2), test_high_byte);
                if (stringify!($get_reg) == "get_af") {
                    assert_eq!(cpu.ram.read(cpu.registers.get_sp() + 1), test_low_byte & 0xF0);
                } else {
                    assert_eq!(cpu.ram.read(cpu.registers.get_sp() + 1), test_low_byte);
                }
            }
        };
    }

    macro_rules! test_rst {
        ($opcode:expr, $func:ident) => {
            #[test]
            fn $func() {
                let int_addr: u16 = (($opcode & 0b00_111_000) >> 3) as u16 * 8;
                let mut cpu_1 = CPU::new();
                let mut program_1: Vec<u8> = vec![$opcode];
                cpu_1.load(&program_1);
                let registers_copy = cpu_1.registers;
                let expected_return_addr = cpu_1.registers.get_pc() + program_1.len() as u16;
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.registers.get_pc(), int_addr);
                assert_eq!(cpu_1.registers.get_sp(), registers_copy.get_sp() - 2);
                assert_eq!(cpu_1.ram.read(cpu_1.registers.get_sp() + 1), (expected_return_addr & 0xFF) as u8);
                assert_eq!(cpu_1.ram.read(cpu_1.registers.get_sp() + 2), (expected_return_addr >> 8) as u8);
                // Flags untouched
                test_flags!(
                    cpu_1,
                    registers_copy.get_zero_flag(),
                    registers_copy.get_negative_flag(),
                    registers_copy.get_half_carry_flag(),
                    registers_copy.get_carry_flag()
                );
            }
        };
    }


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

    test_ld_r16_imm16!(0x01, test_0x01_ld_bc_imm16, set_bc, get_bc, get_b, get_c);

    test_ld_ar16_r8!(0x02, test_0x02_ld__bc__a, set_bc, get_bc, set_a, get_a);

    test_inc_r16!(0x03, test_0x03_inc_bc, set_bc, get_bc, get_b, get_c);

    test_inc_r8!(0x04, test_0x04_inc_b, set_b, get_b);

    test_dec_r8!(0x05, test_0x05_dec_b, set_b, get_b);

    test_ld_r8_imm8!(0x06, test_0x06_ld_b_imm8, set_b, get_b);

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

    test_dec_r16!(0x0B, test_0x0b_dec_bc, set_bc, get_bc, get_b, get_c);
    test_inc_r8!(0x0C, test_0x0c_inc_c, set_c, get_c);
    test_dec_r8!(0x0D, test_0x0d_dec_c, set_c, get_c);
    test_ld_r8_imm8!(0x0E, test_0x0e_ld_c_imm8, set_c, get_c);

    #[test]
    fn test_0x0f_rrca() {
        //No Flags
        let test_value_1: u8 = 0b0001_0001;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x0F, 0x0F];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cpu_1.registers.get_a(), 0b1000_1000);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_a(), 0b0100_0100);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    #[cfg(test)]
    fn test_0x10_stop() {
        // TODO: Study and implement STOP function
    }

    test_ld_r16_imm16!(0x11, test_0x11_ld_de_imm16, set_de, get_de, get_d, get_e);
    test_ld_ar16_r8!(0x12, test_0x12_ld__de__a, set_de, get_de, set_a, get_a);
    test_inc_r16!(0x13, test_0x13_inc_de, set_de, get_de, get_d, get_e);
    test_inc_r8!(0x14, test_0x14_inc_d, set_d, get_d);
    test_dec_r8!(0x15, test_0x15_dec_d, set_d, get_d);
    test_ld_r8_imm8!(0x16, test_0x16_ld_d_imm8, set_d, get_d);

    #[test]
    fn test_0x17_rla() {
        //No Flags
        let test_value_1: u8 = 0b1000_1000;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x17, 0x17];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(false);
        cpu_1.execute_next();
        // The re-entrance Bit is given by the previous content of C Flag
        assert_eq!(cpu_1.registers.get_a(), 0b0001_0000);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_a(), 0b0010_0001);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    #[test]
    fn test_0x18_jr_e8() {
        let mut test_value: i8 = -50;
        let mut start_address: i16 = 0x0350;
        let mut cpu = CPU::new();
        let mut program: Vec<u8> = vec![0x18, test_value as u8];
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        let mut cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);
    }

    #[test]
    fn test_0x19_add_hl_de() {
        //No Flags
        let mut test_value_1: u16 = 0xBD89;
        let mut test_value_2: u16 = 0x1029;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x19];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_de(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_de(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x7000;
        test_value_2 = 0x9000;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_de(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_de(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), 0);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        // H flags on ADD HL, rr should be on only carrying from bit 11 (check is made on H of HL)
        test_value_1 = 0x1070;
        test_value_2 = 0x1090;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_de(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_de(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x1700;
        test_value_2 = 0x1900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_de(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_de(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x9700;
        test_value_2 = 0x7900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_de(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_de(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1.wrapping_add(test_value_2));
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x1a_ld_a__de_() {
        let mut test_value_1: u8 = 0xBD;
        let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x1A];
        cpu_1.load(&program);
        cpu_1.registers.set_de(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        cpu_1.registers.set_a(0x11); // Sure different from expected value
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.ram.read(test_address_1), test_value_1);
        assert_eq!(cpu_1.registers.get_de(), test_address_1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
    }

    test_dec_r16!(0x1B, test_0x1b_dec_de, set_de, get_de, get_d, get_e);
    test_inc_r8!(0x1C, test_0x1c_inc_e, set_e, get_e);
    test_dec_r8!(0x1D, test_0x1d_dec_e, set_e, get_e);
    test_ld_r8_imm8!(0x1E, test_0x1e_ld_e_imm8, set_e, get_e);

    #[test]
    fn test_0x1f_rra() {
        //No Flags
        let test_value_1: u8 = 0b0001_0001;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x1F, 0x1F];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(false);
        cpu_1.execute_next();
        // The re-entrance Bit is given by the previous content of C Flag
        assert_eq!(cpu_1.registers.get_a(), 0b0000_1000);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
        cpu_1.execute_next();
        assert_eq!(cpu_1.registers.get_a(), 0b1000_0100);
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    #[test]
    fn test_0x20_jr_nz_e8() {
        let mut test_value: i8 = -50;
        let mut start_address: i16 = 0x0350;
        let mut cpu = CPU::new();
        let mut program: Vec<u8> = vec![0x20, test_value as u8];
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_zero_flag(false);
        let mut cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

        cpu = CPU::new();
        assert_eq!(cycles, 3);
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_zero_flag(true);
        cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_pc(), 0x352);
    }

    test_ld_r16_imm16!(0x21, test_0x21_ld_hl_imm16, set_hl, get_hl, get_h, get_l);

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

    test_inc_r16!(0x23, test_0x23_inc_hl, set_hl, get_hl, get_h, get_l);
    test_inc_r8!(0x24, test_0x24_inc_h, set_h, get_h);
    test_dec_r8!(0x25, test_0x25_dec_h, set_h, get_h);
    test_ld_r8_imm8!(0x26, test_0x26_ld_h_imm8, set_h, get_h);

    #[test]
    fn test_0x27_daa() {
        // TODO: Implement test for DAA and CBD values
    }

    #[test]
    fn test_0x28_jr_z_e8() {
        let mut test_value: i8 = -50;
        let mut start_address: i16 = 0x0350;
        let mut cpu = CPU::new();
        let mut program: Vec<u8> = vec![0x28, test_value as u8];
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_zero_flag(true);
        let mut cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

        cpu = CPU::new();
        assert_eq!(cycles, 3);
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_zero_flag(false);
        cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_pc(), 0x352);
    }

    #[test]
    fn test_0x29_add_hl_de() {
        let mut test_value: u16 = 0x1029;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x29];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_hl(), test_value + test_value);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value = 0x8000;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_hl(), 0);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        // H flags on ADD HL, rr should be on only carrying from bit 11 (check is made on H of HL)
        test_value = 0x1080;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value);
        cpu_1.registers.set_de(test_value);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_hl(), test_value + test_value);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value = 0x1800;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value);
        cpu_1.registers.set_de(test_value);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_hl(), test_value + test_value);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value = 0x8800;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value);
        cpu_1.registers.set_de(test_value);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_hl(), test_value.wrapping_add(test_value));
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x2a_ld_a__hli_() {
        let mut test_value_1: u8 = 0xBD;
        let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x2A];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        cpu_1.registers.set_a(0x11); // Sure different from expected value
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.ram.read(test_address_1), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1 + 1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
    }

    test_dec_r16!(0x2B, test_0x2b_dec_hl, set_hl, get_hl, get_h, get_l);
    test_inc_r8!(0x2C, test_0x2c_inc_l, set_l, get_l);
    test_dec_r8!(0x2D, test_0x2d_dec_l, set_l, get_l);
    test_ld_r8_imm8!(0x2E, test_0x2e_ld_l_imm8, set_l, get_l);

    #[test]
    fn test_0x2f_cpl() {
        //No Flags
        let test_value_1: u8 = 0xD4;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x2F];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_a(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), !test_value_1);
        // Z/C Flags untouched - N/H Flags on
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x30_jr_nc_e8() {
        let mut test_value: i8 = -50;
        let mut start_address: i16 = 0x0350;
        let mut cpu = CPU::new();
        let mut program: Vec<u8> = vec![0x30, test_value as u8];
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_carry_flag(false);
        let mut cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

        cpu = CPU::new();
        assert_eq!(cycles, 3);
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_carry_flag(true);
        cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_pc(), 0x352);
    }

    test_ld_r16_imm16!(0x31, test_0x31_ld_sp_imm16, set_sp, get_sp);

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

    test_inc_r16!(0x33, test_0x33_inc_sp, set_sp, get_sp);

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
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
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
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
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
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
        assert_eq!(cpu_2.registers.get_half_carry_flag(), true);

        // Test Underflow
        let test_value_4: u8 = 0x00;
        cpu_2 = CPU::new();
        cpu_2.load(&program_1);
        cpu_2.registers.set_hl(test_address);
        cpu_2.ram.write(test_address, test_value_4);
        cycle = cpu_2.execute_next();
        assert_eq!(cpu_2.ram.read(test_address), 0xFF);
        assert_eq!(cpu_2.registers.get_zero_flag(), false);
        assert_eq!(cpu_2.registers.get_negative_flag(), true);
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

    #[test]
    // SCF = Set Carry Flag
    fn test_0x37_scf() {
        //No Flags
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x37];
        cpu_1.load(&program_1);
        cpu_1.registers.set_carry_flag(false);
        let cycles = cpu_1.execute_next();
        // Check load data and FLAGs should be untouched
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x38_jr_c_e8() {
        let mut test_value: i8 = -50;
        let mut start_address: i16 = 0x0350;
        let mut cpu = CPU::new();
        let mut program: Vec<u8> = vec![0x38, test_value as u8];
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_carry_flag(true);
        let mut cycles = cpu.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

        cpu = CPU::new();
        assert_eq!(cycles, 3);
        cpu.load(&program);
        cpu.ram.write(0x0350, program[0]);
        cpu.ram.write(0x0351, program[1]);
        cpu.registers.set_pc(0x0350);
        cpu.registers.set_carry_flag(false);
        cycles = cpu.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.get_pc(), 0x352);
    }

    #[test]
    fn test_0x39_add_hl_sp() {
        //No Flags
        let mut test_value_1: u16 = 0xBD89;
        let mut test_value_2: u16 = 0x1029;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x39];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_sp(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_sp(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x7000;
        test_value_2 = 0x9000;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_sp(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_sp(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), 0);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        // H flags on ADD HL, rr should be on only carrying from bit 11 (check is made on H of HL)
        test_value_1 = 0x1070;
        test_value_2 = 0x1090;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_sp(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_sp(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x1700;
        test_value_2 = 0x1900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_sp(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_sp(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1 + test_value_2);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x9700;
        test_value_2 = 0x7900;
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_value_1);
        cpu_1.registers.set_sp(test_value_2);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_sp(), test_value_2);
        assert_eq!(cpu_1.registers.get_hl(), test_value_1.wrapping_add(test_value_2));
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x3a_ld_a__hld_() {
        let mut test_value_1: u8 = 0xBD;
        let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
        let mut cpu_1 = CPU::new();
        let program: Vec<u8> = vec![0x3A];
        cpu_1.load(&program);
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        cpu_1.registers.set_a(0x11); // Sure different from expected value
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.ram.read(test_address_1), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1 - 1);
        assert_eq!(cpu_1.registers.get_a(), test_value_1);
    }

    test_dec_r16!(0x3B, test_0x3b_dec_sp, set_sp, get_sp);
    test_inc_r8!(0x3C, test_0x3c_inc_a, set_a, get_a);
    test_dec_r8!(0x3D, test_0x3d_dec_a, set_a, get_a);
    test_ld_r8_imm8!(0x3E, test_0x3e_ld_a_imm8, set_a, get_a);

    #[test]
    fn test_0x3f_ccf() {
        // CCF = Complement Carry Flag
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x3F, 0x3F];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        // Z Flag untouched - N/H Flags off
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        // Z Flag untouched - N/H Flags off
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    // LD B, r8 | LD B, [HL]
    test_ld_r8!(0x40, test_0x40_ld_b_b, set_b, get_b);
    test_ld_r8!(0x41, test_0x41_ld_b_c, set_b, get_b, set_c, get_c);
    test_ld_r8!(0x42, test_0x42_ld_b_d, set_b, get_b, set_d, get_d);
    test_ld_r8!(0x43, test_0x43_ld_b_e, set_b, get_b, set_e, get_e);
    test_ld_r8!(0x44, test_0x44_ld_b_h, set_b, get_b, set_h, get_h);
    test_ld_r8!(0x45, test_0x45_ld_b_l, set_b, get_b, set_l, get_l);
    test_ld_ar16!(0x46, test_0x46_ld_b__hl_, set_b, get_b, set_hl, get_hl);
    test_ld_r8!(0x47, test_0x47_ld_b_a, set_b, get_b, set_a, get_a);

    // LD C, r8 | LD C, [HL]
    test_ld_r8!(0x48, test_0x48_ld_b_c, set_c, get_c, set_b, get_b);
    test_ld_r8!(0x49, test_0x49_ld_c_c, set_c, get_c);
    test_ld_r8!(0x4A, test_0x4a_ld_c_d, set_c, get_c, set_d, get_d);
    test_ld_r8!(0x4B, test_0x4b_ld_c_e, set_c, get_c, set_e, get_e);
    test_ld_r8!(0x4C, test_0x4c_ld_c_h, set_c, get_c, set_h, get_h);
    test_ld_r8!(0x4D, test_0x4d_ld_c_l, set_c, get_c, set_l, get_l);
    test_ld_ar16!(0x4E, test_0x4e_ld_c__hl_, set_c, get_c, set_hl, get_hl);
    test_ld_r8!(0x4F, test_0x4f_ld_c_a, set_c, get_c, set_a, get_a);

    // LD D, r8 | LD D, [HL]
    test_ld_r8!(0x50, test_0x50_ld_d_b, set_d, get_d, set_b, get_b);
    test_ld_r8!(0x51, test_0x51_ld_d_c, set_d, get_d, set_c, get_c);
    test_ld_r8!(0x52, test_0x52_ld_d_d, set_d, get_d);
    test_ld_r8!(0x53, test_0x53_ld_d_e, set_d, get_d, set_e, get_e);
    test_ld_r8!(0x54, test_0x54_ld_d_h, set_d, get_d, set_h, get_h);
    test_ld_r8!(0x55, test_0x55_ld_d_l, set_d, get_d, set_l, get_l);
    test_ld_ar16!(0x56, test_0x56_ld_d__hl_, set_d, get_d, set_hl, get_hl);
    test_ld_r8!(0x57, test_0x57_ld_d_a, set_d, get_d, set_a, get_a);

    // LD E, r8 | LD E, [HL]
    test_ld_r8!(0x58, test_0x58_ld_e_b, set_e, get_e, set_b, get_b);
    test_ld_r8!(0x59, test_0x59_ld_e_c, set_e, get_e, set_c, get_c);
    test_ld_r8!(0x5A, test_0x5a_ld_e_d, set_e, get_e, set_d, get_d);
    test_ld_r8!(0x5B, test_0x5b_ld_e_e, set_e, get_e);
    test_ld_r8!(0x5C, test_0x5c_ld_e_h, set_e, get_e, set_h, get_h);
    test_ld_r8!(0x5D, test_0x5d_ld_e_l, set_e, get_e, set_l, get_l);
    test_ld_ar16!(0x5E, test_0x5e_ld_e__hl_, set_e, get_e, set_hl, get_hl);
    test_ld_r8!(0x5F, test_0x5f_ld_e_a, set_e, get_e, set_a, get_a);

    // LD H, r8 | LD H, [HL]
    test_ld_r8!(0x60, test_0x60_ld_h_b, set_h, get_h, set_b, get_b);
    test_ld_r8!(0x61, test_0x61_ld_h_c, set_h, get_h, set_c, get_c);
    test_ld_r8!(0x62, test_0x62_ld_h_d, set_h, get_h, set_d, get_d);
    test_ld_r8!(0x63, test_0x63_ld_h_e, set_h, get_h, set_e, get_e);
    test_ld_r8!(0x64, test_0x64_ld_h_h, set_h, get_h);
    test_ld_r8!(0x65, test_0x65_ld_h_l, set_h, get_h, set_l, get_l);
    #[test]
    fn test_0x66_ld_h__hl_() {
        let test_value_1: u8 = 0xFF;
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x66];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_h(), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1 & 0xFF | (test_value_1 as u16) << 8);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }
    test_ld_r8!(0x67, test_0x67_ld_h_a, set_h, get_h, set_a, get_a);

    // LD E, r8 | LD E, [HL]
    test_ld_r8!(0x68, test_0x68_ld_l_b, set_l, get_l, set_b, get_b);
    test_ld_r8!(0x69, test_0x69_ld_l_c, set_l, get_l, set_c, get_c);
    test_ld_r8!(0x6A, test_0x6a_ld_l_d, set_l, get_l, set_d, get_d);
    test_ld_r8!(0x6B, test_0x6b_ld_l_e, set_l, get_l, set_e, get_e);
    test_ld_r8!(0x6C, test_0x6c_ld_l_h, set_l, get_l, set_h, get_h);
    test_ld_r8!(0x6D, test_0x6d_ld_l_l, set_l, get_l);
    #[test]
    fn test_0x6e_ld_l__hl_() {
        let test_value_1: u8 = 0xFF;
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x6E];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, test_value_1);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_l(), test_value_1);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1 & 0xFF00 | (test_value_1 as u16));
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }
    test_ld_r8!(0x6F, test_0x6f_ld_l_a, set_l, get_l, set_a, get_a);

    // LD [HL], r8
    test_ld_ar16_r8!(0x70, test_0x70_ld__hl__b, set_hl, get_hl, set_b, get_b);
    test_ld_ar16_r8!(0x71, test_0x71_ld__hl__c, set_hl, get_hl, set_c, get_c);
    test_ld_ar16_r8!(0x72, test_0x72_ld__hl__d, set_hl, get_hl, set_d, get_d);
    test_ld_ar16_r8!(0x73, test_0x73_ld__hl__e, set_hl, get_hl, set_e, get_e);

    #[test]
    fn test_0x74_ld__hl__h() {
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
        let expected_value = test_address_1.wrapping_shr(8) as u8;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x74];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, 0x00);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_h(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1);
        assert_eq!(cpu_1.ram.read(test_address_1), expected_value);
        // Flags untouched
        test_flags!(
            cpu_1,
            register_copy.get_zero_flag(),
            register_copy.get_negative_flag(),
            register_copy.get_half_carry_flag(),
            register_copy.get_carry_flag()
        );
    }

    #[test]
    fn test_0x75_ld__hl__l() {
        let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x99;
        let expected_value = (test_address_1 & 0xFF) as u8;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x75];
        cpu_1.load(&program_1);
        let register_copy = cpu_1.registers;
        cpu_1.registers.set_hl(test_address_1);
        cpu_1.ram.write(test_address_1, 0x00);
        let cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_l(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address_1);
        assert_eq!(cpu_1.ram.read(test_address_1), expected_value);
        // Flags untouched
        assert_eq!(cpu_1.registers.get_zero_flag(), register_copy.get_zero_flag());
        assert_eq!(cpu_1.registers.get_negative_flag(), register_copy.get_negative_flag());
        assert_eq!(cpu_1.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
        assert_eq!(cpu_1.registers.get_carry_flag(), register_copy.get_carry_flag());
    }

    #[test]
    fn test_0x76_halt() {
        // TODO: Study and then implement HALT
    }

    test_ld_ar16_r8!(0x77, test_0x77_ld__hl__a, set_hl, get_hl, set_a, get_a);

    // LD A, r8 | LD A, [HL]
    test_ld_r8!(0x78, test_0x78_ld_a_b, set_a, get_a, set_b, get_b);
    test_ld_r8!(0x79, test_0x79_ld_a_c, set_a, get_a, set_c, get_c);
    test_ld_r8!(0x7A, test_0x7a_ld_a_d, set_a, get_a, set_d, get_d);
    test_ld_r8!(0x7B, test_0x7b_ld_a_e, set_a, get_a, set_e, get_e);
    test_ld_r8!(0x7C, test_0x7c_ld_a_h, set_a, get_a, set_h, get_h);
    test_ld_r8!(0x7D, test_0x7d_ld_a_l, set_a, get_a, set_l, get_l);
    test_ld_ar16!(0x7E, test_0x7e_ld_a__hl_, set_a, get_a, set_hl, get_hl);
    test_ld_r8!(0x7F, test_0x7f_ld_a_a, set_a, get_a);

    // ADD A, r8 | ADD A, [HL]
    test_add_r8_r8!(0x80, test_0x80_add_a_b, set_a, get_a, set_b, get_b);
    test_add_r8_r8!(0x81, test_0x81_add_a_c, set_a, get_a, set_c, get_c);
    test_add_r8_r8!(0x82, test_0x82_add_a_d, set_a, get_a, set_d, get_d);
    test_add_r8_r8!(0x83, test_0x83_add_a_e, set_a, get_a, set_e, get_e);
    test_add_r8_r8!(0x84, test_0x84_add_a_h, set_a, get_a, set_h, get_h);
    test_add_r8_r8!(0x85, test_0x85_add_a_l, set_a, get_a, set_l, get_l);
    #[test]
    fn test_0x86_add_a__hl_() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x16;
        let test_address: u16 = WRAM_ADDRESS as u16 + 0xDD;
        let mut expected_value: u8 = test_value_1.wrapping_add(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x86];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.ram.read(test_address), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0x10;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        assert_eq!(cpu_1.ram.read(test_address), test_value_2);
        // Z/C Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x0F;
        test_value_2 = 0x01;
        expected_value = 0x10;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        assert_eq!(cpu_1.ram.read(test_address), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xFF;
        test_value_2 = 0x01;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        assert_eq!(cpu_1.ram.read(test_address), test_value_2);
        // Z/H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }
    #[test]
    fn test_0x87_add_a_a() {
        let mut test_value_1: u8 = 0x24;
        let mut expected_value: u8 = test_value_1.wrapping_add(test_value_1);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x87];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x80;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // Z/C Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x08;
        expected_value = 0x10;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xFF;
        expected_value = 0xFE;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // Z/H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    // ADC A, R8 | ADC A, [HL]
    test_adc_r8_r8!(0x88, test_0x88_adc_a_b__c_off, set_a, get_a, set_b, get_b, false);
    test_adc_r8_r8!(0x88, test_0x88_adc_a_b__c_on, set_a, get_a, set_b, get_b, true);
    test_adc_r8_r8!(0x89, test_0x89_adc_a_c__c_off, set_a, get_a, set_c, get_c, false);
    test_adc_r8_r8!(0x89, test_0x89_adc_a_c__c_on, set_a, get_a, set_c, get_c, true);
    test_adc_r8_r8!(0x8A, test_0x8a_adc_a_d__c_off, set_a, get_a, set_d, get_d, false);
    test_adc_r8_r8!(0x8A, test_0x8a_adc_a_d__c_on, set_a, get_a, set_d, get_d, true);
    test_adc_r8_r8!(0x8B, test_0x8b_adc_a_e__c_off, set_a, get_a, set_e, get_e, false);
    test_adc_r8_r8!(0x8B, test_0x8b_adc_a_e__c_on, set_a, get_a, set_e, get_e, true);
    test_adc_r8_r8!(0x8C, test_0x8c_adc_a_h__c_off, set_a, get_a, set_h, get_h, false);
    test_adc_r8_r8!(0x8C, test_0x8c_adc_a_h__c_on, set_a, get_a, set_h, get_h, true);
    test_adc_r8_r8!(0x8D, test_0x8d_adc_a_l__c_off, set_a, get_a, set_l, get_l, false);
    test_adc_r8_r8!(0x8D, test_0x8d_adc_a_l__c_on, set_a, get_a, set_l, get_l, true);

    #[test]
    fn test_0x8e_adc_a__hl___c_off() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x16;
        let test_address: u16 = WRAM_ADDRESS as u16 + 0xAA;
        let mut expected_value: u8 = test_value_1.wrapping_add(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x8E];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0x10;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        // Z/C Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x0F;
        test_value_2 = 0x01;
        expected_value = 0x10;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xFF;
        test_value_2 = 0x01;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        // Z/H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x8e_adc_a__hl___c_on() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x16;
        let test_address: u16 = WRAM_ADDRESS as u16 + 0xAA;
        let mut expected_value: u8 = test_value_1.wrapping_add(test_value_2 + 1);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x8E];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        cpu_1.registers.set_carry_flag(true);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        // Z/C Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x0D;
        test_value_2 = 0x02;
        expected_value = 0x10;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xFE;
        test_value_2 = 0x01;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        // Z/H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x8f_adc_a_a__c_off() {
        let mut test_value_1: u8 = 0x16;
        let mut expected_value: u8 = test_value_1.wrapping_add(test_value_1);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x8F];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x80;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // Z/C Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x08;
        expected_value = 0x10;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x88;
        expected_value = 0x10;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // Z/H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x8f_adc_a_a__c_on() {
        let mut test_value_1: u8 = 0x16;
        let mut expected_value: u8 = test_value_1.wrapping_add(test_value_1 + 1);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x8F];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(true);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x80;
        expected_value = 0x01;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // C Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x08;
        expected_value = 0x11;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x88;
        expected_value = 0x11;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    // SUB A, r8 | SUB A, [HL]
    #[test]
    fn test_0x90_sub_a_b() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x11;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x90];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0xF0;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x01;
        expected_value = 0x0F;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x20;
        expected_value = test_value_1.wrapping_sub(test_value_2);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x91_sub_a_c() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x11;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x91];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0xF0;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x01;
        expected_value = 0x0F;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x20;
        expected_value = test_value_1.wrapping_sub(test_value_2);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x92_sub_a_d() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x11;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x92];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0xF0;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x01;
        expected_value = 0x0F;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x20;
        expected_value = test_value_1.wrapping_sub(test_value_2);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x93_sub_a_e() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x11;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x93];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0xF0;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x01;
        expected_value = 0x0F;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x20;
        expected_value = test_value_1.wrapping_sub(test_value_2);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x94_sub_a_h() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x11;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x94];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0xF0;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x01;
        expected_value = 0x0F;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x20;
        expected_value = test_value_1.wrapping_sub(test_value_2);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x95_sub_a_l() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x11;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x95];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0xF0;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x01;
        expected_value = 0x0F;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x20;
        expected_value = test_value_1.wrapping_sub(test_value_2);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x96_sub_a__hl_() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x11;
        let test_address: u16 = WRAM_ADDRESS as u16 + 0x55;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x96];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0xF0;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x01;
        expected_value = 0x0F;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x20;
        expected_value = test_value_1.wrapping_sub(test_value_2);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(test_address, test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_hl(), test_address);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x97_sub_a_a() {
        let mut test_value_1: u8 = 0xC4;
        let mut expected_value: u8 = 0;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x97];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // Z/N Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // Z/N Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    #[test]
    fn test_0x98_sbc_a_b__c_off() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x12;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x98];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x0F;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0x01;
        expected_value = 0xEF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x98_sbc_a_b__c_on() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x13;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2 + 1);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x98];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0E;
        expected_value = test_value_1.wrapping_sub(test_value_2 + 1);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // Z/H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x00;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_b(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x99_sbc_a_c__c_off() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x12;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x99];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x0F;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0x01;
        expected_value = 0xEF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x99_sbc_a_c__c_on() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x13;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2 + 1);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x99];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0E;
        expected_value = test_value_1.wrapping_sub(test_value_2 + 1);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // Z/H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x00;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_c(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_c(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x9a_sbc_a_d__c_off() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x12;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9A];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x0F;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0x01;
        expected_value = 0xEF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x9a_sbc_a_d__c_on() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x13;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2 + 1);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9A];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0E;
        expected_value = test_value_1.wrapping_sub(test_value_2 + 1);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // Z/H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x00;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_d(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_d(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x9b_sbc_a_e__c_off() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x12;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9B];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x0F;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0x01;
        expected_value = 0xEF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x9b_sbc_a_e__c_on() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x13;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2 + 1);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9B];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0E;
        expected_value = test_value_1.wrapping_sub(test_value_2 + 1);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // Z/H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x00;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_e(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_e(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x9c_sbc_a_h__c_off() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x12;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9C];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x0F;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0x01;
        expected_value = 0xEF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x9c_sbc_a_h__c_on() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x13;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2 + 1);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9C];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0E;
        expected_value = test_value_1.wrapping_sub(test_value_2 + 1);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // Z/H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x00;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_h(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_h(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x9d_sbc_a_l__c_off() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x12;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9D];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x0F;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0x01;
        expected_value = 0xEF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x9d_sbc_a_l__c_on() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x13;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2 + 1);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9D];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0E;
        expected_value = test_value_1.wrapping_sub(test_value_2 + 1);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // Z/H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x00;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_l(test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.registers.get_l(), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x9e_sbc_a__hl___c_off() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x12;
        let test_address: u16 = WRAM_ADDRESS as u16 + 0x11;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9E];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(cpu_1.registers.get_hl(), test_value_2);
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x0F;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(cpu_1.registers.get_hl(), test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // Z Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        test_value_2 = 0x01;
        expected_value = 0xEF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(cpu_1.registers.get_hl(), test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(cpu_1.registers.get_hl(), test_value_2);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x9e_sbc_a__hl___c_on() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x13;
        let test_address: u16 = WRAM_ADDRESS as u16 + 0x11;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2 + 1);
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9E];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(cpu_1.registers.get_hl(), test_value_2);
        cpu_1.registers.set_carry_flag(true);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0E;
        expected_value = test_value_1.wrapping_sub(test_value_2 + 1);
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(cpu_1.registers.get_hl(), test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x10;
        test_value_2 = 0x0F;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(cpu_1.registers.get_hl(), test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // Z/H Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x00;
        test_value_2 = 0x00;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_hl(test_address);
        cpu_1.ram.write(cpu_1.registers.get_hl(), test_value_2);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        assert_eq!(cpu_1.ram.read(cpu_1.registers.get_hl()), test_value_2);
        // H/C Flag
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    #[test]
    fn test_0x9f_sbc_a_a__c_off() {
        let mut test_value_1: u8 = 0xC4;
        let mut expected_value: u8 = 0;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9F];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(false);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // Z/N Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xF0;
        expected_value = 0x00;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(false);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // Z/N Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), true);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);
    }

    #[test]
    fn test_0x9f_sbc_a_a__c_on() {
        let mut test_value_1: u8 = 0xC4;
        let mut expected_value: u8 = 0xFF;
        let mut cpu_1 = CPU::new();
        let program_1: Vec<u8> = vec![0x9F];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(true);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // Z/N Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0xF0;
        expected_value = 0xFF;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_carry_flag(true);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // Z/N Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), true);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }

    // AND A, r8
    test_and_a_r8!(0xA0, test_0xa0_and_a_b, set_b, get_b);
    test_and_a_r8!(0xA1, test_0xa1_and_a_c, set_c, get_c);
    test_and_a_r8!(0xA2, test_0xa2_and_a_d, set_d, get_d);
    test_and_a_r8!(0xA3, test_0xa3_and_a_e, set_e, get_e);
    test_and_a_r8!(0xA4, test_0xa4_and_a_h, set_h, get_h);
    test_and_a_r8!(0xA5, test_0xa5_and_a_l, set_l, get_l);
    test_and_a_r8!(0xA6, test_0xa6_and_a__hl_, hl);
    test_and_a_r8!(0xA7, test_0xa7_and_a_a, a);

    // XOR A, r8
    test_xor_a_r8!(0xA8, test_0xa8_xor_a_b, set_b, get_b);
    test_xor_a_r8!(0xA9, test_0xa9_xor_a_c, set_c, get_c);
    test_xor_a_r8!(0xAA, test_0xaa_xor_a_d, set_d, get_d);
    test_xor_a_r8!(0xAB, test_0xab_xor_a_e, set_e, get_e);
    test_xor_a_r8!(0xAC, test_0xac_xor_a_h, set_h, get_h);
    test_xor_a_r8!(0xAD, test_0xad_xor_a_l, set_l, get_l);
    test_xor_a_r8!(0xAE, test_0xae_xor_a__hl_, hl);
    test_xor_a_r8!(0xAF, test_0xaf_xor_a_a, a);

    // OR A, r8
    test_or_a_r8!(0xB0, test_0xb0_or_a_b, set_b, get_b);
    test_or_a_r8!(0xB1, test_0xb1_or_a_c, set_c, get_c);
    test_or_a_r8!(0xB2, test_0xb2_or_a_d, set_d, get_d);
    test_or_a_r8!(0xB3, test_0xb3_or_a_e, set_e, get_e);
    test_or_a_r8!(0xB4, test_0xb4_or_a_h, set_h, get_h);
    test_or_a_r8!(0xB5, test_0xb5_or_a_l, set_l, get_l);
    test_or_a_r8!(0xB6, test_0xb6_or_a__hl_, hl);
    test_or_a_r8!(0xB7, test_0xb7_or_a_a, a);

    // CP A, r8
    test_cp_a_r8!(0xB8, test_0xb8_cp_a_b, set_b, get_b);
    test_cp_a_r8!(0xB9, test_0xb9_cp_a_c, set_c, get_c);
    test_cp_a_r8!(0xBA, test_0xba_cp_a_d, set_d, get_d);
    test_cp_a_r8!(0xBB, test_0xbb_cp_a_e, set_e, get_e);
    test_cp_a_r8!(0xBC, test_0xbc_cp_a_h, set_h, get_h);
    test_cp_a_r8!(0xBD, test_0xbd_cp_a_l, set_l, get_l);
    test_cp_a_r8!(0xBE, test_0xbe_cp_a__hl_, hl);
    test_cp_a_r8!(0xBF, test_0xbf_cp_a_a, a);

    // 0xC* Row
    test_ret!(0xC0, test_0xc0_ret_nz, true, set_zero_flag, get_zero_flag);
    test_pop!(0xC1, test_0xc1_pop_bc, set_bc, get_bc);
    test_jump!(0xC2, test_0xc2_jp_nz_imm16, true, set_zero_flag, get_zero_flag);
    test_jump!(0xC3, test_0xc3_jp_imm8);
    test_call!(0xC4, test_0xc4_call_nz_imm16, true, set_zero_flag, get_zero_flag);
    test_push!(0xC5, test_0xc5_push_bc, set_bc, get_bc);

    // 0xC* Row
    test_add_r8_imm8!(0xC6, test_0xc6_add_a_imm8, set_a, get_a);
    test_rst!(0xC7, test_0xc7_rst_00);
    test_ret!(0xC8, test_0xc8_ret_n, false, set_zero_flag, get_zero_flag);
    test_ret!(0xC9, test_0xc9_ret);
    test_jump!(0xCA, test_0xca_jp_z_imm16, false, set_zero_flag, get_zero_flag);
    test_call!(0xCC, test_0xcc_call_z_imm16, false, set_zero_flag, get_zero_flag);
    test_call!(0xCD, test_0xcd_call_imm16);
    test_adc_r8_imm8!(0xCE, test_0xce_adc_a_imm8__c_off, set_a, get_a, false);
    test_adc_r8_imm8!(0xCE, test_0xce_adc_a_imm8__c_on, set_a, get_a, true);
    test_rst!(0xCF, test_0xcf_rst_08);

    // 0xD* Row
    test_ret!(0xD0, test_0xd0_ret_nc, true, set_carry_flag, get_carry_flag);
    test_pop!(0xD1, test_0xd1_pop_de, set_de, get_de);
    test_jump!(0xD2, test_0xd2_jp_nc_imm16, true, set_carry_flag, get_carry_flag);
    test_call!(0xD4, test_0xd4_call_nc_imm16, true, set_carry_flag, get_carry_flag);
    test_push!(0xD5, test_0xd5_push_de, set_de, get_de);
    #[test]
    fn test_0xd6_sub_a_imm8() {
        let mut test_value_1: u8 = 0xC4;
        let mut test_value_2: u8 = 0x11;
        let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
        let mut cpu_1 = CPU::new();
        let mut program_1: Vec<u8> = vec![0xD6, test_value_2];
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // N Flag
        test_flags!(cpu_1, false, true, false, false);

        test_value_1 = 0xF0;
        test_value_2 = 0xF0;
        expected_value = 0x00;
        program_1[1] = test_value_2;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cpu_1.registers.set_b(test_value_2);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // Z/N Flags
        test_flags!(cpu_1, true, true, false, false);

        test_value_1 = 0x10;
        test_value_2 = 0x01;
        expected_value = 0x0F;
        program_1[1] = test_value_2;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // N/H Flag
        test_flags!(cpu_1, false, true, true, false);

        test_value_1 = 0x10;
        test_value_2 = 0x20;
        expected_value = test_value_1.wrapping_sub(test_value_2);
        program_1[1] = test_value_2;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // C Flag
        test_flags!(cpu_1, false, true, false, true);

        test_value_1 = 0x00;
        test_value_2 = 0x01;
        expected_value = 0xFF;
        program_1[1] = test_value_2;
        cpu_1 = CPU::new();
        cpu_1.load(&program_1);
        cpu_1.registers.set_a(test_value_1);
        cycles = cpu_1.execute_next();
        assert_eq!(cycles, 2);
        assert_eq!(cpu_1.registers.get_a(), expected_value);
        // H/C Flag
        test_flags!(cpu_1, false, true, true, true);
    }
    test_rst!(0xD7, test_0xd7_rst_10);
    test_ret!(0xD8, test_0xd8_ret_c, false, set_carry_flag, get_carry_flag);
    // TODO: test for RETI
    test_jump!(0xDA, test_0xda_jp_c_imm16, false, set_carry_flag, get_carry_flag);
    test_call!(0xDC, test_0xdc_call_c_imm16, false, set_carry_flag, get_carry_flag);
    test_sbc_r8_imm8!(0xDE, test_0xde_sbc_a_imm8__c_off, set_a, get_a, false);
    test_sbc_r8_imm8!(0xDE, test_0xde_sbc_a_imm8__c_on, set_a, get_a, true);
    test_rst!(0xDF, test_0xcf_rst_18);

    // 0XE* Row
    test_ldh_r8_imm8!(0xE0, test_0xe0_ldh__imm8__a, set_a, get_a, false);
    test_pop!(0xE1, test_0xe1_pop_hl, set_hl, get_hl);
    test_ldh_r8_r8!(0xE2, test_0xe2_ldh__c__a, set_a, get_a, set_c, get_c, false);
    test_push!(0xE5, test_0xe5_push_hl, set_hl, get_hl);
    test_and_a_imm8!(0xE6, test_0xe6_and_a_imm8);
    test_rst!(0xE7, test_0xe7_rst_20);
    #[test]
    fn test_0xe8_add_sp_e8() {
        let mut test_value_1: u16 = 0xC4C4;
        let mut test_value_2: i8 = 0x12;
        let mut test_value_2_abs: u16 = 0x12;
        let mut expected_value: u16 = test_value_1.wrapping_add(test_value_2_abs);
        let mut cpu_1 = CPU::new();
        let mut program_1: Vec<u8> = vec![0xE8, test_value_2 as u8];
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 4);
        assert_eq!(cpu_1.registers.get_sp(), expected_value);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x0FFF;
        test_value_2 = 0x01;
        test_value_2_abs = 0x01;
        expected_value = test_value_1.wrapping_add(test_value_2_abs);
        let mut cpu_1 = CPU::new();
        program_1[1] = test_value_2 as u8;
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 4);
        assert_eq!(cpu_1.registers.get_sp(), expected_value);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xFFFF;
        test_value_2 = 0x01;
        test_value_2_abs = 0x01;
        expected_value = 0x0;
        let mut cpu_1 = CPU::new();
        program_1[1] = test_value_2 as u8;
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 4);
        assert_eq!(cpu_1.registers.get_sp(), expected_value);
        // H/C Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0xDDFF;
        test_value_2 = -128;
        test_value_2_abs = 128;
        expected_value = test_value_1.wrapping_sub(test_value_2_abs);
        let mut cpu_1 = CPU::new();
        program_1[1] = test_value_2 as u8;
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 4);
        assert_eq!(cpu_1.registers.get_sp(), expected_value);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xD000;
        test_value_2 = -1;
        test_value_2_abs = 1;
        expected_value = test_value_1.wrapping_sub(test_value_2_abs);
        let mut cpu_1 = CPU::new();
        program_1[1] = test_value_2 as u8;
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 4);
        assert_eq!(cpu_1.registers.get_sp(), expected_value);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x0000;
        test_value_2 = -1;
        test_value_2_abs = 1;
        expected_value = test_value_1.wrapping_sub(test_value_2_abs);
        let mut cpu_1 = CPU::new();
        program_1[1] = test_value_2 as u8;
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 4);
        assert_eq!(cpu_1.registers.get_sp(), expected_value);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }
    test_jump!(0xE9, test_0xe9_jp_hl, set_hl, get_hl);
    test_ld_imm16_r8!(0xEA, test_0xea_ld__imm16__a, set_a, get_a);
    test_xor_a_imm8!(0xEE, test_0xee_xor_a_imm8);
    test_rst!(0xEF, test_0xef_rst_28);

    // 0XF* Row
    test_ldh_r8_imm8!(0xF0, test_0xf0_ldh_a__imm8_, set_a, get_a, true);
    test_pop!(0xF1, test_0xf1_pop_af, set_af, get_af);
    test_ldh_r8_r8!(0xF2, test_0xf2_ldh_a__c_, set_a, get_a, set_c, get_c, true);
    #[test]
    fn test_0xf3_di() {
        let mut cpu_1 = CPU::new();
        let mut program_1: Vec<u8> = vec![0xF3];
        cpu_1.load(&program_1);
        cpu_1.interrupts_enabled = true;
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.interrupts_enabled, false);
    }
    test_push!(0xF5, test_0xf5_push_af, set_af, get_af);
    test_or_a_imm8!(0xF6, test_0xf6_or_a_imm8);
    test_rst!(0xF7, test_0xf7_rst_30);
    #[test]
    fn test_0xf8_ld_hl_sp_e8() {
        let mut test_value_1: u16 = 0xC412;
        let mut test_value_2: i8 = 0x12;
        let mut test_value_2_abs: u16 = 0x12;
        let mut expected_value: u16 = test_value_1.wrapping_add(test_value_2_abs);
        let mut cpu_1 = CPU::new();
        let mut program_1: Vec<u8> = vec![0xF8, test_value_2 as u8];
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu_1.registers.get_hl(), expected_value);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x0FFF;
        test_value_2 = 0x01;
        test_value_2_abs = 0x01;
        expected_value = test_value_1.wrapping_add(test_value_2_abs);
        let mut cpu_1 = CPU::new();
        program_1[1] = test_value_2 as u8;
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu_1.registers.get_hl(), expected_value);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xFFFF;
        test_value_2 = 0x01;
        test_value_2_abs = 0x01;
        expected_value = 0x0;
        let mut cpu_1 = CPU::new();
        program_1[1] = test_value_2 as u8;
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu_1.registers.get_hl(), expected_value);
        // H/C Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);

        test_value_1 = 0xDDFF;
        test_value_2 = -128;
        test_value_2_abs = 128;
        expected_value = test_value_1.wrapping_sub(test_value_2_abs);
        let mut cpu_1 = CPU::new();
        program_1[1] = test_value_2 as u8;
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu_1.registers.get_hl(), expected_value);
        // No Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), false);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0xD000;
        test_value_2 = -1;
        test_value_2_abs = 1;
        expected_value = test_value_1.wrapping_sub(test_value_2_abs);
        let mut cpu_1 = CPU::new();
        program_1[1] = test_value_2 as u8;
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu_1.registers.get_hl(), expected_value);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), false);

        test_value_1 = 0x0000;
        test_value_2 = -1;
        test_value_2_abs = 1;
        expected_value = test_value_1.wrapping_sub(test_value_2_abs);
        let mut cpu_1 = CPU::new();
        program_1[1] = test_value_2 as u8;
        cpu_1.load(&program_1);
        cpu_1.registers.set_sp(test_value_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 3);
        assert_eq!(cpu_1.registers.get_hl(), expected_value);
        // H Flags
        assert_eq!(cpu_1.registers.get_zero_flag(), false);
        assert_eq!(cpu_1.registers.get_negative_flag(), false);
        assert_eq!(cpu_1.registers.get_half_carry_flag(), true);
        assert_eq!(cpu_1.registers.get_carry_flag(), true);
    }
    test_ld_r16!(0xF9, test_0xf9_ld_sp_hl, set_sp, get_sp, set_hl, get_hl);
    test_ld_r8_imm16!(0xFA, test_0xfa_ld__imm16__a, set_a, get_a);
    #[test]
    fn test_0xfb_ei() {
        let mut cpu_1 = CPU::new();
        let mut program_1: Vec<u8> = vec![0xFB];
        cpu_1.load(&program_1);
        let mut cycles = cpu_1.execute_next();
        assert_eq!(cycles, 1);
        assert_eq!(cpu_1.interrupts_enabled, true);
    }
    test_cp_a_imm8!(0xFE, test_0xfe_cp_a_imm8);
    test_rst!(0xFF, test_0xef_rst_38);
}

#[cfg(test)]
mod test_cb {
    use crate::GB::CPU::CPU;
    use crate::GB::RAM;
    use crate::GB::RAM::{USER_PROGRAM_ADDRESS, WRAM_ADDRESS};

    macro_rules! test_flags {
        ($cpu:ident, $zero:expr, $negative:expr, $half:expr, $carry:expr) => {
            assert_eq!($cpu.registers.get_zero_flag(), $zero);
            assert_eq!($cpu.registers.get_negative_flag(), $negative);
            assert_eq!($cpu.registers.get_half_carry_flag(), $half);
            assert_eq!($cpu.registers.get_carry_flag(), $carry);
        };
    }

    macro_rules! test_rlc {
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b1000_1000;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_src(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b0001_0001);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b0010_0010);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b1000_1000;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.ram.write(test_addr, test_value_1);
                cpu_1.registers.set_hl(test_addr);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b0001_0001);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b0010_0010);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
    }
    macro_rules! test_rl {
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b1000_1000;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_src(test_value_1);
                cpu_1.registers.set_carry_flag(false);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b0001_0000);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b0010_0001);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b1000_1000;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.ram.write(test_addr, test_value_1);
                cpu_1.registers.set_hl(test_addr);
                cpu_1.registers.set_carry_flag(false);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b0001_0000);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b0010_0001);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
    }
    macro_rules! test_sla {
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b1000_1000;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_src(test_value_1);
                cpu_1.registers.set_carry_flag(false);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b0001_0000);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b0010_0000);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b1000_1000;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.ram.write(test_addr, test_value_1);
                cpu_1.registers.set_hl(test_addr);
                cpu_1.registers.set_carry_flag(false);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b0001_0000);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b0010_0000);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
    }

    macro_rules! test_rrc {
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b0001_0001;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_src(test_value_1);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b1000_1000);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b0100_0100);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b0001_0001;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.ram.write(test_addr, test_value_1);
                cpu_1.registers.set_hl(test_addr);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b1000_1000);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b0100_0100);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
    }
    macro_rules! test_rr {
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b0001_0001;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_src(test_value_1);
                cpu_1.registers.set_carry_flag(false);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b0000_1000);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b1000_0100);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b0001_0001;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.ram.write(test_addr, test_value_1);
                cpu_1.registers.set_hl(test_addr);
                cpu_1.registers.set_carry_flag(false);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b0000_1000);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b1000_0100);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
    }
    macro_rules! test_sra {
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b0001_0001;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_src(test_value_1);
                cpu_1.registers.set_carry_flag(false);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b0000_1000);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b0000_0100);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b0001_0001;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.ram.write(test_addr, test_value_1);
                cpu_1.registers.set_hl(test_addr);
                cpu_1.registers.set_carry_flag(false);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b0000_1000);
                test_flags!(cpu_1, false, false, false, true);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b0000_0100);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
    }

    macro_rules! test_swap {
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b0001_1000;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.registers.$set_reg_src(test_value_1);
                cpu_1.registers.set_carry_flag(false);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), 0b1000_0001);
                test_flags!(cpu_1, false, false, false, false);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu_1.registers.$get_reg_src(), test_value_1);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, $set_reg_src:ident, $get_reg_src:ident, memory) => {
            #[test]
            fn $func() {
                let test_value_1: u8 = 0b0001_1000;
                let test_addr: u16 = WRAM_ADDRESS as u16 + 0xC6;
                let mut cpu_1 = CPU::new();
                let program_1: Vec<u8> = vec![0xCB, $opcode, 0xCB, $opcode];
                cpu_1.load(&program_1);
                cpu_1.ram.write(test_addr, test_value_1);
                cpu_1.registers.set_hl(test_addr);
                cpu_1.registers.set_carry_flag(false);
                let mut cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), 0b1000_0001);
                test_flags!(cpu_1, false, false, false, false);
                cycles = cpu_1.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu_1.ram.read(test_addr), test_value_1);
                test_flags!(cpu_1, false, false, false, false);
            }
        };
    }

    test_rlc!(0x00, test_0x00_rlc_b, set_b, get_b);
    test_rlc!(0x01, test_0x01_rlc_c, set_c, get_c);
    test_rlc!(0x02, test_0x02_rlc_d, set_d, get_d);
    test_rlc!(0x03, test_0x03_rlc_e, set_e, get_e);
    test_rlc!(0x04, test_0x04_rlc_h, set_h, get_h);
    test_rlc!(0x05, test_0x05_rlc_l, set_l, get_l);
    test_rlc!(0x06, test_0x06_rlc__hl_, set_hl, get_hl, memory);
    test_rlc!(0x07, test_0x07_rlc_a, set_a, get_a);

    test_rrc!(0x08, test_0x08_rrc_b, set_b, get_b);
    test_rrc!(0x09, test_0x09_rrc_c, set_c, get_c);
    test_rrc!(0x0A, test_0x0a_rrc_d, set_d, get_d);
    test_rrc!(0x0B, test_0x0b_rrc_e, set_e, get_e);
    test_rrc!(0x0C, test_0x0c_rrc_h, set_h, get_h);
    test_rrc!(0x0D, test_0x0d_rrc_l, set_l, get_l);
    test_rrc!(0x0E, test_0x0e_rrc__hl_, set_hl, get_hl, memory);
    test_rrc!(0x0F, test_0x0f_rrc_a, set_a, get_a);

    test_rl!(0x10, test_0x10_rl_b, set_b, get_b);
    test_rl!(0x11, test_0x11_rl_c, set_c, get_c);
    test_rl!(0x12, test_0x12_rl_d, set_d, get_d);
    test_rl!(0x13, test_0x13_rl_e, set_e, get_e);
    test_rl!(0x14, test_0x14_rl_h, set_h, get_h);
    test_rl!(0x15, test_0x15_rl_l, set_l, get_l);
    test_rl!(0x16, test_0x16_rl__hl_, set_hl, get_hl, memory);
    test_rl!(0x17, test_0x17_rl_a, set_a, get_a);

    test_rr!(0x18, test_0x18_rr_b, set_b, get_b);
    test_rr!(0x19, test_0x19_rr_c, set_c, get_c);
    test_rr!(0x1A, test_0x1a_rr_d, set_d, get_d);
    test_rr!(0x1B, test_0x1b_rr_e, set_e, get_e);
    test_rr!(0x1C, test_0x1c_rr_h, set_h, get_h);
    test_rr!(0x1D, test_0x1d_rr_l, set_l, get_l);
    test_rr!(0x1E, test_0x1e_rr__hl_, set_hl, get_hl, memory);
    test_rr!(0x1F, test_0x1f_rr_a, set_a, get_a);

    test_sla!(0x20, test_0x20_sla_b, set_b, get_b);
    test_sla!(0x21, test_0x21_sla_c, set_c, get_c);
    test_sla!(0x22, test_0x22_sla_d, set_d, get_d);
    test_sla!(0x23, test_0x23_sla_e, set_e, get_e);
    test_sla!(0x24, test_0x24_sla_h, set_h, get_h);
    test_sla!(0x25, test_0x25_sla_l, set_l, get_l);
    test_sla!(0x26, test_0x26_sla__hl_, set_hl, get_hl, memory);
    test_sla!(0x27, test_0x27_sla_a, set_a, get_a);

    test_sra!(0x28, test_0x28_sra_b, set_b, get_b);
    test_sra!(0x29, test_0x29_sra_c, set_c, get_c);
    test_sra!(0x2A, test_0x2a_sra_d, set_d, get_d);
    test_sra!(0x2B, test_0x2b_sra_e, set_e, get_e);
    test_sra!(0x2C, test_0x2c_sra_h, set_h, get_h);
    test_sra!(0x2D, test_0x2d_sra_l, set_l, get_l);
    test_sra!(0x2E, test_0x2e_sra__hl_, set_hl, get_hl, memory);
    test_sra!(0x2F, test_0x2f_sra_a, set_a, get_a);

    test_swap!(0x30, test_0x30_swap_b, set_b, get_b);
    test_swap!(0x31, test_0x31_swap_c, set_c, get_c);
    test_swap!(0x32, test_0x32_swap_d, set_d, get_d);
    test_swap!(0x33, test_0x33_swap_e, set_e, get_e);
    test_swap!(0x34, test_0x34_swap_h, set_h, get_h);
    test_swap!(0x35, test_0x35_swap_l, set_l, get_l);
    test_swap!(0x36, test_0x36_swap__hl_, set_hl, get_hl, memory);
    test_swap!(0x37, test_0x37_swap_a, set_a, get_a);
}