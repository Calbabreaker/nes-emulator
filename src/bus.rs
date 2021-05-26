pub struct Bus {
    ram: Box<[u8]>,
}

impl Bus {
    pub fn new() -> Self {
        return Bus {
            ram: Box::new([0; 1024 * 64]),
        };
    }

    pub fn read(&self, address: u16) -> u8 {
        return self.ram[address as usize];
    }

    pub fn write(&mut self, address: u16, data: u8) {
        self.ram[address as usize] = data;
    }
}
