pub struct Bus {
    pub ram: Box<[u8]>,
    pub cycles_count: u32,
}

impl Bus {
    pub fn new() -> Self {
        return Bus {
            ram: Box::new([0; 1024 * 64]),
            cycles_count: 0,
        };
    }

    pub fn clock(&mut self) {
        self.cycles_count += 1;
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        self.clock();
        self.ram[address as usize]
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let low = self.read_byte(address) as u16;
        let high = self.read_byte(address + 1) as u16;
        (high << 8) | low
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        self.clock();
        self.ram[address as usize] = data;
    }
}
