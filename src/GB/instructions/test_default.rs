use std::cell::RefCell;
use std::rc::Rc;
use crate::GB::CPU::{CPU};
use crate::GB::memory::{self, RAM, UseMemory, USER_PROGRAM_ADDRESS, WRAM_ADDRESS};

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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg_from(test_value_1);
                cpu.registers.$set_reg_addr(test_address_1);
                cpu.write_memory(test_address_1, 0x00);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_1);
                assert_eq!(cpu.registers.$get_reg_addr(), test_address_1);
                assert_eq!(cpu.read_memory(test_address_1), test_value_1);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode, test_address_1_low, test_address_1_high];
                cpu.load(&program_1);
                let register_copy = cpu.registers;
                cpu.registers.$set_from(test_value_1);
                cpu.write_memory(test_address_1, 0x00);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu.registers.$get_from(), test_value_1);
                assert_eq!(cpu.read_memory(test_address_1), test_value_1);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode, test_address_1_low, test_address_1_high];
                cpu.load(&program_1);
                let register_copy = cpu.registers;
                cpu.registers.$set_to(0);
                cpu.write_memory(test_address_1, test_value_1);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu.registers.$get_to(), test_value_1);
                assert_eq!(cpu.read_memory(test_address_1), test_value_1);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let register_copy = cpu.registers;
                cpu.registers.$set_reg_to(0x00);
                cpu.registers.$set_reg_addr(test_address);
                cpu.write_memory(cpu.registers.$get_reg_addr(), test_value);
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_to(), test_value);
                assert_eq!(cpu.registers.$get_reg_addr(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program: Vec<u8> = vec![$opcode, 0x5A];
                cpu.load(&program);
                let registers_copy = cpu.registers;
                cpu.write_memory(test_addr, if $byte_is_src {test_value} else {0});
                cpu.registers.$set_reg(if !$byte_is_src {test_value} else {0});
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.$get_reg(), test_value);
                assert_eq!(cpu.read_memory(test_addr), test_value);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                let registers_copy = cpu.registers;
                cpu.write_memory(test_addr, if $rtl {test_value} else {0});
                cpu.registers.$set_reg_2(test_addr_low);
                cpu.registers.$set_reg(if !$rtl {test_value} else {0});
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg(), test_value);
                assert_eq!(cpu.registers.$get_reg_2(), test_addr_low);
                assert_eq!(cpu.read_memory(test_addr), test_value);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                cpu.registers.$set_reg(test_value_1 - 1);
                cpu.execute_next();
                assert_eq!(cpu.registers.$get_reg(), test_value_1);
                test_flags!(cpu, false, false, false, registers_copy.get_carry_flag());

                // Flags Z/H
                test_value_1 = 0xFF;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                cpu.registers.$set_reg(test_value_1);
                cpu.execute_next();
                assert_eq!(cpu.registers.$get_reg(), 0);
                test_flags!(cpu, true, false, true, registers_copy.get_carry_flag());

                // Flags H
                test_value_1 = 0x0F;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                cpu.registers.$set_reg(test_value_1);
                cpu.execute_next();
                assert_eq!(cpu.registers.$get_reg(), 0x10);
                test_flags!(cpu, false, false, true, registers_copy.get_carry_flag());
            }
        };
    }

macro_rules! test_dec_r8 {
        ($opcode:expr, $func:ident, $set_reg:ident, $get_reg:ident) => {
            #[test]
            fn $func() {
                //No Flags
                let mut test_value_1: u8 = 0xF4;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                cpu.registers.$set_reg(test_value_1 + 1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg(), test_value_1);
                test_flags!(cpu, false, true, false, registers_copy.get_carry_flag());

                // Flags H
                test_value_1 = 0x00;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                cpu.registers.$set_reg(test_value_1);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg(), 0xFF);
                test_flags!(cpu, false, true, true, registers_copy.get_carry_flag());

                // Flags Z
                test_value_1 = 0x00;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                cpu.registers.$set_reg(test_value_1 + 1);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg(), test_value_1);
                test_flags!(cpu, true, true, false, registers_copy.get_carry_flag());

                // Flags H
                test_value_1 = 0xF0;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                cpu.registers.$set_reg(test_value_1);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg(), test_value_1 - 1);
                test_flags!(cpu, false, true, true, registers_copy.get_carry_flag());
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // No Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0x10;
                expected_value = 0x00;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // Z/C Flags
                test_flags!(cpu, true, false, false, true);

                test_value_1 = 0x0F;
                test_value_2 = 0x01;
                expected_value = 0x10;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // H Flag
                test_flags!(cpu, false, false, true, false);

                test_value_1 = 0xFF;
                test_value_2 = 0x01;
                expected_value = 0x00;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // Z/H/C Flag
                test_flags!(cpu, true, false, true, true);
            }
        };
    }

macro_rules! test_sub_r8_r8 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident, $set_reg_from:ident, $get_reg_from:ident) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x11;
                let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // N Flags
                test_flags!(cpu, false, true, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0xF0;
                expected_value = 0x00;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // N/Z Flags
                test_flags!(cpu, true, true, false, false);

                test_value_1 = 0x10;
                test_value_2 = 0x01;
                expected_value = 0x0F;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // N/H Flag
                test_flags!(cpu, false, true, true, false);

                test_value_1 = 0x10;
                test_value_2 = 0x20;
                expected_value = test_value_1.wrapping_sub(test_value_2);
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // N/C Flag
                test_flags!(cpu, false, true, false, true);

                test_value_1 = 0x00;
                test_value_2 = 0x01;
                expected_value = 0xFF;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // H/C Flag
                test_flags!(cpu, false, true, true, true);
            }
        };
    }

macro_rules! test_add_r16_r16 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident, $set_reg_from:ident, $get_reg_from:ident) => {
            #[test]
            fn $func() {
                //No Flags
                let mut test_value_1: u16 = 0xBD89;
                let mut test_value_2: u16 = 0x1029;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program: Vec<u8> = vec![$opcode];
                cpu.load(&program);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                let mut registers_copy = cpu.registers;
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                assert_eq!(cpu.registers.$get_reg_to(), test_value_1 + test_value_2);
                test_flags!(cpu, registers_copy.get_zero_flag(), false, false, false);

                test_value_1 = 0x7000;
                test_value_2 = 0x9000;
                cpu.load(&program);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                registers_copy = cpu.registers;
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                assert_eq!(cpu.registers.$get_reg_to(), 0);
                test_flags!(cpu, registers_copy.get_zero_flag(), false, false, true);

                // H flags on ADD HL, rr should be on only carrying from bit 11 (check is made on H of HL)
                test_value_1 = 0x1070;
                test_value_2 = 0x1090;
                cpu.load(&program);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                registers_copy = cpu.registers;
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                assert_eq!(cpu.registers.$get_reg_to(), test_value_1 + test_value_2);
                test_flags!(cpu, registers_copy.get_zero_flag(), false, false, false);

                test_value_1 = 0x1700;
                test_value_2 = 0x1900;
                cpu.load(&program);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                registers_copy = cpu.registers;
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                assert_eq!(cpu.registers.$get_reg_to(), test_value_1 + test_value_2);
                test_flags!(cpu, registers_copy.get_zero_flag(), false, true, false);

                test_value_1 = 0x9700;
                test_value_2 = 0x7900;
                cpu.load(&program);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                registers_copy = cpu.registers;
                let cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                assert_eq!(cpu.registers.$get_reg_to(), test_value_1.wrapping_add(test_value_2));
                test_flags!(cpu, registers_copy.get_zero_flag(), false, true, true);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cpu.registers.set_carry_flag($carry);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // No Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0x10 - $carry as u8;
                expected_value = 0x00;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // Z/C Flags
                test_flags!(cpu, true, false, $carry, true);

                test_value_1 = 0x0F;
                test_value_2 = 0x01;
                expected_value = 0x10 + $carry as u8;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // H Flag
                test_flags!(cpu, false, false, true, false);

                test_value_1 = 0xFF - $carry as u8;
                test_value_2 = 0x01;
                expected_value = 0x00;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // Z/H/C Flag
                test_flags!(cpu, true, false, true, true);
            }
        };
    }

