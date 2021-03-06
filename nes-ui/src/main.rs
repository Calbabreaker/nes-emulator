fn main() {
    let mut cpu = nes_core::CPU::new();
    // lda #$0a
    cpu.bus.write_byte(0x0000, 0xa9);
    cpu.bus.write_byte(0x0001, 0xff);
    // sec
    cpu.bus.write_byte(0x0002, 0x38);
    // ror A
    cpu.bus.write_byte(0x0003, 0x6a);
    // sta $0200
    cpu.bus.write_byte(0x0004, 0x8d);
    cpu.bus.write_byte(0x0005, 0x00);
    cpu.bus.write_byte(0x0006, 0x02);
    cpu.reset();

    let mut prev_cycles = cpu.bus.cycles_count as i32;
    for _ in 0..4 {
        cpu.execute_next_instruction();
        let cycles_count = cpu.bus.cycles_count as i32;
        println!("Clock cycles took: {}", cycles_count - &prev_cycles);
        prev_cycles = cycles_count;
    }

    println!("Read: 0x{:x}", cpu.bus.read_byte(0x0200));
}
