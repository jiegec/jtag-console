use bitvec::prelude::*;
use jtag_console;
use std::fs::File;
use std::io::Read;

fn main() {
    let mut dev = jtag_console::Device::new(0x0403, 0x6014).unwrap();

    let output = bitvec![Lsb0, u8; 0; 32];
    let res = dev.shift_dr(&output).unwrap();
    println!("IDCODE: {:?}", res);

    let jprogram = bitvec![Lsb0, u8; 1, 1, 0, 1, 0, 0];
    let res = dev.shift_ir(&jprogram).unwrap();
    println!("JPROGRAM: {:?}", res);

    let bypass = bitvec![Lsb0, u8; 1, 1, 1, 1, 1, 1];
    let res = dev.shift_ir(&bypass).unwrap();
    println!("BYPASS: {:?}", res);

    dev.jtag_reset();

    let cfg_in = bitvec![Lsb0, u8; 1, 0, 1, 0, 0, 0];
    let res = dev.shift_ir(&cfg_in).unwrap();
    println!("CFG_IN: {:?}", res);

    let mut bitstream = File::open("test.bit").expect("open bitstream");
    let mut data = Vec::new();
    bitstream.read_to_end(&mut data).expect("read bitstream");
    let bitstream = BitVec::from_slice(&data[158..]);
    let res = dev.shift_dr(&bitstream).unwrap();

    dev.jtag_reset();
    let jstart = bitvec![Lsb0, u8; 0, 0, 1, 1, 0, 0];
    let res = dev.shift_ir(&jstart).unwrap();
    println!("JSTART: {:?}", res);

    dev.jtag_reset();
    println!("NOT WORKING");
}
