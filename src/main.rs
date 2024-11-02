#[macro_use]
extern crate lazy_static;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::{env, thread};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, stdout, Write};
use std::sync::mpsc;
use std::time;
use std::time::{Duration, Instant};
use winit;

mod GB;
mod gui;
#[macro_use]
mod utils;
#[cfg(test)]
mod tests;

use crate::GB::instructions::Instruction;
use crate::GB::memory::Length;
use crate::GB::PPU::tile::Tile;
use GB::memory;
use GB::CPU::{CPU, CPU_CLOCK_SPEED};
use crate::GB::input::GBInputButtonsBits;
use crate::GB::memory::interrupts::InterruptFlagsMask;
// use winit::{event, event_loop, window};

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

    let mut gb = GB::GB::new(Option::from(args.bios.clone()));
    gb.insert_cartridge(&args.rom);
    println!("{}", gb.get_cartridge().as_ref().unwrap());

    let mut ended = false;
    let mut i: u16 = 0;
    let mut cb = false;

    gb.set_use_boot(false);
    let mut file_result = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("logs\\output.txt");
    let mut log_line: u64 = 0;

    // ------------------------------------------------------------------------

    // Frame time
    let frame_duration = Duration::from_millis(33);
    let cycle_duration = Duration::from_nanos(238);
    let mut cycles: u64 = 0;
    let mut time = Instant::now();

    // Input TX/RX Channels
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            if event::poll(Duration::from_millis(30)).unwrap() {
                if let Event::Key(key_event) = event::read().unwrap() {
                    tx.send(key_event).unwrap();
                }
            }
        }
    });

    // Running in loop Game Boy execution
    'running: loop {
        let start = Instant::now();

        if (cycles % (GB::CYCLES_PER_FRAME)) == 0  {
            println!("\x1B[2J\x1B[H{}", gb.get_frame_string(true));
            // println!("{}", gb.get_frame_string(true));
            println!("S/f: {:?}", (Instant::now() - time).as_secs_f64());
            println!("C/s: {:?}", cycles as f64/(Instant::now() - time).as_secs_f64());
            stdout().flush().unwrap();
            time = Instant::now();
            cycles = 0;
        }

        // Polling input
        if let Some(key_event) = rx.try_recv().ok() {
            if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Esc {
                break;
            }
            manage_gb_input_event(&mut gb, key_event);
        }


        match file_result {
            Ok(ref mut file) => {
                log(file, &gb, log_line);
                log_line += 1;
            }
            Err(_) => {}
        }

        gb.cycle();
        cycles += 1;
        let elapsed = start.elapsed();
        // if elapsed < cycle_duration {
        //     thread::sleep(cycle_duration - elapsed);
        // }
    }

    if false {
        let mut i: u32 = 0;
        let mut last_frame_time = time::Instant::now();
        let target_frame_time = time::Duration::from_secs_f64(1.0 / 60.0);
        let mut cycles_per_frame = CPU_CLOCK_SPEED / 60;
        let mut cycles = 0;
        let mut debug_i = 142268;

        println!();
        println!("| n°   |  Adr. |  Hex       |  Instruction    |");
        println!("+------+-------+------------+-----------------+");
        while i < 200000 {
            if false {
                if !(gb.cpu_cycles > 0) {
                    if debug_i == i {
                        print!("");
                    }
                    let mut s = "".to_string();
                    let mut pc = gb.cpu.registers.get_pc();
                    let addr = pc;
                    let mut read_bytes: usize = 0;
                    let mut opcode = gb.memory.borrow().read(pc);
                    let mut s_ins = "UNKNOWN".to_string();
                    let mut opt_ins = CPU::decode(opcode, false);

                    pc += 1;
                    read_bytes += 1;

                    match opt_ins {
                        None => {
                            s += format!("{:02X} ", opcode).as_str();
                        }
                        Some(mut ins) => {
                            s += format!("{:02X} ", opcode).as_str();
                            cb = opcode == 0xCB;
                            if cb {
                                opcode = gb.memory.borrow().read(pc);
                                ins = CPU::decode(opcode, cb).unwrap();
                                s += format!("{:02X} ", opcode).as_str();
                                s_ins = ins.name.to_string();
                                pc += 1;
                                read_bytes += 1;
                            }
                            let mut shift: u16 = 0;
                            let mut immediate_val: u16 = 0;
                            for j in read_bytes as u8..ins.size {
                                let val = gb.memory.borrow().read(pc) as u16;
                                s += format!("{:02X} ", val).as_str();
                                immediate_val |= val << shift;
                                pc += 1;
                                read_bytes += 1;
                                shift += 8;
                            }

                            s_ins = ins.name.to_string();
                            match ins.size {
                                2 => {
                                    let fmt = format!("${:02X}", immediate_val);
                                    let new_s_ins = s_ins.replace("imm8", fmt.as_str());
                                    s_ins = new_s_ins;
                                    let fmt = format!("{}", immediate_val as i8);
                                    let new_s_ins = s_ins.replace("e8", fmt.as_str());
                                    s_ins = new_s_ins;
                                }
                                3 => {
                                    let fmt = format!("${:04X}", immediate_val);
                                    let new_s_ins = s_ins.replace("imm16", fmt.as_str());
                                    s_ins = new_s_ins;
                                }
                                _ => {}
                            }
                        }
                    }

                    for j in read_bytes as u8..3 {
                        s += "   ";
                        read_bytes += 1;
                    }

                    let mem_registers = gb.memory.borrow().get_memory_registers();
                    {
                        let formatted = format!("| {:04} |  {:#06X} |  {} |  {}{}|  {} {} RxM B: {}/{}, {{AF: {:04X}, BC: {:04X}, DE: {:04X}, HL: {:04X}, SP: {:04X}}}, IE: {}, IF: {}, IME: {}",
                                 i, addr, s, s_ins, " ".repeat(15 - s_ins.len()), gb.ppu,
                                 mem_registers,
                                 gb.get_cartridge().as_ref().unwrap().get_rom_bank(),
                                 gb.get_cartridge().as_ref().unwrap().get_ram_bank(),
                                 gb.cpu.registers.get_af(), gb.cpu.registers.get_bc(),
                                 gb.cpu.registers.get_de(), gb.cpu.registers.get_hl(),
                                 gb.cpu.registers.get_sp(),
                                 gb.memory.borrow().read(memory::registers::IE),
                                 gb.memory.borrow().read(memory::registers::IF),
                                 if gb.cpu.ime {"T"} else {"F"},
                        );
                        println!("{}", formatted);
                        match file_result {
                            Ok(ref mut file) => {
                                writeln!(file, "{}", formatted);
                            }
                            _ => {}
                        }
                    }
                }
            }
            if false {
                // let now = time::Instant::now();
                // let delta_time = now.duration_since(last_frame_time);
                if cycles == 0 {
                    println!("{}", gb.ppu.get_frame_string(true));
                }
            }
            if gb.cpu_cycles == 0 {
                i += 1;
            }
            gb.cycle();
            cycles = (cycles + 1) % cycles_per_frame;
        }
        println!("+------+---------+------------+-----------------+");
        println!();
    }

    {
        println!("{}\n\n", gb.ppu.get_frame_string(true));
        // let map = gb.ppu.get_bg_map();
        // for i in 0..16 {
        //     for j in 0..16 {
        //         print!("{}", gb.ppu.get_tile(i * 16 + j, false).get_printable_id_map(true));
        //     }
        //     println!()
        // }
        // println!("{}", gb.ppu.get_tile(0, true));
        // println!("{}", gb.ppu.get_tile_map(0));
        let lines_to_print = 10;
        // println!("{}\n", &(gb.ppu.get_bg_map())[..256*2*3*lines_to_print+lines_to_print]); // x3 because characters used in UTF-8 occupy 3 bytes + 1 byte per line for carriage return
        // println!("SCX {} | SCY {}", gb.ppu.get_scx(), gb.ppu.get_scy())
    }

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

