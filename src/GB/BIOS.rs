use std::fs::File;
use std::io::Read;
use crate::GB::memory::{Length, Memory};

pub struct BIOS {
    #[cfg(test)]
    pub memory: Memory<u8>,
    #[cfg(not(test))]
    memory: Memory<u8>,
    bios: String,
}

impl BIOS {
    pub  fn new() -> Self {
        Self { memory: Memory::<u8>::new(0, 256), bios: String::from("") }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn load_bios(&mut self, path: &String) -> Result<(), std::io::Error> {
        let mut file = File::open(path)?;
        let mut buffer = vec![0u8; 256];
        file.read_exact(&mut buffer)?;
        self.memory = Memory { memory: buffer };
        self.bios = path.clone();
        Ok(())
    }
}

impl Length for BIOS {
    fn len(&self) -> usize {
        self.memory.len()
    }
}
