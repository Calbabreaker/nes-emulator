use crate::{Catridge, PPU};

// this is technically the cpu bus since only the cpu reads and writes to it
pub struct Bus {
    pub ram: [u8; 2048],
    pub ppu: PPU,
    pub catridge: Option<Catridge>,
    pub cycles_count: u32,
}

impl Bus {
    pub fn new() -> Self {
        return Bus {
            ram: [0; 2048],
            ppu: PPU::new(),
            catridge: None,
            cycles_count: 0,
        };
    }

    pub fn connect_catridge(&mut self, catridge: Catridge) {
        self.catridge = Some(catridge);
    }

    pub fn clock(&mut self) {
        self.cycles_count += 1;
    }

    pub fn clock_multiple(&mut self, times: u8) {
        for _ in 0..times {
            self.clock();
        }
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        self.clock();
        match address {
            0x0000..=0x1fff => self.ram[address as usize & 0x07ff],
            0x2000..=0x3fff => self.ppu.read_register(address & 0x0007),
            _ => 0,
        }
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        self.clock();
        match address {
            0x0000..=0x1fff => self.ram[address as usize & 0x07ff] = data,
            0x2000..=0x3fff => self.ppu.write_register(address & 0x0007, data),
            _ => (),
        }
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let low = self.read_byte(address) as u16;
        let high = self.read_byte(address + 1) as u16;
        (high << 8) | low
    }
}
