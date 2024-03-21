use crate::GB::CPU::{CPU};

// #[test]
// fn cpu_new_8bit_registers() {
//     let cpu = CPU::new();
//     assert_eq!(cpu.registers.get_a(), 0);
//     assert_eq!(cpu.registers.get_f(), 0);
//     assert_eq!(cpu.registers.get_b(), 0);
//     assert_eq!(cpu.registers.get_c(), 0);
//     assert_eq!(cpu.registers.get_d(), 0);
//     assert_eq!(cpu.registers.get_e(), 0);
//     assert_eq!(cpu.registers.get_h(), 0);
//     assert_eq!(cpu.registers.get_l(), 0);
// }
//
// #[test]
// fn cpu_new_16bit_registers() {
//     let cpu = CPU::new();
//     assert_eq!(cpu.registers.get_af(), 0);
//     assert_eq!(cpu.registers.get_bc(), 0);
//     assert_eq!(cpu.registers.get_de(), 0);
//     assert_eq!(cpu.registers.get_hl(), 0);
//     assert_eq!(cpu.registers.get_sp(), 0);
//     assert_eq!(cpu.registers.get_pc(), 0);
// }
//
// #[test]
// fn cpu_new_16_8bit_registers() {
//     // 16 Bit register should be 0 as the compound of low register is 0 (and should not be altered by access of 8bit register)
//     let cpu = CPU::new();
//     assert_eq!(cpu.registers.get_a(), 0);
//     assert_eq!(cpu.registers.get_f(), 0);
//     assert_eq!(cpu.registers.get_b(), 0);
//     assert_eq!(cpu.registers.get_c(), 0);
//     assert_eq!(cpu.registers.get_d(), 0);
//     assert_eq!(cpu.registers.get_e(), 0);
//     assert_eq!(cpu.registers.get_h(), 0);
//     assert_eq!(cpu.registers.get_l(), 0);
//     assert_eq!(cpu.registers.get_af(), 0);
//     assert_eq!(cpu.registers.get_bc(), 0);
//     assert_eq!(cpu.registers.get_de(), 0);
//     assert_eq!(cpu.registers.get_hl(), 0);
//     assert_eq!(cpu.registers.get_sp(), 0);
//     assert_eq!(cpu.registers.get_pc(), 0);
// }

// #[test]
// fn cpu_nop() {
//     assert_eq!(1, 1)
// }