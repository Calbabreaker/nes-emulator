mod bus;
mod cpu;

use crate::cpu::CPU;

fn main() {
    let cpu = CPU::new();
    print!("Bus read 123: {}", cpu.bus.read_byte(123));
}
