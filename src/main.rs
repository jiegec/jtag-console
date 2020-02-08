use jtag_console;

fn main() {
    let dev = jtag_console::Device::new(0x0403, 0x6014);
    println!("{}", dev.is_ok());
}