macro_rules! test_sbc_r8_r8 {
        ($opcode:expr, $func:ident, $set_reg_to:ident, $get_reg_to:ident, $set_reg_from:ident, $get_reg_from:ident, $carry:expr) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x12 - $carry as u8;
                let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2 + $carry as u8);
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cpu.registers.set_carry_flag($carry);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // No Flags
                test_flags!(cpu, false, true, false, false);

                test_value_1 = 0x0F;
                test_value_2 = 0x0F - $carry as u8;
                expected_value = 0x00;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // Z Flags
                test_flags!(cpu, true, true, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0x01 - $carry as u8;
                expected_value = 0xEF;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // H Flag
                test_flags!(cpu, false, true, true, false);

                test_value_1 = 0x00;
                test_value_2 = 0x01 - $carry as u8;
                expected_value = 0xFF;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.$set_reg_from(test_value_2);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                assert_eq!(cpu.registers.$get_reg_from(), test_value_2);
                // H/C Flag
                test_flags!(cpu, false, true, true, true);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                // No Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0x10;
                expected_value = 0x00;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                // Z/C Flags
                test_flags!(cpu, true, false, false, true);

                test_value_1 = 0x0F;
                test_value_2 = 0x01;
                expected_value = 0x10;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                // H Flag
                test_flags!(cpu, false, false, true, false);

                test_value_1 = 0xFF;
                test_value_2 = 0x01;
                expected_value = 0x00;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                // Z/H/C Flag
                test_flags!(cpu, true, false, true, true);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.set_carry_flag($carry);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                // No Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0x10 - $carry as u8;
                expected_value = 0x00;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                // Z/C Flags
                test_flags!(cpu, true, false, $carry, true);

                test_value_1 = 0x0F;
                test_value_2 = 0x01;
                expected_value = 0x10 + $carry as u8;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                // H Flag
                test_flags!(cpu, false, false, true, false);

                test_value_1 = 0xFF - $carry as u8;
                test_value_2 = 0x01;
                expected_value = 0x00;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.$set_reg_to(test_value_1);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.$get_reg_to(), expected_value);
                // Z/H/C Flag
                test_flags!(cpu, true, false, true, true);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_carry_flag($carry);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // N Flags
                test_flags!(cpu, false, true, false, false);

                test_value_1 = 0x10;
                test_value_2 = 0x0E;
                expected_value = test_value_1.wrapping_sub(test_value_2 + $carry as u8);
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // H Flags
                test_flags!(cpu, false, true, true, false);

                test_value_1 = 0x10;
                test_value_2 = 0x0F;
                expected_value = test_value_1.wrapping_sub(test_value_2).wrapping_sub($carry as u8);
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // Z/H Flag
                test_flags!(cpu, $carry, true, true, false);

                test_value_1 = 0x00;
                test_value_2 = 0;
                expected_value = test_value_1.wrapping_sub($carry as u8);
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_carry_flag($carry);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // H/C Flag
                test_flags!(cpu, !$carry, true, $carry, $carry);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // H Flag
                test_flags!(cpu, false, false, true, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0100_0110;
                expected_value = 0x0;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu, true, false, true, false);
            }
        };
        ($opcode:expr, $func:ident, hl) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let test_address: u16 = WRAM_ADDRESS as u16 + 0x22;
                let mut expected_value: u8 = test_value_1 & test_value_2;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // H Flag
                test_flags!(cpu, false, false, true, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0100_0110;
                expected_value = 0x0;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu, true, false, true, false);
            }
        };
        ($opcode:expr, $func:ident, a) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut expected_value: u8 = test_value_1 & test_value_1;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // H Flag
                test_flags!(cpu, false, false, true, false);

                test_value_1 = 0b1010_1001;
                expected_value = test_value_1;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu, false, false, true, false);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // H Flag
                test_flags!(cpu, false, false, true, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0100_0110;
                program_1[1] = test_value_2;
                expected_value = 0x0;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu, true, false, true, false);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // H Flag
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                expected_value = 0xFF;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b1010_1001;
                expected_value = 0x0;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu, true, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, hl) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let test_address: u16 = WRAM_ADDRESS as u16 + 0x22;
                let mut expected_value: u8 = test_value_1 ^ test_value_2;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // H Flag
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                expected_value = 0xFF;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b1010_1001;
                expected_value = 0x0;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu, true, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, a) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut expected_value: u8 = 0x0;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // H Flag
                test_flags!(cpu, true, false, false, false);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // H Flag
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                program_1[1] = test_value_2;
                expected_value = 0xFF;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b1010_1001;
                expected_value = 0x0;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu, true, false, false, false);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // H Flag
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                expected_value = 0xFF;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b1010_1001;
                expected_value = test_value_1;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b0;
                test_value_2 = 0b0;
                expected_value = test_value_1;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // Z/H Flags
                test_flags!(cpu, true, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, hl) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut test_value_2: u8 = 0b0100_0111;
                let test_address: u16 = WRAM_ADDRESS as u16 + 0x22;
                let mut expected_value: u8 = test_value_1 | test_value_2;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // H Flag
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                expected_value = 0xFF;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0;
                expected_value = test_value_1;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b0;
                test_value_2 = 0b0;
                expected_value = test_value_1;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // Z/H Flags
                test_flags!(cpu, true, false, false, false);
            }
        };
        ($opcode:expr, $func:ident, a) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut expected_value: u8 = test_value_1;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // No Flag
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b0;
                expected_value = test_value_1;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // Z Flag
                test_flags!(cpu, true, false, false, false);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // No Flag
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b0101_0110;
                expected_value = 0xFF;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b1010_1001;
                test_value_2 = 0b1010_1001;
                expected_value = test_value_1;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu, false, false, false, false);

                test_value_1 = 0b0;
                test_value_2 = 0b0;
                expected_value = test_value_1;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), expected_value);
                // Z/H Flags
                test_flags!(cpu, true, false, false, false);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // No Flags
                test_flags!(cpu, false, true, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0xF0;
                expected_value = 0x00;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // Z/N Flags
                test_flags!(cpu, true, true, false, false);

                test_value_1 = 0x10;
                test_value_2 = 0x01;
                expected_value = 0x0F;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // N/H Flag
                test_flags!(cpu, false, true, true, false);

                test_value_1 = 0x10;
                test_value_2 = 0x20;
                expected_value = test_value_1.wrapping_sub(test_value_2);
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // N/C Flag
                test_flags!(cpu, false, true, false, true);

                test_value_1 = 0x00;
                test_value_2 = 0x01;
                expected_value = 0xFF;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.$set_reg(test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                assert_eq!(cpu.registers.$get_reg(), test_value_2);
                // N/H/C Flag
                test_flags!(cpu, false, true, true, true);
            }
        };
        ($opcode:expr, $func:ident, hl) => {
            #[test]
            fn $func() {
                let test_address: u16 = WRAM_ADDRESS as u16 + 0x55;
                let mut test_value_1: u8 = 0xC4;
                let mut test_value_2: u8 = 0x11;
                let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // No Flags
                test_flags!(cpu, false, true, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0xF0;
                expected_value = 0x00;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // Z/N Flags
                test_flags!(cpu, true, true, false, false);

                test_value_1 = 0x10;
                test_value_2 = 0x01;
                expected_value = 0x0F;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // N/H Flag
                test_flags!(cpu, false, true, true, false);

                test_value_1 = 0x10;
                test_value_2 = 0x20;
                expected_value = test_value_1.wrapping_sub(test_value_2);
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // N/C Flag
                test_flags!(cpu, false, true, false, true);

                test_value_1 = 0x00;
                test_value_2 = 0x01;
                expected_value = 0xFF;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cpu.registers.set_hl(test_address);
                cpu.write_memory(test_address, test_value_2);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                assert_eq!(cpu.registers.get_hl(), test_address);
                assert_eq!(cpu.read_memory(test_address), test_value_2);
                // N/H/C Flag
                test_flags!(cpu, false, true, true, true);
            }
        };
        ($opcode:expr, $func:ident, a) => {
            #[test]
            fn $func() {
                let mut test_value_1: u8 = 0b0110_1001;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                // Z Flag
                test_flags!(cpu, true, true, false, false);

                test_value_1 = 0b0;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 1);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                // Z Flag
                test_flags!(cpu, true, true, false, false);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let mut program_1: Vec<u8> = vec![$opcode, test_value_2];
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                // No Flags
                test_flags!(cpu, false, true, false, false);

                test_value_1 = 0xF0;
                test_value_2 = 0xF0;
                expected_value = 0x00;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                // Z/N Flags
                test_flags!(cpu, true, true, false, false);

                test_value_1 = 0x10;
                test_value_2 = 0x01;
                expected_value = 0x0F;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                // N/H Flag
                test_flags!(cpu, false, true, true, false);

                test_value_1 = 0x10;
                test_value_2 = 0x20;
                expected_value = test_value_1.wrapping_sub(test_value_2);
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                // N/C Flag
                test_flags!(cpu, false, true, false, true);

                test_value_1 = 0x00;
                test_value_2 = 0x01;
                expected_value = 0xFF;
                program_1[1] = test_value_2;
                cpu = CPU::new(Rc::clone(&memory_ref));
                cpu.load(&program_1);
                cpu.registers.set_a(test_value_1);
                cycles = cpu.execute_next();
                assert_eq!(cycles, 2);
                assert_eq!(cpu.registers.get_a(), test_value_1);
                // N/H/C Flag
                test_flags!(cpu, false, true, true, true);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode, test_call_low, test_call_high];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                let return_address = cpu.registers.get_pc() + 3;
                let return_address_low = (return_address & 0xFF) as u8;
                let return_address_high = (return_address >> 8) as u8;
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 6);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp() - 2);
                assert_eq!(cpu.registers.get_pc(), test_call_address);
                assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 1), return_address_low);
                assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 2), return_address_high);
            }
        };
        ($opcode:expr, $func:ident, $inverse:expr, $set_flag:ident, $get_flag:ident) => {
            #[test]
            fn $func() {
                let test_call_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x210;
                let test_call_low: u8 = (test_call_address & 0xFF) as u8;
                let test_call_high: u8 = (test_call_address >> 8) as u8;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode, test_call_low, test_call_high];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                if $inverse {cpu.registers.$set_flag(true)} else {cpu.registers.$set_flag(false)};
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp());
                assert_eq!(cpu.registers.get_pc(), registers_copy.get_pc() + 3);

                cpu = CPU::new(Rc::clone(&memory_ref));
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
                assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 1), return_address_low);
                assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 2), return_address_high);
            }
        };
    }

