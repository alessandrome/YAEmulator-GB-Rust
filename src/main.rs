#[macro_use]
extern crate lazy_static;

use std::fs;
use std::env;
use std::io::Read;
use clap::Parser;

mod GB;
mod gui;
#[macro_use]
mod utils;
#[cfg(test)]
mod tests;

use GB::CPU::{CPU};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    bios: String,

    /// Name of the person to greet
    #[arg(short, long)]
    rom: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();

    // let mut cpu = CPU::new();
    // let program: Vec<u8> = vec![0x00, 0x01, 0xFF, 0xAA, 0x04, 0x05, 0x06, 0xBB];
    // cpu.load(&program);
    // for i in 0..5 {
    //     gb.execute_next();
    //     let &instruction = &(CPU::decode(&gb.opcode, gb.opcode == 0xCB)).unwrap();
    //     println!("[{:#04x}] {} -> {} Bytes, {} Cycles", instruction.opcode, instruction.name, instruction.size, instruction.cycles);
    //     println!("{}", gb.registers);
    // }
    if fs::metadata(&args.rom).is_ok() {
        println!("La rom \"{}\" esiste!", args.rom);
    } else {
        println!("La rom non esiste.");
    }

    let mut gb = GB::GB::new(args.bios.clone());
    gb.insert_cartridge(&args.rom);
    let cartridge_ref =  gb.get_cartridge();
    println!("{}", gb.get_cartridge().unwrap());

    if let Ok(current_dir) = env::current_dir() {
        println!("Il percorso corrente è: {:?}", current_dir);
    } else {
        eprintln!("Impossibile ottenere il percorso corrente.");
    }

    if fs::metadata(&args.bios).is_ok() {
        println!("Il file esiste!");
    } else {
        println!("Il file non esiste.");
    }
}
