// TODO: cannot live in panda-macros due to arch feature flags?

#[macro_export]
#[cfg(any(feature = "i386", feature = "x86_64"))]
macro_rules! cpu_arch_state {
    ($cpu:expr) => {
        $cpu.env_ptr as *mut panda_sys::CPUX86State
    }
}

#[macro_export]
#[cfg(feature = "arm")]
macro_rules! cpu_arch_state {
    ($cpu:expr) => {
        $cpu.env_ptr as *mut panda_sys::CPUARMState
    }
}

#[macro_export]
#[cfg(any(feature = "mips", feature = "mipsel"))]
macro_rules! cpu_arch_state {
    ($cpu:expr) => {
        $cpu.env_ptr as *mut panda_sys::CPUMIPSState
    }
}

#[macro_export]
#[cfg(feature = "ppc")]
macro_rules! cpu_arch_state {
    ($cpu:expr) => {
        $cpu.env_ptr as *mut panda_sys::CPUPPCState
    }
}