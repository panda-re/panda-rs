use std::{env, fs};
use std::path::PathBuf;
use std::process::Command;

const MISSING_ERROR: &'static str = "Missing PANDA_PATH. Please set it to the `build` folder in your panda install or use pip to install the `pandare` package.";
const PYTHON_GET_SITE_PACKAGES: &'static str = r#"import sys; print("\n".join(sys.path))"#;

fn get_site_packages() -> Result<Vec<PathBuf>, std::io::Error> {
    Ok(
        String::from_utf8_lossy(
            &Command::new("python3")
                .args(&["-c", PYTHON_GET_SITE_PACKAGES])
                .output()
                .or_else(|_|
                    Command::new("python")
                        .args(&["-c", PYTHON_GET_SITE_PACKAGES])
                        .output()
                )?
                .stdout[..]
        )
        .split("\n")
        .map(PathBuf::from)
        .collect()
    )
}

fn get_panda_path() -> PathBuf {
    PathBuf::from(
        &env::var("PANDA_PATH")
            .map(PathBuf::from)
            .or_else(|_| -> Result<_, ()> {
                Ok(
                    get_site_packages()
                        .map_err(|_| ())?
                        .into_iter()
                        .filter_map(|site_package_folder| {
                            let path = site_package_folder.join("pandare").join("data");
                            if path.exists() {
                                Some(path)
                            } else {
                                None
                            }
                        })
                        .next()
                        .ok_or(())?
                )
            })
            .expect(MISSING_ERROR)
    )
}

fn main() {
    if cfg!(feature = "libpanda") {
        println!("libpanda mode enabled");
        let dylib_path = get_panda_path().join("x86_64-softmmu");
        println!("cargo:rustc-link-lib=dylib=panda-x86_64");
        println!("cargo:rustc-link-search=native={}", dylib_path.display());

        let out_dir: PathBuf = env::var("OUT_DIR").unwrap().into();
        fs::copy(dylib_path.join("libpanda-x86_64.so"), out_dir.join("..").join("..").join("..").join("libpanda-x86_64.so")).unwrap();
        //println!("cargo:rustc-link-search=dylib={}", out_dir.join("..").join("..").join("..").display());
        //println!("cargo:rustc-link-search=dylib=/lib/x86_64-linux-gnu");
    }
}
