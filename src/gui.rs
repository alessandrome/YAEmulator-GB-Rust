use std::fs::File;
use std::io::Read;
use iced::{widget::{Column, Text, Row, Scrollable, Button}};
use iced::{widget::{column, button, text, row, scrollable}};
use iced::{Application, Command, Element, Settings, Alignment};
use crate::GB::GB;
use crate::GB::instructions::{Instruction};
use crate::GB::memory::{ROM, RAM};

// #[derive(Default)]
// pub enum MainWindow {
//     Window(State)
// }

#[derive(Default)]
pub struct MainWindow {
    value: i32,
    gb: GB
}

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    Decrement,
    LoadBios(Option<String>)
}

impl Application for MainWindow {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = Option<String>;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let m;
        match _flags {
            Some(f) => m = Message::LoadBios(Some(f)),
            None => m = Message::LoadBios(Some(String::from(".\\bios\\gb_bios.bin"))),
        }
        let mut gb = GB::default();
        let status = MainWindow {
            value: 0,
            gb: gb
        };
        (status, Command::none())
    }

    fn title(&self) -> String {
        String::from("Main")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
            Message::LoadBios(bios) => {
                // let _ = match bios {
                //     Some(path) => self.gb.rom.load_bios(&path),
                //     _ => Ok({})
                // };
            }
            _ => {

            }
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
        let r = row![Text::new("BIOS")];
        column![r, button("+").on_press(Message::Increment)].into()
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
