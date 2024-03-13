pub struct RAM {
    memory: [u8; 65536],
}

impl RAM {
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
    }
}
