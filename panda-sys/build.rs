use std::{env, fs};
use std::path::Path;

const MISSING_ERROR: &'static str = "Missing PANDA_PATH. Please set it to the `build` folder in your panda install.";

fn main() {
    if cfg!(feature = "libpanda") {
        println!("libpanda mode enabled");
        let dylib_path = Path::new(&env::var("PANDA_PATH").expect(MISSING_ERROR)).join("x86_64-softmmu");
        println!("cargo:rustc-link-lib=dylib=panda-x86_64");
        println!("cargo:rustc-link-search=native={}", dylib_path.display());

        let out_dir = env::var("OUT_DIR").unwrap();
        let out_dir = Path::new(&out_dir);
        fs::copy(dylib_path.join("libpanda-x86_64.so"), out_dir.join("..").join("..").join("..").join("libpanda-x86_64.so")).unwrap();
    }
}
