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
mod types;

use crate::GB::instructions::Instruction;
use crate::GB::memory::Length;
use crate::GB::PPU::tile::Tile;
use GB::memory;
use GB::cpu::{CPU, CPU_CLOCK_SPEED};
use crate::GB::cpu::CPU_INTERRUPT_CYCLES;
use crate::GB::input::{GBInputButtonsBits, GBInputDPadBits};
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
            println!("{}", gb.input.as_ref().borrow());
            println!("{}", gb.input.as_ref().borrow().symbolic_display());
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

    if !(gb.cpu_left_instruction_cycles() > 0) || (gb.is_cpu_managing_interrupt() && gb.cpu_left_instruction_cycles() == CPU_INTERRUPT_CYCLES) {
        let mut memory_borrowed = gb.memory.borrow();
        let mut s = "".to_string();
        let mut pc = gb.cpu.registers.get_pc();
        let addr = pc;
        let mut read_bytes: usize = 0;
        let mut opcode = memory_borrowed.read(pc);
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
                            opcode = memory_borrowed.read(pc);
                            ins = CPU::decode(opcode, cb).unwrap();
                            s += format!("{:02X} ", opcode).as_str();
                            s_ins = ins.name.to_string();
                            pc += 1;
                            read_bytes += 1;
                        }
                        let mut shift: u16 = 0;
                        let mut immediate_val: u16 = 0;
                        for j in read_bytes as u8..ins.size {
                            let val = memory_borrowed.read(pc) as u16;
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

                        // Extra data
                        if !cb {
                            match ins.opcode {
                                0xD9  => {
                                    let b1 = memory_borrowed.read(gb.cpu.registers.get_sp() + 1);
                                    let b2 = memory_borrowed.read(gb.cpu.registers.get_sp() + 2);
                                    s_ins += format!(" (${:02X}{:02X})", b2, b1).as_str();
                                }
                                _ => {}
                            }
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
            let formatted = format!("| {:04} |  {:#06X} |  {} |  {}{}|  {} {} |  RxM B: {}/{}  |  {{AF: {:04X}, BC: {:04X}, DE: {:04X}, HL: {:04X}, SP: {:04X}}} | IE: {:02X} | IF: {:02X} | IME: {} | STAT: {:02X} | DIV: {:02X}",
                                    log_line, addr, s, s_ins, " ".repeat(16 - s_ins.len()), gb.ppu,
                                    mem_registers,
                                    gb.get_cartridge().as_ref().unwrap().get_rom_bank(),
                                    gb.get_cartridge().as_ref().unwrap().get_ram_bank(),
                                    gb.cpu.registers.get_af(), gb.cpu.registers.get_bc(),
                                    gb.cpu.registers.get_de(), gb.cpu.registers.get_hl(),
                                    gb.cpu.registers.get_sp(),
                                    memory_borrowed.read(memory::registers::IE),
                                    memory_borrowed.read(memory::registers::IF),
                                    if gb.cpu.ime { "T" } else { "F" },
                                    memory_borrowed.read(memory::registers::STAT),
                                    memory_borrowed.read(memory::registers::DIV),
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
                KeyCode::Char('a') => {
                    gb.press_dpad(GBInputDPadBits::Left, true);
                }
                KeyCode::Char('d') => {
                    gb.press_dpad(GBInputDPadBits::Right, true);
                }
                KeyCode::Char('s') => {
                    gb.press_dpad(GBInputDPadBits::Down, true);
                }
                KeyCode::Char('w') => {
                    gb.press_dpad(GBInputDPadBits::Up, true);
                }
                KeyCode::Char('o') => {
                    gb.press_button(GBInputButtonsBits::Start, true);
                }
                KeyCode::Char('p') => {
                    gb.press_button(GBInputButtonsBits::Select, true);
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
                KeyCode::Char('a') => {
                    gb.press_dpad(GBInputDPadBits::Left, false);
                }
                KeyCode::Char('d') => {
                    gb.press_dpad(GBInputDPadBits::Right, false);
                }
                KeyCode::Char('s') => {
                    gb.press_dpad(GBInputDPadBits::Down, false);
                }
                KeyCode::Char('w') => {
                    gb.press_dpad(GBInputDPadBits::Up, false);
                }
                KeyCode::Char('o') => {
                    gb.press_button(GBInputButtonsBits::Start, false);
                }
                KeyCode::Char('p') => {
                    gb.press_button(GBInputButtonsBits::Select, false);
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