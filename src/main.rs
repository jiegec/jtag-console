use bitvec::prelude::*;
use jtag_console;

fn main() {
    let mut dev = jtag_console::Device::new(0x0403, 0x6014).unwrap();

    let output = bitvec![Lsb0, u8; 0; 32];
    let res = dev.shift_dr(&output).unwrap();
    println!("{:?}", res);

    let output = bitvec![Lsb0, u8; 1, 0, 0, 1, 0, 0];
    let res = dev.shift_ir(&output).unwrap();
    println!("{:?}", res);

    let output = bitvec![Lsb0, u8; 0; 32];
    let res = dev.shift_dr(&output).unwrap();
    println!("{:?}", res);
}
