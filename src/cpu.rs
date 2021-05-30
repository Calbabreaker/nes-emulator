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
        self.sp = 0xff;
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
            self.flags &= !(flag as u8);
        }
    }

    fn set_flag_zero_negative(&mut self, value: u8) {
        self.set_flag(Flag::Zero, value == 0);
        self.set_flag(Flag::Negative, value & 0x80 != 0);
    }

    // returns: (address, data at address)
    fn fetch_data(&mut self, addressing_mode: AddressingMode, add_cycles: bool) -> (u16, u8) {
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

                // some instructions need additional clock cycle when changing page
                if add_cycles && address & 0xff00 != address_abs & 0xff00 {
                    self.cycles += 1;
                }

                return (address, self.bus.read_byte(address));
            }

            AddressingMode::AbsoluteY => {
                let address_abs = self.bus.read_word(self.pc);
                let address = address_abs + self.y as u16;
                self.pc += 2;

                if add_cycles && address & 0xff00 != address_abs & 0xff00 {
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

                if add_cycles && address & 0xff00 != address_abs & 0xff00 {
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
            // register loads
            0xa9 => return (CPU::lda, AddressingMode::Immediate, 2, "lda"),
            0xad => return (CPU::lda, AddressingMode::Absolute, 4, "lda"),
            0xbd => return (CPU::lda, AddressingMode::AbsoluteX, 4, "lda"),
            0xb9 => return (CPU::lda, AddressingMode::AbsoluteY, 4, "lda"),
            0xa5 => return (CPU::lda, AddressingMode::ZeroPage, 3, "lda"),
            0xb5 => return (CPU::lda, AddressingMode::ZeroPageX, 4, "lda"),
            0xa1 => return (CPU::lda, AddressingMode::IndexedIndirect, 6, "lda"),
            0xb1 => return (CPU::lda, AddressingMode::IndirectIndexed, 5, "lda"),

            0xa2 => return (CPU::ldx, AddressingMode::Immediate, 2, "ldx"),
            0xae => return (CPU::ldx, AddressingMode::Absolute, 4, "ldx"),
            0xbe => return (CPU::ldx, AddressingMode::AbsoluteY, 4, "ldx"),
            0xa6 => return (CPU::ldx, AddressingMode::ZeroPage, 3, "ldx"),
            0xb6 => return (CPU::ldx, AddressingMode::ZeroPageY, 4, "ldx"),

            0xa0 => return (CPU::ldy, AddressingMode::Immediate, 2, "ldy"),
            0xac => return (CPU::ldy, AddressingMode::Absolute, 4, "ldy"),
            0xbc => return (CPU::ldy, AddressingMode::AbsoluteX, 4, "ldy"),
            0xa4 => return (CPU::ldy, AddressingMode::ZeroPage, 3, "ldy"),
            0xb4 => return (CPU::ldy, AddressingMode::ZeroPageX, 4, "ldy"),

            // register stores
            0x8d => return (CPU::sta, AddressingMode::Absolute, 4, "sta"),
            0x9d => return (CPU::sta, AddressingMode::AbsoluteX, 5, "sta"),
            0x99 => return (CPU::sta, AddressingMode::AbsoluteY, 5, "sta"),
            0x85 => return (CPU::sta, AddressingMode::ZeroPage, 3, "sta"),
            0x95 => return (CPU::sta, AddressingMode::ZeroPageX, 4, "sta"),
            0x81 => return (CPU::sta, AddressingMode::IndexedIndirect, 6, "sta"),
            0x91 => return (CPU::sta, AddressingMode::IndirectIndexed, 6, "sta"),

            0x8e => return (CPU::stx, AddressingMode::Absolute, 4, "stx"),
            0x86 => return (CPU::stx, AddressingMode::ZeroPage, 3, "stx"),
            0x96 => return (CPU::stx, AddressingMode::ZeroPageY, 4, "stx"),

            0x8c => return (CPU::sty, AddressingMode::Absolute, 4, "sty"),
            0x84 => return (CPU::sty, AddressingMode::ZeroPage, 3, "sty"),
            0x94 => return (CPU::sty, AddressingMode::ZeroPageX, 4, "sty"),

            // register transfers
            0xaa => return (CPU::tax, AddressingMode::Implied, 2, "tax"),
            0xa8 => return (CPU::tay, AddressingMode::Implied, 2, "tay"),
            0xba => return (CPU::tsx, AddressingMode::Implied, 2, "tsx"),
            0x8a => return (CPU::txa, AddressingMode::Implied, 2, "txa"),
            0x9a => return (CPU::txs, AddressingMode::Implied, 2, "txs"),
            0x98 => return (CPU::tya, AddressingMode::Implied, 2, "tya"),

            // stack operations
            0x48 => return (CPU::pha, AddressingMode::Implied, 3, "pha"),
            0x08 => return (CPU::php, AddressingMode::Implied, 3, "php"),
            0x68 => return (CPU::pla, AddressingMode::Implied, 4, "pla"),
            0x28 => return (CPU::plp, AddressingMode::Implied, 4, "plp"),

            // shift operations
            0x0a => return (CPU::asl, AddressingMode::Accumulator, 2, "asl"),
            0x0e => return (CPU::asl, AddressingMode::Absolute, 6, "asl"),
            0x1e => return (CPU::asl, AddressingMode::AbsoluteX, 7, "asl"),
            0x06 => return (CPU::asl, AddressingMode::ZeroPage, 5, "asl"),
            0x16 => return (CPU::asl, AddressingMode::ZeroPageX, 6, "asl"),

            0x4a => return (CPU::lsr, AddressingMode::Accumulator, 2, "lsr"),
            0x4e => return (CPU::lsr, AddressingMode::Absolute, 6, "lsr"),
            0x5e => return (CPU::lsr, AddressingMode::AbsoluteX, 7, "lsr"),
            0x46 => return (CPU::lsr, AddressingMode::ZeroPage, 5, "lsr"),
            0x56 => return (CPU::lsr, AddressingMode::ZeroPageX, 6, "lsr"),

            0x2a => return (CPU::rol, AddressingMode::Accumulator, 2, "rol"),
            0x2e => return (CPU::rol, AddressingMode::Absolute, 6, "rol"),
            0x3e => return (CPU::rol, AddressingMode::AbsoluteX, 7, "rol"),
            0x26 => return (CPU::rol, AddressingMode::ZeroPage, 5, "rol"),
            0x36 => return (CPU::rol, AddressingMode::ZeroPageX, 6, "rol"),

            0x6a => return (CPU::ror, AddressingMode::Accumulator, 2, "ror"),
            0x6e => return (CPU::ror, AddressingMode::Absolute, 6, "ror"),
            0x7e => return (CPU::ror, AddressingMode::AbsoluteX, 7, "ror"),
            0x66 => return (CPU::ror, AddressingMode::ZeroPage, 5, "ror"),
            0x76 => return (CPU::ror, AddressingMode::ZeroPageX, 6, "ror"),

            // logic operations
            0x29 => return (CPU::and, AddressingMode::Immediate, 2, "and"),
            0x2d => return (CPU::and, AddressingMode::Absolute, 4, "and"),
            0x3d => return (CPU::and, AddressingMode::AbsoluteX, 4, "and"),
            0x39 => return (CPU::and, AddressingMode::AbsoluteY, 4, "and"),
            0x25 => return (CPU::and, AddressingMode::ZeroPage, 3, "and"),
            0x35 => return (CPU::and, AddressingMode::ZeroPageX, 4, "and"),
            0x21 => return (CPU::and, AddressingMode::IndexedIndirect, 6, "and"),
            0x31 => return (CPU::and, AddressingMode::IndirectIndexed, 5, "and"),

            0x2c => return (CPU::bit, AddressingMode::Absolute, 4, "bit"),
            0x24 => return (CPU::bit, AddressingMode::ZeroPage, 3, "bit"),

            0x49 => return (CPU::eor, AddressingMode::Immediate, 2, "eor"),
            0x4d => return (CPU::eor, AddressingMode::Absolute, 4, "eor"),
            0x5d => return (CPU::eor, AddressingMode::AbsoluteX, 4, "eor"),
            0x59 => return (CPU::eor, AddressingMode::AbsoluteY, 4, "eor"),
            0x45 => return (CPU::eor, AddressingMode::ZeroPage, 3, "eor"),
            0x55 => return (CPU::eor, AddressingMode::ZeroPageX, 4, "eor"),
            0x41 => return (CPU::eor, AddressingMode::IndexedIndirect, 6, "eor"),
            0x51 => return (CPU::eor, AddressingMode::IndirectIndexed, 5, "eor"),

            0x09 => return (CPU::ora, AddressingMode::Immediate, 2, "ora"),
            0x0d => return (CPU::ora, AddressingMode::Absolute, 4, "ora"),
            0x1d => return (CPU::ora, AddressingMode::AbsoluteX, 4, "ora"),
            0x19 => return (CPU::ora, AddressingMode::AbsoluteY, 4, "ora"),
            0x05 => return (CPU::ora, AddressingMode::ZeroPage, 3, "ora"),
            0x15 => return (CPU::ora, AddressingMode::ZeroPageX, 4, "ora"),
            0x01 => return (CPU::ora, AddressingMode::IndexedIndirect, 6, "ora"),
            0x11 => return (CPU::ora, AddressingMode::IndirectIndexed, 5, "ora"),

            // arithmetic operations
            0x69 => return (CPU::adc, AddressingMode::Immediate, 2, "adc"),
            0x6d => return (CPU::adc, AddressingMode::Absolute, 4, "adc"),
            0x7d => return (CPU::adc, AddressingMode::AbsoluteX, 4, "adc"),
            0x79 => return (CPU::adc, AddressingMode::AbsoluteY, 4, "adc"),
            0x65 => return (CPU::adc, AddressingMode::ZeroPage, 3, "adc"),
            0x75 => return (CPU::adc, AddressingMode::ZeroPageX, 4, "adc"),
            0x61 => return (CPU::adc, AddressingMode::IndexedIndirect, 6, "adc"),
            0x71 => return (CPU::adc, AddressingMode::IndirectIndexed, 5, "adc"),

            0xe9 => return (CPU::sbc, AddressingMode::Immediate, 2, "sbc"),
            0xed => return (CPU::sbc, AddressingMode::Absolute, 4, "sbc"),
            0xfd => return (CPU::sbc, AddressingMode::AbsoluteX, 4, "sbc"),
            0xf9 => return (CPU::sbc, AddressingMode::AbsoluteY, 4, "sbc"),
            0xe5 => return (CPU::sbc, AddressingMode::ZeroPage, 3, "sbc"),
            0xf5 => return (CPU::sbc, AddressingMode::ZeroPageX, 4, "sbc"),
            0xe1 => return (CPU::sbc, AddressingMode::IndexedIndirect, 6, "sbc"),
            0xf1 => return (CPU::sbc, AddressingMode::IndirectIndexed, 5, "sbc"),

            0xc9 => return (CPU::cmp, AddressingMode::Immediate, 2, "cmp"),
            0xcd => return (CPU::cmp, AddressingMode::Absolute, 4, "cmp"),
            0xdd => return (CPU::cmp, AddressingMode::AbsoluteX, 4, "cmp"),
            0xd9 => return (CPU::cmp, AddressingMode::AbsoluteY, 4, "cmp"),
            0xc5 => return (CPU::cmp, AddressingMode::ZeroPage, 3, "cmp"),
            0xd5 => return (CPU::cmp, AddressingMode::ZeroPageX, 4, "cmp"),
            0xc1 => return (CPU::cmp, AddressingMode::IndexedIndirect, 6, "cmp"),
            0xd1 => return (CPU::cmp, AddressingMode::IndirectIndexed, 5, "cmp"),

            0xe0 => return (CPU::cpx, AddressingMode::Immediate, 2, "cpx"),
            0xec => return (CPU::cpx, AddressingMode::Absolute, 4, "cpx"),
            0xe4 => return (CPU::cpx, AddressingMode::ZeroPage, 3, "cpx"),

            0xc0 => return (CPU::cpy, AddressingMode::Immediate, 2, "cpy"),
            0xcc => return (CPU::cpy, AddressingMode::Absolute, 4, "cpy"),
            0xc4 => return (CPU::cpy, AddressingMode::ZeroPage, 3, "cpy"),

            // increment
            0xee => return (CPU::inc, AddressingMode::Absolute, 6, "inc"),
            0xfe => return (CPU::inc, AddressingMode::AbsoluteX, 7, "inc"),
            0xe6 => return (CPU::inc, AddressingMode::ZeroPage, 5, "inc"),
            0xf6 => return (CPU::inc, AddressingMode::ZeroPageX, 6, "inc"),

            0xe8 => return (CPU::inx, AddressingMode::Implied, 2, "inx"),
            0xc8 => return (CPU::iny, AddressingMode::Implied, 2, "iny"),

            // decrement
            0xce => return (CPU::dec, AddressingMode::Absolute, 6, "dec"),
            0xde => return (CPU::dec, AddressingMode::AbsoluteX, 7, "dec"),
            0xc6 => return (CPU::dec, AddressingMode::ZeroPage, 5, "dec"),
            0xd6 => return (CPU::dec, AddressingMode::ZeroPageX, 6, "dec"),

            0xca => return (CPU::dex, AddressingMode::Implied, 2, "dex"),
            0x88 => return (CPU::dey, AddressingMode::Implied, 2, "dey"),

            // control operations
            0x4c => return (CPU::jmp, AddressingMode::Absolute, 3, "jmp"),
            0x6c => return (CPU::jmp, AddressingMode::Indirect, 5, "jmp"),

            0x00 => return (CPU::brk, AddressingMode::Implied, 7, "brk"),
            0x20 => return (CPU::jsr, AddressingMode::Absolute, 6, "jsr"),
            0x40 => return (CPU::rti, AddressingMode::Implied, 6, "rti"),
            0x60 => return (CPU::rts, AddressingMode::Implied, 6, "rts"),

            // branch operations
            0x90 => return (CPU::bcc, AddressingMode::Relative, 2, "bcc"),
            0xb0 => return (CPU::bcs, AddressingMode::Relative, 2, "bcs"),
            0xf0 => return (CPU::beq, AddressingMode::Relative, 2, "beq"),
            0x30 => return (CPU::bmi, AddressingMode::Relative, 2, "bmi"),
            0xd0 => return (CPU::bne, AddressingMode::Relative, 2, "bne"),
            0x10 => return (CPU::bpl, AddressingMode::Relative, 2, "bpl"),
            0x50 => return (CPU::bvc, AddressingMode::Relative, 2, "bvc"),
            0x70 => return (CPU::bvs, AddressingMode::Relative, 2, "bvs"),

            // flag operations
            0x18 => return (CPU::clc, AddressingMode::Implied, 2, "clc"),
            0xd8 => return (CPU::cld, AddressingMode::Implied, 2, "cld"),
            0x58 => return (CPU::cli, AddressingMode::Implied, 2, "cli"),
            0xb8 => return (CPU::clv, AddressingMode::Implied, 2, "clv"),
            0x38 => return (CPU::sec, AddressingMode::Implied, 2, "sec"),
            0xf8 => return (CPU::sed, AddressingMode::Implied, 2, "sed"),
            0x78 => return (CPU::sei, AddressingMode::Implied, 2, "sei"),

            // does nothing
            0xea => return (CPU::nop, AddressingMode::Implied, 2, "nop"),

            _ => panic!(
                "Instruction with opcode '0x{:x}' not supported! PC: {}",
                opcode, self.pc
            ),
        }
    }

    // begin instructions!

    fn lda(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, true);
        self.set_flag_zero_negative(data);
        self.a = data;
    }

    fn ldx(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, true);
        self.set_flag_zero_negative(data);
        self.x = data;
    }

    fn ldy(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, true);
        self.set_flag_zero_negative(data);
        self.y = data;
    }

    fn sta(&mut self, mode: AddressingMode) {
        let (address, _) = self.fetch_data(mode, false);
        self.bus.write_byte(address, self.a);
    }

    fn stx(&mut self, mode: AddressingMode) {
        let (address, _) = self.fetch_data(mode, false);
        self.bus.write_byte(address, self.x);
    }

    fn sty(&mut self, mode: AddressingMode) {
        let (address, _) = self.fetch_data(mode, false);
        self.bus.write_byte(address, self.y);
    }

    fn tax(&mut self, mode: AddressingMode) {
        self.x = self.a;
        self.set_flag_zero_negative(self.x);
    }

    fn tay(&mut self, mode: AddressingMode) {
        self.y = self.a;
        self.set_flag_zero_negative(self.y);
    }

    fn tsx(&mut self, mode: AddressingMode) {
        self.x = self.sp;
        self.set_flag_zero_negative(self.x);
    }

    fn txa(&mut self, mode: AddressingMode) {
        self.a = self.x;
        self.set_flag_zero_negative(self.a);
    }

    fn txs(&mut self, mode: AddressingMode) {
        self.sp = self.x;
        self.set_flag_zero_negative(self.sp);
    }

    fn tya(&mut self, mode: AddressingMode) {
        self.a = self.y;
        self.set_flag_zero_negative(self.a);
    }

    fn pha(&mut self, mode: AddressingMode) {
        self.bus.write_byte(0x0100 + self.sp as u16, self.a);
        self.sp -= 1;
    }

    fn php(&mut self, mode: AddressingMode) {
        self.bus.write_byte(0x0100 + self.sp as u16, self.flags);
        self.sp -= 1;
    }

    fn pla(&mut self, mode: AddressingMode) {
        self.sp += 1;
        let data = self.bus.read_byte(0x0100 + self.sp as u16);
        self.a = data;
        self.set_flag_zero_negative(self.a);
    }

    fn plp(&mut self, mode: AddressingMode) {
        self.sp += 1;
        let data = self.bus.read_byte(0x0100 + self.sp as u16);
        self.flags = data;
    }

    fn asl(&mut self, mode: AddressingMode) {
        let (address, data) = self.fetch_data(mode, false);
        let result = data << 1;

        self.set_flag(Flag::Carry, data & 0x80 != 0);
        self.set_flag_zero_negative(result);

        if matches!(mode, AddressingMode::Accumulator) {
            self.a = result;
        } else {
            self.bus.write_byte(address, result);
        }
    }

    fn lsr(&mut self, mode: AddressingMode) {
        let (address, data) = self.fetch_data(mode, false);
        let result = data >> 1;

        self.set_flag(Flag::Carry, data & 0x01 != 0);
        self.set_flag_zero_negative(result);

        if matches!(mode, AddressingMode::Accumulator) {
            self.a = result;
        } else {
            self.bus.write_byte(address, result);
        }
    }

    fn rol(&mut self, mode: AddressingMode) {
        let (address, data) = self.fetch_data(mode, false);
        let result = (data << 1) | self.get_flag(Flag::Carry) as u8;

        self.set_flag(Flag::Carry, data & 0x80 != 0);
        self.set_flag_zero_negative(result);

        if matches!(mode, AddressingMode::Accumulator) {
            self.a = result;
        } else {
            self.bus.write_byte(address, result);
        }
    }

    fn ror(&mut self, mode: AddressingMode) {
        let (address, data) = self.fetch_data(mode, false);
        let result = (data >> 1) | (self.get_flag(Flag::Carry) as u8) << 7;

        self.set_flag(Flag::Carry, data & 0x01 != 0);
        self.set_flag_zero_negative(result);

        if matches!(mode, AddressingMode::Accumulator) {
            self.a = result;
        } else {
            self.bus.write_byte(address, result);
        }
    }

    fn and(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, true);
        self.a &= data;
        self.set_flag_zero_negative(self.a);
    }

    fn bit(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, false);
        let result = self.a & data;

        self.set_flag(Flag::Zero, result == 0);
        self.set_flag(Flag::Overflow, data & (1 << 6) != 0);
        self.set_flag(Flag::Negative, data & (1 << 7) != 0);
    }

    fn eor(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, true);
        self.a ^= data;
        self.set_flag_zero_negative(self.a);
    }

    fn ora(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, true);
        self.a |= data;
        self.set_flag_zero_negative(self.a);
    }

    fn adc(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, true);

        let result_word = self.a as u16 + data as u16 + self.get_flag(Flag::Carry) as u16;
        let result = result_word as u8;

        self.set_flag_zero_negative(result);
        self.set_flag(Flag::Carry, result_word > 0xff);
        self.set_flag(
            Flag::Overflow,
            (!(self.a ^ data) & (self.a ^ result)) & 0x80 != 0,
        );

        self.a = result;
    }

    fn sbc(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, true);

        // invert data and use the same code as adc
        data = !data;

        let result_word = self.a as u16 + data as u16 + self.get_flag(Flag::Carry) as u16;
        let result = result_word as u8;

        self.set_flag_zero_negative(result);
        self.set_flag(Flag::Carry, result_word > 0xff);
        self.set_flag(
            Flag::Overflow,
            (!(self.a ^ data) & (self.a ^ result)) & 0x80 != 0,
        );

        self.a = result;
    }

    fn cmp(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, true);
        let result = self.a - data;

        self.set_flag(Flag::Carry, self.a >= result);
        self.set_flag_zero_negative(result);
    }

    fn cpx(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, false);
        let result = self.x - data;

        self.set_flag(Flag::Carry, self.x >= result);
        self.set_flag_zero_negative(result);
    }

    fn cpy(&mut self, mode: AddressingMode) {
        let (_, data) = self.fetch_data(mode, false);
        let result = self.y - data;

        self.set_flag(Flag::Carry, self.y >= result);
        self.set_flag_zero_negative(result);
    }

    fn inc(&mut self, mode: AddressingMode) {
        let (address, data) = self.fetch_data(mode, false);
        let result = data + 1;

        self.set_flag_zero_negative(result);
        self.bus.write_byte(address, result);
    }

    fn inx(&mut self, mode: AddressingMode) {
        self.x += 1;
        self.set_flag_zero_negative(self.x);
    }

    fn iny(&mut self, mode: AddressingMode) {
        self.y += 1;
        self.set_flag_zero_negative(self.y);
    }

    fn dec(&mut self, mode: AddressingMode) {
        let (address, data) = self.fetch_data(mode, false);
        let result = data - 1;

        self.set_flag_zero_negative(result);
        self.bus.write_byte(address, result);
    }

    fn dex(&mut self, mode: AddressingMode) {
        self.x -= 1;
        self.set_flag_zero_negative(self.x);
    }

    fn dey(&mut self, mode: AddressingMode) {
        self.y -= 1;
        self.set_flag_zero_negative(self.y);
    }

    fn jmp(&mut self, mode: AddressingMode) {
        let (address, _) = self.fetch_data(mode, false);
        self.pc = address;
    }

    fn brk(&mut self, mode: AddressingMode) {
        self.bus.write_word(0x0100 + (self.sp as u16), self.pc);
        self.sp -= 2;

        let flags = self.flags | Flag::Break as u8;
        self.bus.write_byte(0x0100 + (self.sp as u16), flags);
        self.sp -= 1;

        self.pc = self.bus.read_word(0xfffe);
    }

    fn nop(&mut self, mode: AddressingMode) {
        // does nothing
    }
}