macro_rules! test_ret {
        ($opcode:expr, $func:ident) => {
            #[test]
            fn $func() {
                let test_return_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x210;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                cpu.push((test_return_address >> 8) as u8);
                cpu.push((test_return_address & 0xFF) as u8);
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp());
                assert_eq!(cpu.registers.get_pc(), test_return_address);
            }
        };
        ($opcode:expr, $func:ident, $inverse:expr, $set_flag:ident, $get_flag:ident) => {
            #[test]
            fn $func() {
                let test_return_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x210;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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

                cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode, test_call_address_low, test_call_address_high];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                let test_return_address = registers_copy.get_pc() + program_1.len() as u16;
                let test_return_address_high = (test_return_address >> 8) as u8;
                let test_return_address_low = (test_return_address & 0xFF) as u8;
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 6);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp() - 2);
                assert_eq!(cpu.registers.get_pc(), test_call_address);
                assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 1), test_return_address_low);
                assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 2), test_return_address_high);
            }
        };
        ($opcode:expr, $func:ident, $inverse:expr, $set_flag:ident, $get_flag:ident) => {
            #[test]
            fn $func() {
                let test_call_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x0C16;
                let test_call_address_high: u8 = (test_call_address >> 8) as u8;
                let test_call_address_low: u8 = (test_call_address & 0xFF) as u8;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode, test_call_address_low, test_call_address_high];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                if $inverse {cpu.registers.$set_flag(true)} else {cpu.registers.$set_flag(false)};
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp());
                assert_eq!(cpu.registers.get_pc(), registers_copy.get_pc() + program_1.len() as u16);

                cpu = CPU::new(Rc::clone(&memory_ref));
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
                assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 1), test_return_address_low);
                assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 2), test_return_address_high);
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode, test_address_low_byte, test_address_high_byte];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu.registers.get_pc(), test_jump_address);
            }
        };
        ($opcode:expr, $func:ident, $inverse:expr, $set_flag:ident, $get_flag:ident) => {
            #[test]
            fn $func() {
                let test_jump_address: u16 = USER_PROGRAM_ADDRESS as u16 + 0x210;
                let test_address_high_byte: u8 = (test_jump_address >> 8) as u8;
                let test_address_low_byte: u8 = (test_jump_address & 0xFF) as u8;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode, test_address_low_byte, test_address_high_byte];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                if $inverse {cpu.registers.$set_flag(true)} else {cpu.registers.$set_flag(false)};
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.get_pc(), registers_copy.get_pc() + 3);

                cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                cpu.registers.$set_reg(test_value);
                let registers_copy = cpu.registers;
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 3);
                assert_eq!(cpu.registers.$get_reg(), if stringify!($get_reg) == "get_af" {test_value & 0xFFF0} else {test_value});
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp() - 2);
                assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 2), test_high_byte);
                if (stringify!($get_reg) == "get_af") {
                    assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 1), test_low_byte & 0xF0);
                } else {
                    assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 1), test_low_byte);
                }
            }
        };
    }

