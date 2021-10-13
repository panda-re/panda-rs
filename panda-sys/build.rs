use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

const MISSING_ERROR: &'static str = "Missing PANDA_PATH. Please set it to the `build` folder in your panda install or use pip to install the `pandare` package.";
const PYTHON_GET_SITE_PACKAGES: &'static str = r#"import sys; print("\n".join(sys.path))"#;

fn get_site_packages() -> Result<Vec<PathBuf>, std::io::Error> {
    Ok(String::from_utf8_lossy(
        &Command::new("python3")
            .args(&["-c", PYTHON_GET_SITE_PACKAGES])
            .output()
            .or_else(|_| {
                Command::new("python")
                    .args(&["-c", PYTHON_GET_SITE_PACKAGES])
                    .output()
            })?
            .stdout[..],
    )
    .split("\n")
    .map(PathBuf::from)
    .collect())
}

fn get_panda_path() -> PathBuf {
    PathBuf::from(
        &env::var("PANDA_PATH")
            .map(PathBuf::from)
            .or_else(|_| -> Result<_, ()> {
                println!("cargo:warning=PANDA_PATH is missing");
                Ok(get_site_packages()
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
                    .ok_or(())?)
            })
            .expect(MISSING_ERROR),
    )
}

#[cfg(feature = "x86_64")]
const ARCH: &str = "x86_64";

#[cfg(feature = "i386")]
const ARCH: &str = "i386";

#[cfg(feature = "arm")]
const ARCH: &str = "arm";

#[cfg(feature = "aarch64")]
const ARCH: &str = "aarch64";

#[cfg(feature = "ppc")]
const ARCH: &str = "ppc";

#[cfg(feature = "mips")]
const ARCH: &str = "mips";

#[cfg(feature = "mipsel")]
const ARCH: &str = "mipsel";

#[cfg(feature = "mips64")]
const ARCH: &str = "mips64";

fn main() {
    if cfg!(feature = "libpanda") {
        println!("libpanda mode enabled");
        let dylib_path = get_panda_path().join(format!("{}-softmmu", ARCH));
        println!("cargo:rustc-link-lib=dylib=panda-{}", ARCH);
        println!("cargo:rustc-link-search=native={}", dylib_path.display());

        let out_dir: PathBuf = env::var("OUT_DIR").unwrap().into();
        fs::copy(
            dylib_path.join(format!("libpanda-{}.so", ARCH)),
            out_dir
                .join("..")
                .join("..")
                .join("..")
                .join(format!("libpanda-{}.so", ARCH)),
        )
        .unwrap();
    }
}
