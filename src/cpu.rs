use crate::bus::Bus;

enum Flag {
    Carry = 1 << 0,
    Zero = 1 << 1,
    InterruptDisable = 1 << 2,
    DecimalMode = 1 << 3,
    Break = 1 << 4,
    Unused = 1 << 5,
    Overflow = 1 << 6,
    Negative = 1 << 7,
}

enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

pub struct CPU {
    pub bus: Bus,

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

    pub fn reset(&mut self) {
        self.pc = self.bus.read_word(0xfffc);
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
            let opcode = self.bus.read_byte(self.pc);
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

    // returns (address, data at address)
    fn fetch_data(&mut self, addressing_mode: AddressingMode) -> (u16, u8) {
        match addressing_mode {
            AddressingMode::Implied => return (0, 0),
            AddressingMode::Accumulator => return (0, self.a),
            AddressingMode::Immediate => {
                let address = self.pc;
                self.pc += 1;
                return (address, self.bus.read_byte(address));
            }

            AddressingMode::ZeroPage => {
                let address = self.bus.read_byte(self.pc) as u16;
                self.pc += 1;
                return (address, self.bus.read_byte(address));
            }

            AddressingMode::ZeroPageX => {
                let address = (self.bus.read_byte(self.pc) + self.x) as u16;
                self.pc += 1;
                return (address, self.bus.read_byte(address));
            }

            AddressingMode::ZeroPageY => {
                let address = (self.bus.read_byte(self.pc) + self.y) as u16;
                self.pc += 1;
                return (address, self.bus.read_byte(address));
            }

            AddressingMode::Relative => {
                let data_at_pc = (self.bus.read_byte(self.pc) as i8) as i16;
                let address = (self.pc as i16 + data_at_pc) as u16;
                self.pc += 1;
                return (address, self.bus.read_byte(address));
            }

            AddressingMode::Absolute => {
                let address = self.bus.read_word(self.pc);
                self.pc += 2;
                return (address, self.bus.read_byte(address));
            }

            AddressingMode::AbsoluteX => {
                let address_abs = self.bus.read_word(self.pc);
                let address = address_abs + self.x as u16;
                self.pc += 2;

                // additional clock cycle required when changing page
                if address & 0xff00 != address_abs & 0xff00 {
                    self.cycles += 1;
                }

                return (address, self.bus.read_byte(address));
            }

            AddressingMode::AbsoluteY => {
                let address_abs = self.bus.read_word(self.pc);
                let address = address_abs + self.y as u16;
                self.pc += 2;

                // additional clock cycle required when changing page
                if address & 0xff00 != address_abs & 0xff00 {
                    self.cycles += 1;
                }

                return (address, self.bus.read_byte(address));
            }

            AddressingMode::Indirect => {
                let pointer = self.bus.read_word(self.pc);
                self.pc += 2;

                let address: u16;
                // emulate page boundary crossing bug
                if pointer & 0x00ff == 0x00ff {
                    let low = self.bus.read_byte(pointer) as u16;
                    let high = self.bus.read_byte(pointer & 0xff00) as u16;
                    address = (high << 8) | low;
                } else {
                    address = self.bus.read_word(self.pc);
                }

                return (address, self.bus.read_byte(address));
            }

            AddressingMode::IndexedIndirect => {
                let pointer = (self.bus.read_byte(self.pc) + self.x) as u16;
                let address = self.bus.read_word(pointer);
                self.pc += 1;

                return (address, self.bus.read_byte(address));
            }

            AddressingMode::IndirectIndexed => {
                let pointer = self.bus.read_byte(self.pc) as u16;
                let address_abs = self.bus.read_word(pointer);
                let address = address_abs + self.y as u16;
                self.pc += 1;

                // additional clock cycle required when changing page
                if address & 0xff00 != address_abs & 0xff00 {
                    self.cycles += 1;
                }

                return (address, self.bus.read_byte(address));
            }
        }
    }
}