macro_rules! test_rst {
        ($opcode:expr, $func:ident) => {
            #[test]
            fn $func() {
                let int_addr: u16 = (($opcode & 0b00_111_000) >> 3) as u16 * 8;
                let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
                let mut cpu = CPU::new(Rc::clone(&memory_ref));
                let mut program_1: Vec<u8> = vec![$opcode];
                cpu.load(&program_1);
                let registers_copy = cpu.registers;
                let expected_return_addr = cpu.registers.get_pc() + program_1.len() as u16;
                let mut cycles = cpu.execute_next();
                assert_eq!(cycles, 4);
                assert_eq!(cpu.registers.get_pc(), int_addr);
                assert_eq!(cpu.registers.get_sp(), registers_copy.get_sp() - 2);
                assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 1), (expected_return_addr & 0xFF) as u8);
                assert_eq!(cpu.read_memory(cpu.registers.get_sp() + 2), (expected_return_addr >> 8) as u8);
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


#[test]
fn test_0x00_nop() {
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x07, 0x07];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.execute_next();
    // Check load data and FLAGs should be untouched
    assert_eq!(cpu.registers.get_a(), 0b0001_0001);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);
    cpu.execute_next();
    assert_eq!(cpu.registers.get_a(), 0b0010_0010);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);
}

#[test]
fn test_0x08_ld__a16__sp() {
    //No Flags
    let test_value_1: u16 = 0xBD89;
    let test_address_1: u16 = WRAM_ADDRESS as u16 + 0x89;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x08, 0x89, (test_address_1 >> 8) as u8];
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let cycles = cpu.execute_next();
    // Check address and data are correctly used
    assert_eq!(cycles, 5);
    assert_eq!(cpu.registers.get_sp(), test_value_1);
    assert_eq!(cpu.read_memory(test_address_1), 0x89);
    assert_eq!(cpu.read_memory(test_address_1 + 1), (test_value_1 >> 8) as u8);
}

test_add_r16_r16!(0x09, test_0x09_add_hl_bc, set_hl, get_hl, set_bc, get_bc);

#[test]
fn test_0x0a_ld_a__bc_() {
    let mut test_value_1: u8 = 0xBD;
    let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program: Vec<u8> = vec![0x0A];
    cpu.load(&program);
    cpu.registers.set_bc(test_address_1);
    cpu.write_memory(test_address_1, test_value_1);
    cpu.registers.set_a(0x11); // Sure different from expected value
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.read_memory(test_address_1), test_value_1);
    assert_eq!(cpu.registers.get_bc(), test_address_1);
    assert_eq!(cpu.registers.get_a(), test_value_1);
}

test_dec_r16!(0x0B, test_0x0b_dec_bc, set_bc, get_bc, get_b, get_c);
test_inc_r8!(0x0C, test_0x0c_inc_c, set_c, get_c);
test_dec_r8!(0x0D, test_0x0d_dec_c, set_c, get_c);
test_ld_r8_imm8!(0x0E, test_0x0e_ld_c_imm8, set_c, get_c);

#[test]
fn test_0x0f_rrca() {
    //No Flags
    let test_value_1: u8 = 0b0001_0001;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x0F, 0x0F];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.execute_next();
    // Check load data and FLAGs should be untouched
    assert_eq!(cpu.registers.get_a(), 0b1000_1000);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);
    cpu.execute_next();
    assert_eq!(cpu.registers.get_a(), 0b0100_0100);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x17, 0x17];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(false);
    cpu.execute_next();
    // The re-entrance Bit is given by the previous content of C Flag
    assert_eq!(cpu.registers.get_a(), 0b0001_0000);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);
    cpu.execute_next();
    assert_eq!(cpu.registers.get_a(), 0b0010_0001);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);
}

#[test]
fn test_0x18_jr_e8() {
    let mut test_value: i8 = -50;
    let mut start_address: i16 = 0x0350;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let mut program: Vec<u8> = vec![0x18, test_value as u8];
    cpu.load(&program);
    cpu.write_memory(0x0350, program[0]);
    cpu.write_memory(0x0351, program[1]);
    cpu.registers.set_pc(0x0350);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);
}

test_add_r16_r16!(0x19, test_0x19_add_hl_de, set_hl, get_hl, set_de, get_de);

#[test]
fn test_0x1a_ld_a__de_() {
    let mut test_value_1: u8 = 0xBD;
    let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program: Vec<u8> = vec![0x1A];
    cpu.load(&program);
    cpu.registers.set_de(test_address_1);
    cpu.write_memory(test_address_1, test_value_1);
    cpu.registers.set_a(0x11); // Sure different from expected value
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.read_memory(test_address_1), test_value_1);
    assert_eq!(cpu.registers.get_de(), test_address_1);
    assert_eq!(cpu.registers.get_a(), test_value_1);
}

test_dec_r16!(0x1B, test_0x1b_dec_de, set_de, get_de, get_d, get_e);
test_inc_r8!(0x1C, test_0x1c_inc_e, set_e, get_e);
test_dec_r8!(0x1D, test_0x1d_dec_e, set_e, get_e);
test_ld_r8_imm8!(0x1E, test_0x1e_ld_e_imm8, set_e, get_e);

#[test]
fn test_0x1f_rra() {
    //No Flags
    let test_value_1: u8 = 0b0001_0001;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x1F, 0x1F];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(false);
    cpu.execute_next();
    // The re-entrance Bit is given by the previous content of C Flag
    assert_eq!(cpu.registers.get_a(), 0b0000_1000);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);
    cpu.execute_next();
    assert_eq!(cpu.registers.get_a(), 0b1000_0100);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);
}

