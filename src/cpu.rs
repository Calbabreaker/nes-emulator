use crate::bus::Bus;

enum Flag {
    Carry = 1 << 0,
    Zero = 1 << 1,
    IntDisable = 1 << 2,
    Decimal = 1 << 3,
    Break = 1 << 4,
    Unused = 1 << 5,
    Overflow = 1 << 6,
    Negative = 1 << 7,
}

pub struct CPU {
    bus: Bus,

    cycles: u8,

    // registers
    pc: u16,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    flags: u8,
}

impl CPU {
    pub fn new() -> Self {
        return CPU {
            bus: Bus::new(),
            pc: 0,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            flags: 0,
            cycles: 0,
        };
    }

    pub fn read(&self, address: u16) -> u8 {
        return self.bus.read(address);
    }

    pub fn write(&mut self, address: u16, data: u8) {
        return self.bus.write(address, data);
    }

    pub fn reset(&mut self) {
        // load program counter from 0xfffc
        let low = self.read(0xfffc);
        let high = self.read(0xfffd);
        self.pc = low as u16 | (high << 8) as u16;

        self.sp = 0xfd;
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.flags = 0;
        self.set_flag(Flag::Unused, true);

        self.cycles = 8;
    }

    pub fn clock(&mut self) {
        if self.cycles == 0 {
            let opcode = self.read(self.pc);
            self.pc += 1;
        }

        self.cycles -= 1;
    }

    fn get_flag(&self, flag: Flag) -> bool {
        return self.flags & (flag as u8) != 0;
    }

    fn set_flag(&mut self, flag: Flag, status: bool) {
        if status {
            self.flags |= flag as u8;
        } else {
            self.flags |= !(flag as u8);
        }
    }
}
