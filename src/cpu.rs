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

type InstructionFunc = fn(&mut CPU, AddressingMode);

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
        println!("Clock cycles left: {}", self.cycles);
        if self.cycles == 0 {
            let opcode = self.bus.read_byte(self.pc);
            self.pc += 1;

            let (func, addressing_mode, cycles, _) = self.get_instruction_info(opcode);
            func(self, addressing_mode);
            self.cycles += cycles;
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

    // returns: (address, data at address)
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

    // returns: (instruction function pointer, addressing mode, cycles it takes, opcode name)
    fn get_instruction_info(&self, opcode: u8) -> (InstructionFunc, AddressingMode, u8, &str) {
        println!("Opcode: {:x}", opcode);
        match opcode {
            0xa9 => return (CPU::lda, AddressingMode::Immediate, 2, "lda"),
            0xa5 => return (CPU::lda, AddressingMode::ZeroPage, 3, "lda"),
            0xb5 => return (CPU::lda, AddressingMode::ZeroPageX, 4, "lda"),
            0xad => return (CPU::lda, AddressingMode::Absolute, 4, "lda"),
            0xbd => return (CPU::lda, AddressingMode::AbsoluteX, 4, "lda"),
            0xb9 => return (CPU::lda, AddressingMode::AbsoluteY, 4, "lda"),
            0xa1 => return (CPU::lda, AddressingMode::IndexedIndirect, 6, "lda"),
            0xb1 => return (CPU::lda, AddressingMode::IndirectIndexed, 5, "lda"),

            0x85 => return (CPU::sta, AddressingMode::ZeroPage, 3, "sta"),
            0x95 => return (CPU::sta, AddressingMode::ZeroPageX, 4, "sta"),
            0x8d => return (CPU::sta, AddressingMode::Absolute, 4, "sta"),
            0x9d => return (CPU::sta, AddressingMode::AbsoluteX, 5, "sta"),
            0x99 => return (CPU::sta, AddressingMode::AbsoluteY, 5, "sta"),
            0x81 => return (CPU::sta, AddressingMode::IndexedIndirect, 6, "sta"),
            0x91 => return (CPU::sta, AddressingMode::IndirectIndexed, 6, "sta"),

            _ => panic!(
                "Instruction with opcode '0x{:x}' not supported! PC: {}",
                opcode, self.pc
            ),
        }
    }

    fn lda(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode);
        self.a = data;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, self.a & (1 << 7) != 0);
    }

    fn sta(&mut self, mode: AddressingMode) {
        let (address, _) = self.fetch_data(mode);
        self.bus.write_byte(address, self.a);
    }
}