#[test]
fn test_0x20_jr_nz_e8() {
    let mut test_value: i8 = -50;
    let mut start_address: i16 = 0x0350;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let mut program: Vec<u8> = vec![0x20, test_value as u8];
    cpu.load(&program);
    cpu.write_memory(0x0350, program[0]);
    cpu.write_memory(0x0351, program[1]);
    cpu.registers.set_pc(0x0350);
    cpu.registers.set_zero_flag(false);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

    cpu = CPU::new(Rc::clone(&memory_ref));
    assert_eq!(cycles, 3);
    cpu.load(&program);
    cpu.write_memory(0x0350, program[0]);
    cpu.write_memory(0x0351, program[1]);
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
    let test_address: u16 = memory::WRAM_ADDRESS as u16 + 0x0500;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program: Vec<u8> = vec![0x22];
    cpu.load(&program);
    cpu.registers.set_a(test_value);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, 0x00);
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), test_value);
    assert_eq!(cpu.registers.get_hl(), test_address + 1);
    assert_eq!(cpu.read_memory(test_address), test_value);
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let mut program: Vec<u8> = vec![0x28, test_value as u8];
    cpu.load(&program);
    cpu.write_memory(0x0350, program[0]);
    cpu.write_memory(0x0351, program[1]);
    cpu.registers.set_pc(0x0350);
    cpu.registers.set_zero_flag(true);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

    cpu = CPU::new(Rc::clone(&memory_ref));
    assert_eq!(cycles, 3);
    cpu.load(&program);
    cpu.write_memory(0x0350, program[0]);
    cpu.write_memory(0x0351, program[1]);
    cpu.registers.set_pc(0x0350);
    cpu.registers.set_zero_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_pc(), 0x352);
}

#[test]
fn test_0x29_add_hl_hl() {
    let mut test_value: u16 = 0x1029;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program: Vec<u8> = vec![0x29];
    cpu.load(&program);
    cpu.registers.set_hl(test_value);
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_hl(), test_value + test_value);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value = 0x8000;
    cpu.load(&program);
    cpu.registers.set_hl(test_value);
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_hl(), 0);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    // H flags on ADD HL, rr should be on only carrying from bit 11 (check is made on H of HL)
    test_value = 0x1080;
    cpu.load(&program);
    cpu.registers.set_hl(test_value);
    cpu.registers.set_de(test_value);
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_hl(), test_value + test_value);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value = 0x1800;
    cpu.load(&program);
    cpu.registers.set_hl(test_value);
    cpu.registers.set_de(test_value);
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_hl(), test_value + test_value);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value = 0x8800;
    cpu.load(&program);
    cpu.registers.set_hl(test_value);
    cpu.registers.set_de(test_value);
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_hl(), test_value.wrapping_add(test_value));
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
}

#[test]
fn test_0x2a_ld_a__hli_() {
    let mut test_value_1: u8 = 0xBD;
    let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program: Vec<u8> = vec![0x2A];
    cpu.load(&program);
    cpu.registers.set_hl(test_address_1);
    cpu.write_memory(test_address_1, test_value_1);
    cpu.registers.set_a(0x11); // Sure different from expected value
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.read_memory(test_address_1), test_value_1);
    assert_eq!(cpu.registers.get_hl(), test_address_1 + 1);
    assert_eq!(cpu.registers.get_a(), test_value_1);
}

test_dec_r16!(0x2B, test_0x2b_dec_hl, set_hl, get_hl, get_h, get_l);
test_inc_r8!(0x2C, test_0x2c_inc_l, set_l, get_l);
test_dec_r8!(0x2D, test_0x2d_dec_l, set_l, get_l);
test_ld_r8_imm8!(0x2E, test_0x2e_ld_l_imm8, set_l, get_l);

#[test]
fn test_0x2f_cpl() {
    //No Flags
    let test_value_1: u8 = 0xD4;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x2F];
    cpu.load(&program_1);
    let register_copy = cpu.registers;
    cpu.registers.set_a(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), !test_value_1);
    // Z/C Flags untouched - N/H Flags on
    assert_eq!(cpu.registers.get_zero_flag(), register_copy.get_zero_flag());
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), register_copy.get_carry_flag());
}

#[test]
fn test_0x30_jr_nc_e8() {
    let mut test_value: i8 = -50;
    let mut start_address: i16 = 0x0350;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let mut program: Vec<u8> = vec![0x30, test_value as u8];
    cpu.load(&program);
    cpu.write_memory(0x0350, program[0]);
    cpu.write_memory(0x0351, program[1]);
    cpu.registers.set_pc(0x0350);
    cpu.registers.set_carry_flag(false);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

    cpu = CPU::new(Rc::clone(&memory_ref));
    assert_eq!(cycles, 3);
    cpu.load(&program);
    cpu.write_memory(0x0350, program[0]);
    cpu.write_memory(0x0351, program[1]);
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
    let test_address: u16 = memory::WRAM_ADDRESS as u16 + 0x0500;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program: Vec<u8> = vec![0x32];
    cpu.load(&program);
    cpu.registers.set_a(test_value);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, 0x00);
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), test_value);
    assert_eq!(cpu.registers.get_hl(), test_address - 1);
    assert_eq!(cpu.read_memory(test_address), test_value);
}

test_inc_r16!(0x33, test_0x33_inc_sp, set_sp, get_sp);

#[test]
fn test_0x34_inc__hl_() {
    // No Flags
    let test_value_1: u8 = 0b1111_0100;
    let test_address = WRAM_ADDRESS as u16 + 0x50;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x34];
    cpu.load(&program_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_1);
    let mut cycle = cpu.execute_next();
    assert_eq!(cycle, 3);
    assert_eq!(cpu.read_memory(test_address), test_value_1 + 1);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);

    // Flags Z/H
    let test_value_2: u8 = 0xFF;
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cycle = cpu.execute_next();
    assert_eq!(cpu.read_memory(test_address), 0);
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);

    // Flags H
    let test_value_3: u8 = 0x0F;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_3);
    cycle = cpu.execute_next();
    assert_eq!(cpu.read_memory(test_address), 0x10);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
}

#[test]
fn test_0x35_dec__hl_() {
    // No Flags
    let test_value_1: u8 = 0b1111_0100;
    let test_address = WRAM_ADDRESS as u16 + 0x50;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x35];
    cpu.load(&program_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_1);
    let mut cycle = cpu.execute_next();
    assert_eq!(cycle, 3);
    assert_eq!(cpu.read_memory(test_address), test_value_1 - 1);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);

    // Flags Z
    let test_value_2: u8 = 0x01;
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cycle = cpu.execute_next();
    assert_eq!(cpu.read_memory(test_address), 0);
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);

    // Flags H
    let test_value_3: u8 = 0xF0;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_3);
    cycle = cpu.execute_next();
    assert_eq!(cpu.read_memory(test_address), test_value_3 - 1);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);

    // Test Underflow
    let test_value_4: u8 = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_4);
    cycle = cpu.execute_next();
    assert_eq!(cpu.read_memory(test_address), 0xFF);
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
}

