mod bus;
mod cpu;

use crate::cpu::CPU;

fn main() {
    let mut cpu = CPU::new();
    cpu.bus.write_byte(0x0000, 0xa9);
    cpu.bus.write_byte(0x0001, 0x0a);
    cpu.bus.write_byte(0x0002, 0x81);
    cpu.bus.write_byte(0x0003, 0x00);
    cpu.reset();

    for i in 1..15 {
        cpu.clock();
    }

    println!("Read: 0x{:x}", cpu.bus.read_byte(0x0aa9));
}
