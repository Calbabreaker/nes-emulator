use crate::bus::Bus;

pub struct CPU<'a> {
    bus: Option<&'a mut Bus<'a>>,
}

impl<'a> CPU<'a> {
    pub fn new() -> Self {
        return CPU { bus: None };
    }

    pub fn connect_bus(&mut self, bus: &'a mut Bus<'a>) {
        self.bus = Some(bus);
    }

    pub fn read(&self, address: u16) -> u8 {
        return self.bus.unwrap().read(address);
    }

    pub fn write(&self, address: u16, data: u8) {
        return self.bus.unwrap().write(address, data);
    }
}
