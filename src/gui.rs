use crate::GB::instructions::Instruction;
use crate::GB::memory::{RAM, ROM};
use crate::GB::GB;
use iced::widget::container;
use iced::widget::pane_grid::{self, Pane, PaneGrid};
use iced::widget::{button, column, row, scrollable, text};
use iced::widget::{Button, Column, Row, Scrollable, Text};
use iced::{Alignment, Application, Command, Element, Settings};
use std::fs::File;
use std::io::Read;
use crate::GB::CPU::CPU;

// #[derive(Default)]
// pub enum MainWindow {
//     Window(State)
// }

struct PaneStatus {
    id: usize,
}

impl PaneStatus {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

// #[derive(Default)]
pub struct MainWindow {
    panes: pane_grid::State<PaneStatus>,
    gb: GB,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadBios(Option<String>),
}

impl Application for MainWindow {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = Option<String>;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        // Init Pane with two vertical panes (Hex - Assembly)
        let (mut panes, _pane) = pane_grid::State::new(PaneStatus::new(0));
        panes.split(pane_grid::Axis::Vertical, _pane, PaneStatus::new(1));

        // Init Status
        let mut gb = GB::new(_flags.unwrap());
        let status = MainWindow {
            panes: panes,
            gb: gb,
        };
        (status, Command::none())
    }

    fn title(&self) -> String {
        String::from("Main")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadBios(bios) => {
                let _ = match bios {
                    Some(path) => self.gb.rom.load_bios(&path),
                    _ => Ok({}),
                };
            }
            _ => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        // let mut left_pane = Pane::new(
        //     Column::new()
        //         .push(Text::new("Hex & Assembly"))
        //         .push(Scrollable::new(
        //             // ... Populate with list elements ...
        //         ))
        // );
        // let row = Element::from(Row::new()
        //     .push(Text::new("Address"))
        //     .push(Text::new("Hex"))
        //     .push(Text::new("Assembly")));
        // container(row).into()
        // let r = row![Text::new("BIOS")];
        // let c = column![r, button("+").on_press(Message::Increment)];
        let ram = &self.gb.cpu.ram;
        let ram = &self.gb.rom;
        PaneGrid::new(&self.panes, |pane, state, maximized| {
            let r1 = row![text("BIOS"), text(state.id)];
            let mut c = Column::new().push(r1);
            let mut i = 0;
            let mut step = 1;
            while i < 256 {
                if state.id == 0 {
                    let mut opcode = ram.read(i as u16);
                    let mut hexes = vec![opcode];
                    let mut hexes_str: String = format!("{:02x}", opcode);
                    let is_cb = opcode == 0xCB;
                    if is_cb {
                        i = i + 1;
                        opcode = ram.read(i as u16);
                        hexes.push(opcode);
                        hexes_str.push_str(&(format!(" {:02x}", opcode)).as_str());
                    }
                    let _ins = CPU::decode(&opcode, is_cb);
                    match (_ins) {
                        Some(ins) => {
                            let mut extra: u16 = 0;
                            if !is_cb {
                                if ins.size == 2 {
                                    i += 1;
                                    extra = ram.read(i as u16) as u16;
                                    opcode = ram.read(i as u16);
                                    hexes.push(opcode);
                                    hexes_str += format!(" {:02x}", opcode).as_str();
                                } else if ins.size == 3 {
                                    i += 1;
                                    extra = ram.read(i as u16) as u16;
                                    opcode = ram.read(i as u16);
                                    hexes.push(opcode);
                                    hexes_str += format!(" {:02x}", opcode).as_str();
                                    i += 1;
                                    extra |= (ram.read(i as u16) as u16) << 8;
                                    opcode = ram.read(i as u16);
                                    hexes.push(opcode);
                                    hexes_str += format!(" {:02x}", opcode).as_str();
                                }
                            }
                        }
                        _ => {}
                    }
                    c = c.push(row![text(hexes_str)]);
                } else {
                    c = c.push(row![text("ex")]);
                }
                i += 1;
            }
            pane_grid::Content::new(c)
        }).into()
    }
}

// impl MainWindow {
//     fn load_bios(&mut self, path: &str) -> Result<(), std::io::Error> {
//         let mut file = File::open(path)?;
//         let mut buffer = [0u8; 256];
//         file.read_exact(&mut buffer)?;
//         // self.rom.memory = buffer;
//         Ok(())
//     }
//
//     pub fn update(&mut self, message: Message) {
//         match message {
//             Message::Increment => {
//                 self.value += 1;
//             }
//             Message::Decrement => {
//                 self.value -= 1;
//             }
//             _ => {
//
//             }
//         }
//     }
//
//     pub fn view(&self) -> Column<Message> {
//         // We use a column: a simple vertical layout
//         let b_p = button("+").on_press(Message::Increment);
//         let b_m = button("-").on_press(Message::Decrement);
//         let t = text(self.value).size(50);
//         let c = column![b_p, t, b_m];
//         c
//     }
// }

// fn update(message: Message, state: &mut State) {
//     match message {
//         Message::LoadBios(path) => {
//             state.load_bios(&path).unwrap();
//         }
//     }
// }
