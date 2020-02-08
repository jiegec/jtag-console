use bindgen;
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=ftdi1");
    println!("cargo:rustc-link-search=/usr/local/lib");

    let out_dir = env::var("OUT_DIR").unwrap();
    let bindings = bindgen::Builder::default()
        .header("/usr/local/include/libftdi1/ftdi.h")
        .generate().unwrap();
    
    let out_path = PathBuf::from(out_dir);
    bindings.write_to_file(out_path.join("bindings.rs")).unwrap();
}