fn log(log_channel: &mut File, gb: &GB::GB, log_line: u64) {
    let mut i: u32 = 0;
    let mut cb = false;
    let mut cycles = 0;
    let mut debug_i = 142268;

    if !(gb.cpu_cycles > 0) {
        let mut s = "".to_string();
        let mut pc = gb.cpu.registers.get_pc();
        let addr = pc;
        let mut read_bytes: usize = 0;
        let mut opcode = gb.memory.borrow().read(pc);
        let mut s_ins = "UNKNOWN".to_string();
        let mut opt_ins = CPU::decode(opcode, false);

        pc += 1;
        read_bytes += 1;

        match gb.is_managing_interrupt() {
            (interrupt_type, Some(cycle)) => {
                if cycle == 0 {
                    match interrupt_type {
                        InterruptFlagsMask::JoyPad => { s_ins = "JoyPad int.".to_string(); }
                        InterruptFlagsMask::Serial => { s_ins = "Serial int.".to_string(); }
                        InterruptFlagsMask::Timer => { s_ins = "Timer int.".to_string(); }
                        InterruptFlagsMask::LCD => { s_ins = "LCD int.".to_string(); }
                        InterruptFlagsMask::VBlank => { s_ins = "VBlank int.".to_string(); }
                    }
                }
            }
            _ => {
                match opt_ins {
                    None => {
                        s += format!("{:02X} ", opcode).as_str();
                    }
                    Some(mut ins) => {
                        s += format!("{:02X} ", opcode).as_str();
                        cb = opcode == 0xCB;
                        if cb {
                            opcode = gb.memory.borrow().read(pc);
                            ins = CPU::decode(opcode, cb).unwrap();
                            s += format!("{:02X} ", opcode).as_str();
                            s_ins = ins.name.to_string();
                            pc += 1;
                            read_bytes += 1;
                        }
                        let mut shift: u16 = 0;
                        let mut immediate_val: u16 = 0;
                        for j in read_bytes as u8..ins.size {
                            let val = gb.memory.borrow().read(pc) as u16;
                            s += format!("{:02X} ", val).as_str();
                            immediate_val |= val << shift;
                            pc += 1;
                            read_bytes += 1;
                            shift += 8;
                        }

                        s_ins = ins.name.to_string();
                        match ins.size {
                            2 => {
                                let fmt = format!("${:02X}", immediate_val);
                                let new_s_ins = s_ins.replace("imm8", fmt.as_str());
                                s_ins = new_s_ins;
                                let fmt = format!("{}", immediate_val as i8);
                                let new_s_ins = s_ins.replace("e8", fmt.as_str());
                                s_ins = new_s_ins;
                            }
                            3 => {
                                let fmt = format!("${:04X}", immediate_val);
                                let new_s_ins = s_ins.replace("imm16", fmt.as_str());
                                s_ins = new_s_ins;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        for j in read_bytes as u8..3 {
            s += "   ";
            read_bytes += 1;
        }

        let mem_registers = gb.memory.borrow().get_memory_registers();
        {
            let formatted = format!("| {:04} |  {:#06X} |  {} |  {}{}|  {} {} RxM B: {}/{}, {{AF: {:04X}, BC: {:04X}, DE: {:04X}, HL: {:04X}, SP: {:04X}}}, IE: {:02X}, IF: {:02X}, IME: {}",
                                    log_line, addr, s, s_ins, " ".repeat(15 - s_ins.len()), gb.ppu,
                                    mem_registers,
                                    gb.get_cartridge().as_ref().unwrap().get_rom_bank(),
                                    gb.get_cartridge().as_ref().unwrap().get_ram_bank(),
                                    gb.cpu.registers.get_af(), gb.cpu.registers.get_bc(),
                                    gb.cpu.registers.get_de(), gb.cpu.registers.get_hl(),
                                    gb.cpu.registers.get_sp(),
                                    gb.memory.borrow().read(memory::registers::IE),
                                    gb.memory.borrow().read(memory::registers::IF),
                                    if gb.cpu.ime { "T" } else { "F" },
            );
            writeln!(log_channel, "{}", formatted).expect("TODO: panic message");
        }
    }

}

fn manage_gb_input_event(gb: &mut GB::GB, key_event: KeyEvent) {
    match key_event.kind {
        KeyEventKind::Press => {
            match key_event.code {
                KeyCode::Char('z') => {
                    gb.press_button(GBInputButtonsBits::A, true);
                }
                KeyCode::Char('x') => {
                    gb.press_button(GBInputButtonsBits::B, true);
                }
                KeyCode::Char('o') => {
                    gb.press_button(GBInputButtonsBits::Start, true);
                }
                KeyCode::Enter => {
                    gb.press_button(GBInputButtonsBits::Start, true);
                }
                KeyCode::Backspace => {
                    gb.press_button(GBInputButtonsBits::Select, true);
                }
                _ => {}
            }
        }
        KeyEventKind::Release => {
            match key_event.code {
                KeyCode::Char('z') => {
                    gb.press_button(GBInputButtonsBits::A, false);
                }
                KeyCode::Char('x') => {
                    gb.press_button(GBInputButtonsBits::B, false);
                }
                KeyCode::Char('o') => {
                    gb.press_button(GBInputButtonsBits::Start, false);
                }
                KeyCode::Enter => {
                    gb.press_button(GBInputButtonsBits::Start, false);
                }
                KeyCode::Backspace => {
                    gb.press_button(GBInputButtonsBits::Select, false);
                }
                _ => {}
            }
        }
        _ => {} // Puoi gestire altri tipi di eventi se necessario
    }
}