#[test]
fn test_0x36_ld__hl__imm8() {
    //No Flags
    let test_value_1: u8 = 0xCD;
    let test_address: u16 = WRAM_ADDRESS as u16 + 0x88;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x36, test_value_1];
    cpu.load(&program_1);
    let register_copy = cpu.registers;
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, 0x00);
    let cycles = cpu.execute_next();
    // Check load data and FLAGs should be untouched
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_hl(), test_address);
    assert_eq!(cpu.read_memory(test_address), test_value_1);
    assert_eq!(cpu.registers.get_zero_flag(), register_copy.get_zero_flag());
    assert_eq!(cpu.registers.get_negative_flag(), register_copy.get_negative_flag());
    assert_eq!(cpu.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
    assert_eq!(cpu.registers.get_carry_flag(), register_copy.get_carry_flag());
}

#[test]
// SCF = Set Carry Flag
fn test_0x37_scf() {
    //No Flags
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x37];
    cpu.load(&program_1);
    cpu.registers.set_carry_flag(false);
    let cycles = cpu.execute_next();
    // Check load data and FLAGs should be untouched
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);
}

#[test]
fn test_0x38_jr_c_e8() {
    let mut test_value: i8 = -50;
    let mut start_address: i16 = 0x0350;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let mut program: Vec<u8> = vec![0x38, test_value as u8];
    cpu.load(&program);
    cpu.write_memory(0x0350, program[0]);
    cpu.write_memory(0x0351, program[1]);
    cpu.registers.set_pc(0x0350);
    cpu.registers.set_carry_flag(true);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_pc(), ((start_address + test_value as i16 + program.len() as i16)) as u16);

    cpu = CPU::new(Rc::clone(&memory_ref));
    assert_eq!(cycles, 3);
    cpu.load(&program);
    cpu.write_memory(0x0350, program[0]);
    cpu.write_memory(0x0351, program[1]);
    cpu.registers.set_pc(0x0350);
    cpu.registers.set_carry_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_pc(), 0x352);
}

test_add_r16_r16!(0x39, test_0x39_add_hl_sp, set_hl, get_hl, set_sp, get_sp);

#[test]
fn test_0x3a_ld_a__hld_() {
    let mut test_value_1: u8 = 0xBD;
    let mut test_address_1: u16 = WRAM_ADDRESS as u16 + 0x0128;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program: Vec<u8> = vec![0x3A];
    cpu.load(&program);
    cpu.registers.set_hl(test_address_1);
    cpu.write_memory(test_address_1, test_value_1);
    cpu.registers.set_a(0x11); // Sure different from expected value
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.read_memory(test_address_1), test_value_1);
    assert_eq!(cpu.registers.get_hl(), test_address_1 - 1);
    assert_eq!(cpu.registers.get_a(), test_value_1);
}

test_dec_r16!(0x3B, test_0x3b_dec_sp, set_sp, get_sp);
test_inc_r8!(0x3C, test_0x3c_inc_a, set_a, get_a);
test_dec_r8!(0x3D, test_0x3d_dec_a, set_a, get_a);
test_ld_r8_imm8!(0x3E, test_0x3e_ld_a_imm8, set_a, get_a);

#[test]
fn test_0x3f_ccf() {
    // CCF = Complement Carry Flag
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x3F, 0x3F];
    cpu.load(&program_1);
    let register_copy = cpu.registers;
    cpu.registers.set_carry_flag(false);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    // Z Flag untouched - N/H Flags off
    assert_eq!(cpu.registers.get_zero_flag(), register_copy.get_zero_flag());
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    // Z Flag untouched - N/H Flags off
    assert_eq!(cpu.registers.get_zero_flag(), register_copy.get_zero_flag());
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x66];
    cpu.load(&program_1);
    let register_copy = cpu.registers;
    cpu.registers.set_hl(test_address_1);
    cpu.write_memory(test_address_1, test_value_1);
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_h(), test_value_1);
    assert_eq!(cpu.registers.get_hl(), test_address_1 & 0xFF | (test_value_1 as u16) << 8);
    // Flags untouched
    assert_eq!(cpu.registers.get_zero_flag(), register_copy.get_zero_flag());
    assert_eq!(cpu.registers.get_negative_flag(), register_copy.get_negative_flag());
    assert_eq!(cpu.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
    assert_eq!(cpu.registers.get_carry_flag(), register_copy.get_carry_flag());
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x6E];
    cpu.load(&program_1);
    let register_copy = cpu.registers;
    cpu.registers.set_hl(test_address_1);
    cpu.write_memory(test_address_1, test_value_1);
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_l(), test_value_1);
    assert_eq!(cpu.registers.get_hl(), test_address_1 & 0xFF00 | (test_value_1 as u16));
    // Flags untouched
    assert_eq!(cpu.registers.get_zero_flag(), register_copy.get_zero_flag());
    assert_eq!(cpu.registers.get_negative_flag(), register_copy.get_negative_flag());
    assert_eq!(cpu.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
    assert_eq!(cpu.registers.get_carry_flag(), register_copy.get_carry_flag());
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x74];
    cpu.load(&program_1);
    let register_copy = cpu.registers;
    cpu.registers.set_hl(test_address_1);
    cpu.write_memory(test_address_1, 0x00);
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_h(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address_1);
    assert_eq!(cpu.read_memory(test_address_1), expected_value);
    // Flags untouched
    test_flags!(
            cpu,
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x75];
    cpu.load(&program_1);
    let register_copy = cpu.registers;
    cpu.registers.set_hl(test_address_1);
    cpu.write_memory(test_address_1, 0x00);
    let cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_l(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address_1);
    assert_eq!(cpu.read_memory(test_address_1), expected_value);
    // Flags untouched
    assert_eq!(cpu.registers.get_zero_flag(), register_copy.get_zero_flag());
    assert_eq!(cpu.registers.get_negative_flag(), register_copy.get_negative_flag());
    assert_eq!(cpu.registers.get_half_carry_flag(), register_copy.get_half_carry_flag());
    assert_eq!(cpu.registers.get_carry_flag(), register_copy.get_carry_flag());
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x86];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.read_memory(test_address), test_value_2);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xF0;
    test_value_2 = 0x10;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    assert_eq!(cpu.read_memory(test_address), test_value_2);
    // Z/C Flags
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    test_value_1 = 0x0F;
    test_value_2 = 0x01;
    expected_value = 0x10;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    assert_eq!(cpu.read_memory(test_address), test_value_2);
    // H Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xFF;
    test_value_2 = 0x01;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    assert_eq!(cpu.read_memory(test_address), test_value_2);
    // Z/H/C Flag
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
}
#[test]
fn test_0x87_add_a_a() {
    let mut test_value_1: u8 = 0x24;
    let mut expected_value: u8 = test_value_1.wrapping_add(test_value_1);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x87];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x80;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // Z/C Flags
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    test_value_1 = 0x08;
    expected_value = 0x10;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // H Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xFF;
    expected_value = 0xFE;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // Z/H/C Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x8E];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cpu.registers.set_carry_flag(false);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xF0;
    test_value_2 = 0x10;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cpu.registers.set_carry_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    // Z/C Flags
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    test_value_1 = 0x0F;
    test_value_2 = 0x01;
    expected_value = 0x10;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cpu.registers.set_carry_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    // H Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xFF;
    test_value_2 = 0x01;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cpu.registers.set_carry_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    // Z/H/C Flag
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
}

