#[cfg(any(feature = "i386", feature = "x86_64"))]
pub type CPUArchPtr = *mut panda_sys::CPUX86State;

#[cfg(feature = "arm")]
pub type CPUArchPtr = *mut panda_sys::CPUARMState;

#[cfg(any(feature = "mips", feature = "mipsel"))]
pub type CPUArchPtr = *mut panda_sys::CPUMIPSState;

#[cfg(feature = "ppc")]
pub type CPUArchPtr = *mut panda_sys::CPUPPCState;

#[macro_export]
macro_rules! cpu_arch_state {
    ($cpu:expr) => {
        $cpu.env_ptr as CPUArchPtr
    }
}