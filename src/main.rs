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
use crate::GB::instructions::Instruction;
use crate::GB::memory::Length;


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

    let mut gb = GB::GB::new(args.bios.clone());
    gb.insert_cartridge(&args.rom);
    let cartridge_ref =  gb.get_cartridge();
    println!("{}", gb.get_cartridge().as_ref().unwrap());

    let mut ended = false;
    let mut i: u16 = 0;
    let mut cb = false;
    let bios = gb.get_bios();
    while i < bios.len() as u16 {
        let mut s = "".to_string();
        let mut read_bytes: usize = 0;
        let mut opcode = bios.read(i);
        let mut s_ins = "UNKNOWN";
        let mut opt_ins = CPU::decode(opcode, cb);
        i += 1;
        read_bytes += 1;
        match opt_ins {
            None => { s += format!("{:02X} ", opcode).as_str(); }
            Some(mut ins) => {
                s += format!("{:02X} ", opcode).as_str();
                cb = opcode == 0xCB;
                if cb {
                    opcode = bios.read(i);
                    ins = CPU::decode(opcode, cb).unwrap();
                    s += format!("{:02X} ", opcode).as_str();
                    s_ins = ins.name;
                    i += 1;
                    read_bytes +=1;
                }
                for j in read_bytes as u8..ins.size {
                    s += format!("{:02X} ", bios.read(i)).as_str();
                    i += 1;
                    read_bytes +=1;
                }
                s_ins = ins.name;
            }
        }
        for j in read_bytes as u8..3 {
            s += "   ";
            i += 1;
            read_bytes +=1;
        }
        println!("{} |  {}", s, s_ins);
    }

    println!();

    if fs::metadata(&args.rom).is_ok() {
        println!("La ROM \"{}\" esiste!", args.rom);
    } else {
        println!("La ROM non esiste.");
    }

    if fs::metadata(&args.bios).is_ok() {
        println!("Il BIOS \"{}\" esiste!", &args.bios);
    } else {
        println!("Il file non esiste.");
    }

    if let Ok(current_dir) = env::current_dir() {
        println!("Il percorso corrente è: {:?}", current_dir);
    } else {
        eprintln!("Impossibile ottenere il percorso corrente.");
    }
}