#[test]
fn test_0x8e_adc_a__hl___c_on() {
    let mut test_value_1: u8 = 0xC4;
    let mut test_value_2: u8 = 0x16;
    let test_address: u16 = WRAM_ADDRESS as u16 + 0xAA;
    let mut expected_value: u8 = test_value_1.wrapping_add(test_value_2 + 1);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x8E];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cpu.registers.set_carry_flag(true);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xF0;
    test_value_2 = 0x0F;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cpu.registers.set_carry_flag(true);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    // Z/C Flags
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    test_value_1 = 0x0D;
    test_value_2 = 0x02;
    expected_value = 0x10;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cpu.registers.set_carry_flag(true);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    // H Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xFE;
    test_value_2 = 0x01;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    cpu.registers.set_carry_flag(true);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    // Z/H/C Flag
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
}

#[test]
fn test_0x8f_adc_a_a__c_off() {
    let mut test_value_1: u8 = 0x16;
    let mut expected_value: u8 = test_value_1.wrapping_add(test_value_1);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x8F];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(false);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x80;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // Z/C Flags
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    test_value_1 = 0x08;
    expected_value = 0x10;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // H Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x88;
    expected_value = 0x10;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // Z/H/C Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
}

#[test]
fn test_0x8f_adc_a_a__c_on() {
    let mut test_value_1: u8 = 0x16;
    let mut expected_value: u8 = test_value_1.wrapping_add(test_value_1 + 1);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x8F];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(true);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x80;
    expected_value = 0x01;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(true);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // C Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    test_value_1 = 0x08;
    expected_value = 0x11;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(true);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // H Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x88;
    expected_value = 0x11;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(true);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // H/C Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
}

// SUB A, r8 | SUB A, [HL]
test_sub_r8_r8!(0x90, test_0x90_sub_a_b, set_a, get_a, set_b, get_b);
test_sub_r8_r8!(0x91, test_0x91_sub_a_c, set_a, get_a, set_c, get_c);
test_sub_r8_r8!(0x92, test_0x92_sub_a_d, set_a, get_a, set_d, get_d);
test_sub_r8_r8!(0x93, test_0x93_sub_a_e, set_a, get_a, set_e, get_e);
test_sub_r8_r8!(0x94, test_0x94_sub_a_h, set_a, get_a, set_h, get_h);
test_sub_r8_r8!(0x95, test_0x95_sub_a_l, set_a, get_a, set_l, get_l);
#[test]
fn test_0x96_sub_a__hl_() {
    let mut test_value_1: u8 = 0xC4;
    let mut test_value_2: u8 = 0x11;
    let test_address: u16 = WRAM_ADDRESS as u16 + 0x55;
    let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x96];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xF0;
    test_value_2 = 0xF0;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // Z Flags
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x10;
    test_value_2 = 0x01;
    expected_value = 0x0F;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_l(test_value_2);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // H Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x10;
    test_value_2 = 0x20;
    expected_value = test_value_1.wrapping_sub(test_value_2);
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // C Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    test_value_1 = 0x00;
    test_value_2 = 0x01;
    expected_value = 0xFF;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(test_address, test_value_2);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.registers.get_hl(), test_address);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // H/C Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
}

#[test]
fn test_0x97_sub_a_a() {
    let mut test_value_1: u8 = 0xC4;
    let mut expected_value: u8 = 0;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x97];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // Z/N Flags
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xF0;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // Z/N Flags
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);
}

// SBC A, r8 | SBC A, [HL]
test_sbc_r8_r8!(0x98, test_0x98_sbc_a_b__c_off, set_a, get_a, set_b, get_b, false);
test_sbc_r8_r8!(0x98, test_0x98_sbc_a_b__c_on, set_a, get_a, set_b, get_b, true);
test_sbc_r8_r8!(0x99, test_0x99_sbc_a_c__c_off, set_a, get_a, set_c, get_c, false);
test_sbc_r8_r8!(0x99, test_0x99_sbc_a_c__c_on, set_a, get_a, set_c, get_c, true);
test_sbc_r8_r8!(0x9A, test_0x9a_sbc_a_d__c_off, set_a, get_a, set_d, get_d, false);
test_sbc_r8_r8!(0x9A, test_0x9a_sbc_a_d__c_on, set_a, get_a, set_d, get_d, true);
test_sbc_r8_r8!(0x9B, test_0x9b_sbc_a_e__c_off, set_a, get_a, set_e, get_e, false);
test_sbc_r8_r8!(0x9B, test_0x9b_sbc_a_e__c_on, set_a, get_a, set_e, get_e, true);
test_sbc_r8_r8!(0x9C, test_0x9c_sbc_a_h__c_off, set_a, get_a, set_h, get_h, false);
test_sbc_r8_r8!(0x9C, test_0x9c_sbc_a_h__c_on, set_a, get_a, set_h, get_h, true);
test_sbc_r8_r8!(0x9D, test_0x9d_sbc_a_l__c_off, set_a, get_a, set_l, get_l, false);
test_sbc_r8_r8!(0x9D, test_0x9d_sbc_a_l__c_on, set_a, get_a, set_l, get_l, true);
#[test]
fn test_0x9e_sbc_a__hl___c_off() {
    let mut test_value_1: u8 = 0xC4;
    let mut test_value_2: u8 = 0x12;
    let test_address: u16 = WRAM_ADDRESS as u16 + 0x11;
    let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x9E];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(cpu.registers.get_hl(), test_value_2);
    cpu.registers.set_carry_flag(false);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x0F;
    test_value_2 = 0x0F;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(cpu.registers.get_hl(), test_value_2);
    cpu.registers.set_carry_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // Z Flags
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xF0;
    test_value_2 = 0x01;
    expected_value = 0xEF;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(cpu.registers.get_hl(), test_value_2);
    cpu.registers.set_carry_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // H Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x00;
    test_value_2 = 0x01;
    expected_value = 0xFF;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(cpu.registers.get_hl(), test_value_2);
    cpu.registers.set_carry_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // H/C Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
}

#[test]
fn test_0x9e_sbc_a__hl___c_on() {
    let mut test_value_1: u8 = 0xC4;
    let mut test_value_2: u8 = 0x13;
    let test_address: u16 = WRAM_ADDRESS as u16 + 0x11;
    let mut expected_value: u8 = test_value_1.wrapping_sub(test_value_2 + 1);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x9E];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(cpu.registers.get_hl(), test_value_2);
    cpu.registers.set_carry_flag(true);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x10;
    test_value_2 = 0x0E;
    expected_value = test_value_1.wrapping_sub(test_value_2 + 1);
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(cpu.registers.get_hl(), test_value_2);
    cpu.registers.set_carry_flag(true);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // H Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x10;
    test_value_2 = 0x0F;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(cpu.registers.get_hl(), test_value_2);
    cpu.registers.set_carry_flag(true);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // Z/H Flag
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x00;
    test_value_2 = 0x00;
    expected_value = 0xFF;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_hl(test_address);
    cpu.write_memory(cpu.registers.get_hl(), test_value_2);
    cpu.registers.set_carry_flag(true);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    assert_eq!(cpu.read_memory(cpu.registers.get_hl()), test_value_2);
    // H/C Flag
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
}

