use crate::cpu::CPU;

pub struct Bus<'a> {
    cpu: CPU<'a>,
    ram: Box<[u8]>,
}

impl<'a> Bus<'a> {
    pub fn new() -> Self {
        let mut bus = Bus {
            cpu: CPU::new(),
            ram: Box::new([0; 1024 * 64]),
        };

        bus.cpu.connect_bus(&mut bus);
        return bus;
    }

    pub fn read(&self, address: u16) -> u8 {
        return self.ram[address as usize];
    }

    pub fn write(&mut self, address: u16, data: u8) {
        self.ram[address as usize] = data;
    }
}
