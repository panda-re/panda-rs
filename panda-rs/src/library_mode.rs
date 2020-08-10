/// Architecture of the guest system
#[allow(non_camel_case_types)]
pub enum Arch {
    i386,
    x86_64,
    Arm,
    Mips,
}

impl Default for Arch {
    fn default() -> Self {
        Self::i386
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
    os: String,
    mem: String,
    arch: Arch,
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
            mem: "128M".into(),
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
        self.arch = arch;
        
        self
    }
    
    /// When set, don't specify a `-monitor`. This argument allows for use of `-nographc` in args
    /// with `ctrl-a+c` for interactive QEMU prompt.
    pub fn raw_monitor(&mut self, x: bool) -> &mut Self {
        self.raw_monitor = x;
        
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
        self.mem = mem.into();

        self
    }

    /// Start the PANDA instance with the given settings. This is a blocking operation.
    pub fn run(&mut self) {
        todo!()
    }
}
