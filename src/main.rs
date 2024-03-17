mod GB;
use GB::CPU::{CPU};

use std::io::Read;
use std::fs::File;


fn main() {
    let mut gb = CPU::new();
    let program: Vec<u8> = vec![0x00, 0x01, 0xFF, 0xAA];
    gb.load(&program);
    gb.execute_next();
    gb.execute_next();
    println!("{}", gb.registers);
    println!("Hello, world!");
}
