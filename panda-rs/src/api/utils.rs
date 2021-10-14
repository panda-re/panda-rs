#[cfg(any(feature = "i386", feature = "x86_64"))]
pub type CPUArchPtr = *mut panda_sys::CPUX86State;

#[cfg(any(feature = "arm", feature = "aarch64"))]
pub type CPUArchPtr = *mut panda_sys::CPUARMState;

#[cfg(any(feature = "mips", feature = "mipsel", feature = "mips64"))]
pub type CPUArchPtr = *mut panda_sys::CPUMIPSState;

#[cfg(feature = "ppc")]
pub type CPUArchPtr = *mut panda_sys::CPUPPCState;

#[macro_export]
macro_rules! cpu_arch_state {
    ($cpu:expr) => {
        $cpu.env_ptr as CPUArchPtr
    };
}