#[test]
fn test_0x9f_sbc_a_a__c_off() {
    let mut test_value_1: u8 = 0xC4;
    let mut expected_value: u8 = 0;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x9F];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(false);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // Z/N Flags
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xF0;
    expected_value = 0x00;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(false);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // Z/N Flags
    assert_eq!(cpu.registers.get_zero_flag(), true);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);
}

#[test]
fn test_0x9f_sbc_a_a__c_on() {
    let mut test_value_1: u8 = 0xC4;
    let mut expected_value: u8 = 0xFF;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let program_1: Vec<u8> = vec![0x9F];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(true);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // Z/N Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    test_value_1 = 0xF0;
    expected_value = 0xFF;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_carry_flag(true);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // Z/N Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), true);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let mut program_1: Vec<u8> = vec![0xD6, test_value_2];
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_b(test_value_2);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // N Flag
    test_flags!(cpu, false, true, false, false);

    test_value_1 = 0xF0;
    test_value_2 = 0xF0;
    expected_value = 0x00;
    program_1[1] = test_value_2;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cpu.registers.set_b(test_value_2);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // Z/N Flags
    test_flags!(cpu, true, true, false, false);

    test_value_1 = 0x10;
    test_value_2 = 0x01;
    expected_value = 0x0F;
    program_1[1] = test_value_2;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // N/H Flag
    test_flags!(cpu, false, true, true, false);

    test_value_1 = 0x10;
    test_value_2 = 0x20;
    expected_value = test_value_1.wrapping_sub(test_value_2);
    program_1[1] = test_value_2;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // C Flag
    test_flags!(cpu, false, true, false, true);

    test_value_1 = 0x00;
    test_value_2 = 0x01;
    expected_value = 0xFF;
    program_1[1] = test_value_2;
    cpu = CPU::new(Rc::clone(&memory_ref));
    cpu.load(&program_1);
    cpu.registers.set_a(test_value_1);
    cycles = cpu.execute_next();
    assert_eq!(cycles, 2);
    assert_eq!(cpu.registers.get_a(), expected_value);
    // H/C Flag
    test_flags!(cpu, false, true, true, true);
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let mut program_1: Vec<u8> = vec![0xE8, test_value_2 as u8];
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 4);
    assert_eq!(cpu.registers.get_sp(), expected_value);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x0FFF;
    test_value_2 = 0x01;
    test_value_2_abs = 0x01;
    expected_value = test_value_1.wrapping_add(test_value_2_abs);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    program_1[1] = test_value_2 as u8;
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 4);
    assert_eq!(cpu.registers.get_sp(), expected_value);
    // H Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xFFFF;
    test_value_2 = 0x01;
    test_value_2_abs = 0x01;
    expected_value = 0x0;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    program_1[1] = test_value_2 as u8;
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 4);
    assert_eq!(cpu.registers.get_sp(), expected_value);
    // H/C Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    test_value_1 = 0xDDFF;
    test_value_2 = -128;
    test_value_2_abs = 128;
    expected_value = test_value_1.wrapping_sub(test_value_2_abs);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    program_1[1] = test_value_2 as u8;
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 4);
    assert_eq!(cpu.registers.get_sp(), expected_value);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xD000;
    test_value_2 = -1;
    test_value_2_abs = 1;
    expected_value = test_value_1.wrapping_sub(test_value_2_abs);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    program_1[1] = test_value_2 as u8;
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 4);
    assert_eq!(cpu.registers.get_sp(), expected_value);
    // H Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x0000;
    test_value_2 = -1;
    test_value_2_abs = 1;
    expected_value = test_value_1.wrapping_sub(test_value_2_abs);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    program_1[1] = test_value_2 as u8;
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 4);
    assert_eq!(cpu.registers.get_sp(), expected_value);
    // H Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let mut program_1: Vec<u8> = vec![0xF3];
    cpu.load(&program_1);
    cpu.ime = true;
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.ime, false);
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
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let mut program_1: Vec<u8> = vec![0xF8, test_value_2 as u8];
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_hl(), expected_value);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x0FFF;
    test_value_2 = 0x01;
    test_value_2_abs = 0x01;
    expected_value = test_value_1.wrapping_add(test_value_2_abs);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    program_1[1] = test_value_2 as u8;
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_hl(), expected_value);
    // H Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xFFFF;
    test_value_2 = 0x01;
    test_value_2_abs = 0x01;
    expected_value = 0x0;
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    program_1[1] = test_value_2 as u8;
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_hl(), expected_value);
    // H/C Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);

    test_value_1 = 0xDDFF;
    test_value_2 = -128;
    test_value_2_abs = 128;
    expected_value = test_value_1.wrapping_sub(test_value_2_abs);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    program_1[1] = test_value_2 as u8;
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_hl(), expected_value);
    // No Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), false);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0xD000;
    test_value_2 = -1;
    test_value_2_abs = 1;
    expected_value = test_value_1.wrapping_sub(test_value_2_abs);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    program_1[1] = test_value_2 as u8;
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_hl(), expected_value);
    // H Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), false);

    test_value_1 = 0x0000;
    test_value_2 = -1;
    test_value_2_abs = 1;
    expected_value = test_value_1.wrapping_sub(test_value_2_abs);
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    program_1[1] = test_value_2 as u8;
    cpu.load(&program_1);
    cpu.registers.set_sp(test_value_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 3);
    assert_eq!(cpu.registers.get_hl(), expected_value);
    // H Flags
    assert_eq!(cpu.registers.get_zero_flag(), false);
    assert_eq!(cpu.registers.get_negative_flag(), false);
    assert_eq!(cpu.registers.get_half_carry_flag(), true);
    assert_eq!(cpu.registers.get_carry_flag(), true);
}
test_ld_r16!(0xF9, test_0xf9_ld_sp_hl, set_sp, get_sp, set_hl, get_hl);
test_ld_r8_imm16!(0xFA, test_0xfa_ld__imm16__a, set_a, get_a);
#[test]
fn test_0xfb_ei() {
    let mut memory_ref = Rc::new(RefCell::new(RAM::new()));
    let mut cpu = CPU::new(Rc::clone(&memory_ref));
    let mut program_1: Vec<u8> = vec![0xFB];
    cpu.load(&program_1);
    let mut cycles = cpu.execute_next();
    assert_eq!(cycles, 1);
    assert_eq!(cpu.ime, true);
}
test_cp_a_imm8!(0xFE, test_0xfe_cp_a_imm8);
test_rst!(0xFF, test_0xef_rst_38);