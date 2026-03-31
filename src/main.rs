#[macro_use]
extern crate lazy_static;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::{env, thread};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, stdout, Write};
use std::sync::mpsc;
use std::time;
use std::time::{Duration, Instant};
use winit;

mod GB;
#[macro_use]
mod utils;
#[cfg(test)]
mod tests;

use crate::GB::cpu::instructions::Instruction;
use crate::GB::memory::Length;
use crate::GB::ppu::tile::{GbColor, Tile};
use GB::cpu::{CPU};
use crate::GB::addresses;
use crate::GB::cpu::{InterruptType, CPU_INTERRUPT_CYCLES};
use crate::GB::joypad::{JoypadButtonsBits, JoypadDPadBits};
use crate::GB::ppu::PPU;
use crate::GB::types::address::Address;

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

    #[arg(long, default_value = ".\\logs\\output.txt")]
    log_file: String,
}

lazy_static! {
    pub static ref CONSOLE_PALETTE: HashMap<GbColor, char> = HashMap::from([
        (GbColor::White, '█'),
        (GbColor::LightGray, '▓'),
        (GbColor::DarkGray, '▒'),
        (GbColor::Black, '░'),
    ]);
}

fn frame_string(frame: &[GbColor; PPU::SCREEN_PIXELS as usize], doubled: bool) -> String {
    let mut s = "".to_string();
    for i in 0..PPU::SCREEN_LINES {
        for j in 0..PPU::SCREEN_COLUMNS {
            let frame_char = CONSOLE_PALETTE[&frame[j as usize + i as usize * PPU::SCREEN_COLUMNS as usize]];
            s.push(frame_char);
            if doubled {
                s.push(frame_char);
            }
        }
        s.push('\n')
    }
    s
}

