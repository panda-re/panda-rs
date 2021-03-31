mod qcows;
use std::fmt;
use panda_sys::{panda_set_library_mode, panda_init, panda_run};
use std::os::raw::c_char;
use std::ffi::CString;
use std::mem::transmute;
use super::{inventory, Callback, PPPCallbackSetup, sys, PandaArgs};

/// Architecture of the guest system
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Arch {
    i386,
    x86_64,
    Arm,
    Mips,
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::i386 => "i386",
            Self::x86_64 => "x86_64",
            Self::Arm => "arm",
            Self::Mips => "mips",
        })
    }
}

/// Builder for creating PANDA instances. Only for use in libpanda mode.
#[derive(Default)]
pub struct Panda {
    expect_prompt: Option<String>,
    generic_qcow: Option<String>,
    os_version: Option<String>,
    qcow: Option<String>,
    raw_monitor: bool,
    graphics: bool,
    os: String,
    mem: Option<String>,
    arch: Option<Arch>,
    extra_args: Vec<String>,
    replay: Option<String>,
    configurable: bool,
}

impl Panda {
    /// Create a new PANDA instance.
    ///
    /// ### Example
    /// ```rust
    /// # use panda::prelude::*;
    /// Panda::new()
    ///     .generic("x86_64")
    ///     .run();
    /// ```
    pub fn new() -> Self {
        Self {
            os: "linux".into(),
            ..Default::default()
        }
    }

    /// Add an argument for PANDA
    ///
    /// ### Example
    /// ```rust
    /// # use panda::prelude::*;
    /// Panda::new()
    ///     .arg("-nomonitor")
    ///     .run();
    /// ```
    pub fn arg<S: Into<String>>(&mut self, arg: S) -> &mut Self {
        self.extra_args.push(arg.into());

        self
    }

