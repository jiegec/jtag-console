use bitvec::prelude::*;
use jtag_console;

fn main() {
    let dev = jtag_console::Device::new(0x0403, 0x6014);
    println!("{}", dev.is_ok());
    let output = bitvec![Lsb0, u8; 0; 32];
    let res = dev.unwrap().shift_dr(&output).unwrap();
    println!("{:?}", res);
    for bit in &*res {
        print!("{}", *bit as usize);
    }
}