fn main() {
    let args = Args::parse();

    let mut gb = GB::GB::new(Option::from(args.bios.clone()));
    gb.insert_cartridge(&args.rom);
    println!("{}", gb.cartridge().as_ref().unwrap());

    let mut ended = false;
    let mut i: u16 = 0;
    let mut cb = false;

    gb.set_use_boot(false);
    let mut file_result = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(args.log_file);
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
            let frame = gb.frame();
            let frame_str = frame_string(frame, true);
            println!("\x1B[2J\x1B[H{}", frame_str);
            // println!("{}", gb.get_frame_string(true));
            println!("S/f: {:?}", (Instant::now() - time).as_secs_f64());
            println!("C/s: {:?}", cycles as f64/(Instant::now() - time).as_secs_f64());

            let joypad = gb.joypad();
            println!("{}", joypad);
            println!("{}", joypad.symbolic_display());
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
                log(file, &mut gb, log_line);
                log_line += 1;
            }
            Err(_) => {}
        }

        gb.tick();
        cycles += 1;
        let elapsed = start.elapsed();
        // if elapsed < cycle_duration {
        //     thread::sleep(cycle_duration - elapsed);
        // }
    }

    {
        // println!("{}\n\n", gb.ppu.get_frame_string(true));
        // let map = gb.ppu.get_bg_map();
        // for i in 0..16 {
        //     for j in 0..16 {
        //         print!("{}", gb.ppu.get_tile(i * 16 + j, false).get_printable_id_map(true));
        //     }
        //     println!()
        // }
        // println!("{}", gb.ppu.get_tile(0, true));
        // println!("{}", gb.ppu.get_tile_map(0));
        // let lines_to_print = 10;
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

    if gb.cpu().instruction_m_cycle() == 0 && gb.cpu().instruction_t_cycle() == 0 {
        let mut s = "".to_string();
        let mut pc = gb.cpu().registers().get_pc();
        let addr = pc;
        let mut read_bytes: usize = 0;
        let mut opcode = gb.read(Address(pc));
        let mut s_ins = "UNKNOWN".to_string();
        let mut opt_ins = gb.cpu().instruction();

        pc += 1;
        read_bytes += 1;

        match gb.cpu().maneging_interrupt() {
            Some(interrupt_type) => {
                match interrupt_type {
                    InterruptType::Joypad => { s_ins = "JoyPad int.".to_string(); }
                    InterruptType::Serial => { s_ins = "Serial int.".to_string(); }
                    InterruptType::Timer => { s_ins = "Timer int.".to_string(); }
                    InterruptType::LCD => { s_ins = "LCD int.".to_string(); }
                    InterruptType::VBlank => { s_ins = "VBlank int.".to_string(); }
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
                            opcode = gb.read(Address(pc));
                            ins = CPU::decode(opcode, cb).unwrap();
                            s += format!("{:02X} ", opcode).as_str();
                            s_ins = ins.name.to_string();
                            pc += 1;
                            read_bytes += 1;
                        }
                        let mut shift: u16 = 0;
                        let mut immediate_val: u16 = 0;
                        for j in read_bytes as u8..ins.size {
                            let val = gb.read(Address(pc)) as u16;
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
                                    let b1 = gb.read(Address(gb.cpu().registers.get_sp() + 1));
                                    let b2 = gb.read(Address(gb.cpu().registers.get_sp() + 2));
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

        let cartridge = gb.cartridge().unwrap();
        {
            let formatted = format!("| {:04} |  {:#06X} |  {} |  {}{}|  {} {{}} |  RxM B: {}/{}  |  {{AF: {:04X}, BC: {:04X}, DE: {:04X}, HL: {:04X}, SP: {:04X}}} | IE: {:02X} | IF: {:02X} | IME: {} | STAT: {:02X} | DIV: {:02X}",
                                    log_line, addr, s, s_ins, " ".repeat(16 - s_ins.len()), gb.ppu(),
                                    // mem_registers,
                                    cartridge.rom_bank(),
                                    cartridge.ram_bank(),
                                    gb.cpu().registers().get_af(), gb.cpu().registers().get_bc(),
                                    gb.cpu().registers().get_de(), gb.cpu().registers().get_hl(),
                                    gb.cpu().registers().get_sp(),
                                    gb.read(addresses::cpu::INTERRUPT_ENABLED_REGISTER),
                                    gb.read(addresses::cpu::INTERRUPT_FLAGS_REGISTER),
                                    if gb.cpu().ime() { "T" } else { "F" },
                                    gb.read(addresses::ppu::STAT_REGISTER),
                                    gb.read(addresses::ppu::LCDC_REGISTER),
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
                    gb.press_button(JoypadButtonsBits::A, true);
                }
                KeyCode::Char('x') => {
                    gb.press_button(JoypadButtonsBits::B, true);
                }
                KeyCode::Char('a') => {
                    gb.press_dpad(JoypadDPadBits::Left, true);
                }
                KeyCode::Char('d') => {
                    gb.press_dpad(JoypadDPadBits::Right, true);
                }
                KeyCode::Char('s') => {
                    gb.press_dpad(JoypadDPadBits::Down, true);
                }
                KeyCode::Char('w') => {
                    gb.press_dpad(JoypadDPadBits::Up, true);
                }
                KeyCode::Char('o') => {
                    gb.press_button(JoypadButtonsBits::Start, true);
                }
                KeyCode::Char('p') => {
                    gb.press_button(JoypadButtonsBits::Select, true);
                }
                KeyCode::Enter => {
                    gb.press_button(JoypadButtonsBits::Start, true);
                }
                KeyCode::Backspace => {
                    gb.press_button(JoypadButtonsBits::Select, true);
                }
                _ => {}
            }
        }
        KeyEventKind::Release => {
            match key_event.code {
                KeyCode::Char('z') => {
                    gb.press_button(JoypadButtonsBits::A, false);
                }
                KeyCode::Char('x') => {
                    gb.press_button(JoypadButtonsBits::B, false);
                }
                KeyCode::Char('a') => {
                    gb.press_dpad(JoypadDPadBits::Left, false);
                }
                KeyCode::Char('d') => {
                    gb.press_dpad(JoypadDPadBits::Right, false);
                }
                KeyCode::Char('s') => {
                    gb.press_dpad(JoypadDPadBits::Down, false);
                }
                KeyCode::Char('w') => {
                    gb.press_dpad(JoypadDPadBits::Up, false);
                }
                KeyCode::Char('o') => {
                    gb.press_button(JoypadButtonsBits::Start, false);
                }
                KeyCode::Char('p') => {
                    gb.press_button(JoypadButtonsBits::Select, false);
                }
                KeyCode::Enter => {
                    gb.press_button(JoypadButtonsBits::Start, false);
                }
                KeyCode::Backspace => {
                    gb.press_button(JoypadButtonsBits::Select, false);
                }
                _ => {}
            }
        }
        _ => {} // Puoi gestire altri tipi di eventi se necessario
    }
}