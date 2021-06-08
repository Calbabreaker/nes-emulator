pub struct PPU {
    nametables: [[u8; 1024]; 2],
    palletes: [u8; 32],
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            nametables: [[0; 1024]; 2],
            palletes: [0; 32],
        }
    }

    pub fn read_register(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x0007 => 0xff,

            _ => panic!(
                "Reading at address 0x{:x} goes out of range of PPU! (0x0000 to 0x0007)",
                address
            ),
        }
    }

    pub fn write_register(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x0007 => (),

            _ => panic!(
                "Writing at address 0x{:x} goes out of range of PPU! (0x0000 to 0x0007)",
                address
            ),
        }
    }
}
