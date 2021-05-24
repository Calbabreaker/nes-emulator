mod bus;
mod cpu;

use crate::bus::Bus;

fn main() {
    let bus = Bus::new();
    bus.write(123, 23);
    print!("Bus read 123: {}", bus.read(123));
}
