mod qcows;
use std::fmt;
use panda_sys::{panda_set_library_mode, panda_init, panda_run};
use std::os::raw::c_char;
use std::ffi::CString;
use std::mem::transmute;

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
}

impl Panda {
    /// Create a new PANDA instance.
    ///
    /// ### Example
    /// ```rust
    /// # use panda::{Panda, Arch};
    /// Panda::new()
    ///     .arch(Arch::i386)
    ///     .arg("-nomonitor")
    ///     .mem("2G")
    ///     .run();
    /// ```
    pub fn new() -> Self {
        Self {
            os: "linux".into(),
            ..Default::default()
        }
    }

    /// Add an argument for PANDA
    pub fn arg<S: Into<String>>(&mut self, arg: S) -> &mut Self {
        self.extra_args.push(arg.into());

        self
    }

    /// Add a set of extra arguments for PANDA
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for arg in args {
            self.arg(arg.into());
        }

        self
    }

    /// Sets the architecture of the guest
    pub fn arch(&mut self, arch: Arch) -> &mut Self {
        self.arch = Some(arch);
        
        self
    }

    // Don't pass `-nographic` to QEMU, allowing you use to use a monitor
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
    
    pub fn generic<S: Into<String>>(&mut self, generic: S) -> &mut Self {
        self.generic_qcow = Some(generic.into());
        
        self
    }

    fn get_args(&self) -> Vec<String> {
        let generic_info =
            self.generic_qcow
                .as_ref()
                .map(|generic| qcows::get_supported_image(generic));

        let qcow_path = self.qcow.clone().unwrap_or_else(||{
            self.generic_qcow
                .as_ref()
                .map(|generic| qcows::get_generic_path(generic).display().to_string())
                .expect("Either a qcow or a generic image must be specified.")
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
            qcow_path,
        ];

        if !self.graphics {
            args.push("-nographic".into());
        }

        args
    }

    /// Start the PANDA instance with the given settings. This is a blocking operation.
    pub fn run(&mut self) {
        let args = self.get_args();

        println!("Running with args: {:?}", args);

        let args: Vec<_> = args.into_iter().map(|x| CString::new(x).unwrap()).collect();

        let args_ptrs: Vec<_> = args.iter().map(|s| s.as_ptr()).collect();

        let x = &mut 0i8;
        let empty = &mut (x as *mut c_char);
        unsafe {
            panda_set_library_mode(true);
            panda_init(args_ptrs.len() as i32, transmute(args_ptrs.as_ptr()), empty);

            for cb in crate::inventory::iter::<crate::Callback> {
                crate::sys::panda_register_callback(self as *mut _ as _, cb.cb_type, ::core::mem::transmute(cb.fn_pointer));
            }

            panda_run();
        }
    }
}
