pub struct Bus {
    ram: Box<[u8]>,
}

impl Bus {
    pub fn new() -> Self {
        return Bus {
            ram: Box::new([0; 1024 * 64]),
        };
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        return self.ram[address as usize];
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let low = self.read_byte(address) as u16;
        let high = self.read_byte(address + 1) as u16;
        return (high << 8) | low;
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        self.ram[address as usize] = data;
    }
}