    /// Add a set of extra arguments for PANDA
    ///
    /// ### Example
    /// ```rust
    /// # use panda::prelude::*;
    /// Panda::new()
    ///     .args(&["-panda", "callstack_instr"])
    ///     .run();
    /// ```
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            self.arg(arg.as_ref());
        }

        self
    }

    /// Sets the architecture of the guest
    ///
    /// ### Example
    /// ```rust
    /// # use panda::{prelude::*, Arch};
    /// Panda::new()
    ///     .arch(Arch::i386)
    ///     .run();
    /// ```
    pub fn arch(&mut self, arch: Arch) -> &mut Self {
        self.arch = Some(arch);

        self
    }

    /// Set the machine to PANDA's configurable machine
    ///
    /// ### Example
    /// ```rust
    /// # use panda::{prelude::*, Arch};
    /// Panda::new()
    ///     .configurable()
    ///     .run();
    /// ```
    pub fn configurable(&mut self) -> &mut Self {
        self.configurable = true;

        self
    }

    // Don't pass `-nographic` to QEMU, allowing you use to use a monitor
    ///
    /// ### Example
    /// ```rust
    /// # use panda::prelude::*;
    /// Panda::new()
    ///     .enable_graphics()
    ///     .run();
    /// ```
    pub fn enable_graphics(&mut self) -> &mut Self {
        self.graphics = true;

        self
    }

    /// Regular expression describing the prompt exposed by the guest on a serial console. Used in
    /// order to know when running a command has finished with its output.
    pub fn expect_prompt<S: Into<String>>(&mut self, prompt_regex: S) -> &mut Self {
        self.expect_prompt = Some(prompt_regex.into());

        self
    }

    /// Set the available memory. If restoring from a snapshot or viewing a replay, this must be
    /// the same as when the replay/snapshot was taken.
    pub fn mem<S: Into<String>>(&mut self, mem: S) -> &mut Self {
        self.mem = Some(mem.into());

        self
    }
    
    /// Use generic PANDA Qcow for run
    ///
    /// ### Example
    /// ```rust
    /// # use panda::prelude::*;
    /// Panda::new()
    ///     .generic("x86_64")
    ///     .run();
    /// ```
    pub fn generic<S: Into<String>>(&mut self, generic: S) -> &mut Self {
        self.generic_qcow = Some(generic.into());
        
        self
    }
    
    /// Run the given replay in the PANDA instance. Equivalent to `-replay [name]` from the PANDA
    /// command line.
    ///
    /// ### Example
    /// ```rust
    /// # use panda::prelude::*;
    /// Panda::new()
    ///     .replay("grep_recording")
    ///     .run();
    /// ```
    pub fn replay<S: Into<String>>(&mut self, replay: S) -> &mut Self {
        self.replay = Some(replay.into());
        
        self
    }

    /// Load a plugin with args provided by a `PandaArgs` struct.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use panda::prelude::*;
    ///
    /// #[derive(PandaArgs)]
    /// #[name = "stringsearch"]
    /// struct StringSearch {
    ///     str: String
    /// }
    /// 
    /// fn main() {
    ///     Panda::new()
    ///         .generic("x86_64")
    ///         .replay("test")
    ///         .plugin_args(&StringSearch {
    ///             str: "test".into()
    ///         })
    ///         .run();
    /// }
    /// ```
    pub fn plugin_args<T: PandaArgs>(&mut self, args: &T) -> &mut Self {
        self.arg("-panda")
            .arg(args.to_panda_args_str())
    }

    fn get_args(&self) -> Vec<String> {
        let generic_info =
            self.generic_qcow
                .as_ref()
                .map(|generic| qcows::get_supported_image(generic));

        let qcow_path = self.qcow.clone().map(Some).unwrap_or_else(||{
            self.generic_qcow
                .as_ref()
                .map(|generic| qcows::get_generic_path(generic).display().to_string())
        });
        
        let arch = self.arch
            .or_else(|| generic_info.as_ref().map(|x| x.arch))
            .unwrap_or(Arch::x86_64);

        let mem = self.mem
            .as_ref()
            .map(|x| &x[..])
            .or_else(|| generic_info.as_ref().map(|x| x.default_mem))
            .unwrap_or("128M")
            .to_owned();

        assert_eq!(arch, Arch::x86_64, "Only x86_64 is currently supported");

        let mut args = vec![
            "".into(), // filler, argv[0] == path of executable, n/a
            "-L".into(),
            std::env::var("PANDA_PATH")
                .expect("PANDA_PATH not set. Set it to panda's build folder.")
                + "/pc-bios",
            "-m".into(),
            mem,
        ];

        if let Some(qcow) = qcow_path {
            args.push(qcow)
        }

        if let Some(generic) = generic_info {
            args.push("-os".into());
            args.push(generic.os.into());
        }

        if self.configurable {
            args.push("-M".into());
            args.push("configurable".into());
        }

        if !self.graphics {
            args.push("-nographic".into());
        }

        if let Some(replay) = &self.replay {
            args.push("-replay".into());
            args.push(replay.clone());
        }

        args.extend(self.extra_args.clone().into_iter());

        args
    }

    /// Start the PANDA instance with the given settings. This is a blocking operation.
    ///
    /// ### Example
    /// ```rust
    /// # use panda::prelude::*;
    /// Panda::new()
    ///     .generic("x86_64")
    ///     .run();
    /// ```
    pub fn run(&mut self) {
        let args = self.get_args();

        println!("Running with args: {:?}", args);

        let args: Vec<_> = args.into_iter().map(|x| CString::new(x).unwrap()).collect();
        let args_ptrs: Vec<_> = args.iter().map(|s| s.as_ptr()).collect();

        std::env::set_var("PANDA_DIR", std::env::var("PANDA_PATH").unwrap());

        let x = &mut 0i8;
        let empty = &mut (x as *mut c_char);
        unsafe {
            for cb in inventory::iter::<Callback> {
                sys::panda_register_callback(
                    self as *mut _ as _,
                    cb.cb_type,
                    ::core::mem::transmute(cb.fn_pointer)
                );
            }

            panda_set_library_mode(true);
            panda_init(args_ptrs.len() as i32, transmute(args_ptrs.as_ptr()), empty);
            
            for cb in inventory::iter::<PPPCallbackSetup> {
                cb.0();
            }

            panda_run();
        }
    }
}
