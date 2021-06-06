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

enum AdressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}

pub struct CPU {
    pub bus: Bus,

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
        for _ in 1..8 {
            self.bus.clock();
        }
    }

    pub fn execute_next_instruction(&mut self) {
        let opcode = self.bus.read_byte(self.pc);
        self.pc += 1;
        self.execute_instruction(opcode);
    }

    // helper flag funtions
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

    // stack helper functions
    fn push_byte(&mut self, value: u8) {
        self.bus.write_byte(0x100 + self.sp as u16, value);
        self.sp -= 1;
    }

    fn push_word(&mut self, value: u16) {
        self.push_byte((value >> 8) as u8);
        self.push_byte(value as u8);
    }

    fn pop_byte(&mut self) -> u8 {
        self.sp += 1;
        self.bus.read_byte(0x100 + self.sp as u16)
    }

    fn pop_word(&mut self) -> u16 {
        self.pop_byte() as u16 | (self.pop_byte() as u16) << 8
    }

    fn read_operand_address(&mut self, mode: AdressingMode) -> u16 {
        match mode {
            AdressingMode::Immediate => {
                let address = self.pc;
                self.pc += 1;
                address
            }

            AdressingMode::ZeroPage => {
                let address = self.bus.read_byte(self.pc) as u16;
                self.pc += 1;
                address
            }

            AdressingMode::ZeroPageX => {
                let address = (self.bus.read_byte(self.pc) + self.x) as u16;
                self.pc += 1;
                address
            }

            AdressingMode::ZeroPageY => {
                let address = (self.bus.read_byte(self.pc) + self.y) as u16;
                self.pc += 1;
                address
            }

            AdressingMode::Relative => {
                let data_at_pc = (self.bus.read_byte(self.pc) as i8) as i16;
                let address = (self.pc as i16 + data_at_pc) as u16;
                self.pc += 1;
                address
            }

            AdressingMode::Absolute => {
                let address = self.bus.read_word(self.pc);
                self.pc += 2;
                address
            }

            AdressingMode::AbsoluteX => {
                let address_abs = self.bus.read_word(self.pc);
                let address = address_abs + self.x as u16;
                self.pc += 2;

                // some instructions need additional clock cycle when changing page
                // if add_cycles && address & 0xff00 != address_abs & 0xff00 {
                //     self.cycles += 1;
                // }

                address
            }

            AdressingMode::AbsoluteY => {
                let address_abs = self.bus.read_word(self.pc);
                let address = address_abs + self.y as u16;
                self.pc += 2;

                // some instructions need additional clock cycle when changing page
                // if add_cycles && address & 0xff00 != address_abs & 0xff00 {
                //     self.cycles += 1;
                // }

                address
            }

            AdressingMode::Indirect => {
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

                address
            }

            AdressingMode::IndirectX => {
                let pointer = (self.bus.read_byte(self.pc) + self.x) as u16;
                let address = self.bus.read_word(pointer);
                self.pc += 1;

                address
            }

            AdressingMode::IndirectY => {
                let pointer = self.bus.read_byte(self.pc) as u16;
                let address_abs = self.bus.read_word(pointer);
                let address = address_abs + self.y as u16;
                self.pc += 1;

                // if add_cycles && address & 0xff00 != address_abs & 0xff00 {
                //     self.cycles += 1;
                // }

                address
            }
        }
    }

    // returns (data, address)
    fn read_operand(&mut self, mode: AdressingMode) -> (u8, u16) {
        let address = self.read_operand_address(mode);
        let data = self.bus.read_byte(address);
        (data, address)
    }

    fn execute_instruction(&mut self, opcode: u8) {
        println!("Opcode: {:x}", opcode);
        match opcode {
            // register loads
            0xa9 => self.lda(AdressingMode::Immediate),
            0xad => self.lda(AdressingMode::Absolute),
            0xbd => self.lda(AdressingMode::AbsoluteX),
            0xb9 => self.lda(AdressingMode::AbsoluteY),
            0xa5 => self.lda(AdressingMode::ZeroPage),
            0xb5 => self.lda(AdressingMode::ZeroPageX),
            0xa1 => self.lda(AdressingMode::IndirectX),
            0xb1 => self.lda(AdressingMode::IndirectY),

            0xa2 => self.ldx(AdressingMode::Immediate),
            0xae => self.ldx(AdressingMode::Absolute),
            0xbe => self.ldx(AdressingMode::AbsoluteY),
            0xa6 => self.ldx(AdressingMode::ZeroPage),
            0xb6 => self.ldx(AdressingMode::ZeroPageY),

            0xa0 => self.ldy(AdressingMode::Immediate),
            0xac => self.ldy(AdressingMode::Absolute),
            0xbc => self.ldy(AdressingMode::AbsoluteX),
            0xa4 => self.ldy(AdressingMode::ZeroPage),
            0xb4 => self.ldy(AdressingMode::ZeroPageX),

            // register stores
            0x8d => self.sta(AdressingMode::Absolute),
            0x9d => self.sta(AdressingMode::AbsoluteX),
            0x99 => self.sta(AdressingMode::AbsoluteY),
            0x85 => self.sta(AdressingMode::ZeroPage),
            0x95 => self.sta(AdressingMode::ZeroPageX),
            0x81 => self.sta(AdressingMode::IndirectX),
            0x91 => self.sta(AdressingMode::IndirectY),

            0x8e => self.stx(AdressingMode::Absolute),
            0x86 => self.stx(AdressingMode::ZeroPage),
            0x96 => self.stx(AdressingMode::ZeroPageY),

            0x8c => self.sty(AdressingMode::Absolute),
            0x84 => self.sty(AdressingMode::ZeroPage),
            0x94 => self.sty(AdressingMode::ZeroPageX),

            // register transfers
            0xaa => self.tax(),
            0xa8 => self.tay(),
            0xba => self.tsx(),
            0x8a => self.txa(),
            0x9a => self.txs(),
            0x98 => self.tya(),

            // stack operations
            0x48 => self.pha(),
            0x08 => self.php(),
            0x68 => self.pla(),
            0x28 => self.plp(),

            // shift operations
            0x0a => self.asl_a(),
            0x0e => self.asl(AdressingMode::Absolute),
            0x1e => self.asl(AdressingMode::AbsoluteX),
            0x06 => self.asl(AdressingMode::ZeroPage),
            0x16 => self.asl(AdressingMode::ZeroPageX),

            0x4a => self.lsr_a(),
            0x4e => self.lsr(AdressingMode::Absolute),
            0x5e => self.lsr(AdressingMode::AbsoluteX),
            0x46 => self.lsr(AdressingMode::ZeroPage),
            0x56 => self.lsr(AdressingMode::ZeroPageX),

            0x2a => self.rol_a(),
            0x2e => self.rol(AdressingMode::Absolute),
            0x3e => self.rol(AdressingMode::AbsoluteX),
            0x26 => self.rol(AdressingMode::ZeroPage),
            0x36 => self.rol(AdressingMode::ZeroPageX),

            0x6a => self.ror_a(),
            0x6e => self.ror(AdressingMode::Absolute),
            0x7e => self.ror(AdressingMode::AbsoluteX),
            0x66 => self.ror(AdressingMode::ZeroPage),
            0x76 => self.ror(AdressingMode::ZeroPageX),

            // logic operations
            0x29 => self.and(AdressingMode::Immediate),
            0x2d => self.and(AdressingMode::Absolute),
            0x3d => self.and(AdressingMode::AbsoluteX),
            0x39 => self.and(AdressingMode::AbsoluteY),
            0x25 => self.and(AdressingMode::ZeroPage),
            0x35 => self.and(AdressingMode::ZeroPageX),
            0x21 => self.and(AdressingMode::IndirectX),
            0x31 => self.and(AdressingMode::IndirectY),

            0x2c => self.bit(AdressingMode::Absolute),
            0x24 => self.bit(AdressingMode::ZeroPage),

            0x49 => self.eor(AdressingMode::Immediate),
            0x4d => self.eor(AdressingMode::Absolute),
            0x5d => self.eor(AdressingMode::AbsoluteX),
            0x59 => self.eor(AdressingMode::AbsoluteY),
            0x45 => self.eor(AdressingMode::ZeroPage),
            0x55 => self.eor(AdressingMode::ZeroPageX),
            0x41 => self.eor(AdressingMode::IndirectX),
            0x51 => self.eor(AdressingMode::IndirectY),

            0x09 => self.ora(AdressingMode::Immediate),
            0x0d => self.ora(AdressingMode::Absolute),
            0x1d => self.ora(AdressingMode::AbsoluteX),
            0x19 => self.ora(AdressingMode::AbsoluteY),
            0x05 => self.ora(AdressingMode::ZeroPage),
            0x15 => self.ora(AdressingMode::ZeroPageX),
            0x01 => self.ora(AdressingMode::IndirectX),
            0x11 => self.ora(AdressingMode::IndirectY),

            // arithmetic operations
            0x69 => self.adc(AdressingMode::Immediate),
            0x6d => self.adc(AdressingMode::Absolute),
            0x7d => self.adc(AdressingMode::AbsoluteX),
            0x79 => self.adc(AdressingMode::AbsoluteY),
            0x65 => self.adc(AdressingMode::ZeroPage),
            0x75 => self.adc(AdressingMode::ZeroPageX),
            0x61 => self.adc(AdressingMode::IndirectX),
            0x71 => self.adc(AdressingMode::IndirectY),

            0xe9 => self.sbc(AdressingMode::Immediate),
            0xed => self.sbc(AdressingMode::Absolute),
            0xfd => self.sbc(AdressingMode::AbsoluteX),
            0xf9 => self.sbc(AdressingMode::AbsoluteY),
            0xe5 => self.sbc(AdressingMode::ZeroPage),
            0xf5 => self.sbc(AdressingMode::ZeroPageX),
            0xe1 => self.sbc(AdressingMode::IndirectX),
            0xf1 => self.sbc(AdressingMode::IndirectY),

            0xc9 => self.cmp(AdressingMode::Immediate),
            0xcd => self.cmp(AdressingMode::Absolute),
            0xdd => self.cmp(AdressingMode::AbsoluteX),
            0xd9 => self.cmp(AdressingMode::AbsoluteY),
            0xc5 => self.cmp(AdressingMode::ZeroPage),
            0xd5 => self.cmp(AdressingMode::ZeroPageX),
            0xc1 => self.cmp(AdressingMode::IndirectX),
            0xd1 => self.cmp(AdressingMode::IndirectY),

            0xe0 => self.cpx(AdressingMode::Immediate),
            0xec => self.cpx(AdressingMode::Absolute),
            0xe4 => self.cpx(AdressingMode::ZeroPage),

            0xc0 => self.cpy(AdressingMode::Immediate),
            0xcc => self.cpy(AdressingMode::Absolute),
            0xc4 => self.cpy(AdressingMode::ZeroPage),

            // increment
            0xee => self.inc(AdressingMode::Absolute),
            0xfe => self.inc(AdressingMode::AbsoluteX),
            0xe6 => self.inc(AdressingMode::ZeroPage),
            0xf6 => self.inc(AdressingMode::ZeroPageX),

            0xe8 => self.inx(),
            0xc8 => self.iny(),

            // decrement
            0xce => self.dec(AdressingMode::Absolute),
            0xde => self.dec(AdressingMode::AbsoluteX),
            0xc6 => self.dec(AdressingMode::ZeroPage),
            0xd6 => self.dec(AdressingMode::ZeroPageX),

            0xca => self.dex(),
            0x88 => self.dey(),

            // control operations
            0x4c => self.jmp(AdressingMode::Absolute),
            0x6c => self.jmp(AdressingMode::Indirect),

            0x00 => self.brk(),
            0x20 => self.jsr(AdressingMode::Absolute),
            0x40 => self.rti(),
            0x60 => self.rts(),

            // branch operations
            // 0x90 => self.bcc(AdressingMode::Relative),
            // 0xb0 => self.bcs(AdressingMode::Relative),
            // 0xf0 => self.beq(AdressingMode::Relative),
            // 0x30 => self.bmi(AdressingMode::Relative),
            // 0xd0 => self.bne(AdressingMode::Relative),
            // 0x10 => self.bpl(AdressingMode::Relative),
            // 0x50 => self.bvc(AdressingMode::Relative),
            // 0x70 => self.bvs(AdressingMode::Relative),

            // // flag operations
            // 0x18 => self.clc(),
            // 0xd8 => self.cld(),
            // 0x58 => self.cli(),
            // 0xb8 => self.clv(),
            // 0x38 => self.sec(),
            // 0xf8 => self.sed(),
            // 0x78 => self.sei(),

            // does nothing
            0xea => self.nop(),

            _ => panic!(
                "Instruction with opcode '0x{:x}' not supported! PC: {}",
                opcode, self.pc
            ),
        }
    }

    // begin instructions!

    fn lda(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);
        self.set_flag_zero_negative(data);
        self.a = data;
    }

    fn ldx(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);
        self.set_flag_zero_negative(data);
        self.x = data;
    }

    fn ldy(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);
        self.set_flag_zero_negative(data);
        self.y = data;
    }

    fn sta(&mut self, mode: AdressingMode) {
        let address = self.read_operand_address(mode);
        self.bus.write_byte(address, self.a);
    }

    fn stx(&mut self, mode: AdressingMode) {
        let address = self.read_operand_address(mode);
        self.bus.write_byte(address, self.x);
    }

    fn sty(&mut self, mode: AdressingMode) {
        let address = self.read_operand_address(mode);
        self.bus.write_byte(address, self.y);
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.set_flag_zero_negative(self.x);
    }

    fn tay(&mut self) {
        self.y = self.a;
        self.set_flag_zero_negative(self.y);
    }

    fn tsx(&mut self) {
        self.x = self.sp;
        self.set_flag_zero_negative(self.x);
    }

    fn txa(&mut self) {
        self.a = self.x;
        self.set_flag_zero_negative(self.a);
    }

    fn txs(&mut self) {
        self.sp = self.x;
        self.set_flag_zero_negative(self.sp);
    }

    fn tya(&mut self) {
        self.a = self.y;
        self.set_flag_zero_negative(self.a);
    }

    fn pha(&mut self) {
        self.push_byte(self.a);
    }

    fn php(&mut self) {
        self.push_byte(self.flags);
    }

    fn pla(&mut self) {
        self.a = self.pop_byte();
        self.set_flag_zero_negative(self.a);
    }

    fn plp(&mut self) {
        self.flags = self.pop_byte();
    }

    fn do_asl(&mut self, data: u8) -> u8 {
        let result = data << 1;
        self.set_flag(Flag::Carry, data & 0x80 != 0);
        self.set_flag_zero_negative(result);
        return result;
    }

    fn asl(&mut self, mode: AdressingMode) {
        let (data, address) = self.read_operand(mode);
        let result = self.do_asl(data);
        self.bus.write_byte(address, result);
    }

    fn asl_a(&mut self) {
        self.a = self.do_asl(self.a);
    }

    fn do_lsr(&mut self, data: u8) -> u8 {
        let result = data >> 1;
        self.set_flag(Flag::Carry, data & 0x01 != 0);
        self.set_flag_zero_negative(result);
        result
    }

    fn lsr(&mut self, mode: AdressingMode) {
        let (data, address) = self.read_operand(mode);
        let result = self.do_lsr(data);
        self.bus.write_byte(address, result);
    }

    fn lsr_a(&mut self) {
        self.a = self.do_lsr(self.a);
    }

    fn do_rol(&mut self, data: u8) -> u8 {
        let result = (data << 1) | self.get_flag(Flag::Carry) as u8;
        self.set_flag(Flag::Carry, data & 0x80 != 0);
        self.set_flag_zero_negative(result);
        result
    }

    fn rol(&mut self, mode: AdressingMode) {
        let (data, address) = self.read_operand(mode);
        let result = self.do_rol(data);
        self.bus.write_byte(address, result);
    }

    fn rol_a(&mut self) {
        self.a = self.do_rol(self.a);
    }

    fn do_ror(&mut self, data: u8) -> u8 {
        let result = (data >> 1) | (self.get_flag(Flag::Carry) as u8) << 7;
        self.set_flag(Flag::Carry, data & 0x01 != 0);
        self.set_flag_zero_negative(result);
        return result;
    }

    fn ror(&mut self, mode: AdressingMode) {
        let (data, address) = self.read_operand(mode);
        let result = self.do_ror(data);
        self.bus.write_byte(address, result);
    }

    fn ror_a(&mut self) {
        self.a = self.do_ror(self.a);
    }

    fn and(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);
        self.a &= data;
        self.set_flag_zero_negative(self.a);
    }

    fn bit(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);
        let result = self.a & data;

        self.set_flag(Flag::Zero, result == 0);
        self.set_flag(Flag::Overflow, data & (1 << 6) != 0);
        self.set_flag(Flag::Negative, data & (1 << 7) != 0);
    }

    fn eor(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);
        self.a ^= data;
        self.set_flag_zero_negative(self.a);
    }

    fn ora(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);
        self.a |= data;
        self.set_flag_zero_negative(self.a);
    }

    fn do_adc(&mut self, data: u8) {
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

    fn adc(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);
        self.do_adc(data);
    }

    fn sbc(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);

        // invert data and use the same code as adc
        self.do_adc(!data);
    }

    fn cmp(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);
        let result = self.a - data;

        self.set_flag(Flag::Carry, self.a >= result);
        self.set_flag_zero_negative(result);
    }

    fn cpx(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);
        let result = self.x - data;

        self.set_flag(Flag::Carry, self.x >= result);
        self.set_flag_zero_negative(result);
    }

    fn cpy(&mut self, mode: AdressingMode) {
        let (data, _) = self.read_operand(mode);
        let result = self.y - data;

        self.set_flag(Flag::Carry, self.y >= result);
        self.set_flag_zero_negative(result);
    }

    fn inc(&mut self, mode: AdressingMode) {
        let (data, address) = self.read_operand(mode);
        let result = data + 1;

        self.set_flag_zero_negative(result);
        self.bus.write_byte(address, result);
    }

    fn inx(&mut self) {
        self.x += 1;
        self.set_flag_zero_negative(self.x);
    }

    fn iny(&mut self) {
        self.y += 1;
        self.set_flag_zero_negative(self.y);
    }

    fn dec(&mut self, mode: AdressingMode) {
        let (data, address) = self.read_operand(mode);
        let result = data - 1;

        self.set_flag_zero_negative(result);
        self.bus.write_byte(address, result);
    }

    fn dex(&mut self) {
        self.x -= 1;
        self.set_flag_zero_negative(self.x);
    }

    fn dey(&mut self) {
        self.y -= 1;
        self.set_flag_zero_negative(self.y);
    }

    fn jmp(&mut self, mode: AdressingMode) {
        let address = self.read_operand_address(mode);
        self.pc = address;
    }

    fn brk(&mut self) {
        self.push_word(self.pc);

        let flags = self.flags | Flag::Break as u8;
        self.push_byte(flags);

        self.pc = self.bus.read_word(0xfffe);
    }

    fn jsr(&mut self, mode: AdressingMode) {
        let address = self.read_operand_address(mode);
        let return_address = self.pc - 1;

        self.push_word(return_address);
        self.pc = address;
    }

    fn rti(&mut self) {
        self.flags = self.pop_byte();
        self.pc = self.pop_word();
    }

    fn rts(&mut self) {
        self.pc = self.pop_word();
        self.pc += 1;
    }

    fn nop(&mut self) {
        // does nothing
    }
}
