use std::fs::File;
use std::io::Read;
use iced::{widget::{Column, Text, Row, Scrollable, Button}, Element};
use iced::{widget::{column, button, text, row, scrollable}};
use iced::widget::pane_grid::Axis;
use iced::widget::PaneGrid;
use crate::GB::instructions::{Instruction};
use crate::GB::memory::{ROM, RAM};

#[derive(Default)]
pub struct MainWindow {
    value: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    Decrement,
    LoadBios(String)
}

impl MainWindow {
    fn load_bios(&mut self, path: &str) -> Result<(), std::io::Error> {
        let mut file = File::open(path)?;
        let mut buffer = [0u8; 256];
        file.read_exact(&mut buffer)?;
        // self.rom.memory = buffer;
        Ok(())
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
            _ => {

            }
        }
    }

    pub fn view(&self) -> Column<Message> {
        // We use a column: a simple vertical layout
        let b_p = button("+").on_press(Message::Increment);
        let b_m = button("-").on_press(Message::Decrement);
        let t = text(self.value).size(50);
        let c = column![b_p, t, b_m];
        c
    }
}

// fn update(message: Message, state: &mut State) {
//     match message {
//         Message::LoadBios(path) => {
//             state.load_bios(&path).unwrap();
//         }
//     }
// }

fn view(state: &State) -> Column<Message> {
    Column::new()
        .push(Row::new()
            .push(Text::new("BIOS Viewer"))
            .push(Button::new(Text::new("Load BIOS")).on_click(Message::LoadBios))
        )
        .push(Scrollable::new(
            PaneGrid::new(
                Axis::Horizontal,
                Ratio::
            )
                .columns(|_| FixedColumn { width: 50 })
                .push_column("Address", |row, index| {
                    Text::new(format!("{:04X}", index)).into_element()
                })
                .push_column("Hex", |row, index| {
                    Text::new(format!("{:02X}", state.rom.memory[index])).into_element()
                })
                .push_column("Assembly", |row, index| {
                    let instruction = state.instructions.get(index as usize);
                    let instruction_text = match instruction {
                        Some(instr) => format!("{} ({})", instr.name, instr.cycles),
                        None => String::from("UNKNOWN"),
                    };
                    Text::new(instruction_text).into_element()
                })
                .rows(0..state.rom.memory.len())
                .into_element()
        ))

}