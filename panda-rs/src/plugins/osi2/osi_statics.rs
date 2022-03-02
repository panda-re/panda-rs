use std::ops::Deref;

use crate::guest_ptr::GuestReadFail;
use crate::prelude::*;
use crate::GuestType;

use super::{find_per_cpu_address, kaslr_offset, symbol_addr_from_name};

/// A trait representing that a type is readable using OSI 2.
///
/// See the [`OsiType`](macro@panda::plugins::osi2::OsiType) derive macro for more details.
pub trait OsiType: Sized {
    type MethodDispatcher;

    /// Read the given type out of memory starting at `base_ptr`
    fn osi_read(cpu: &mut CPUState, base_ptr: target_ptr_t) -> Result<Self, GuestReadFail>;
}

#[doc(hidden)]
pub struct EmptyMethodDelegator(&'static str, bool);

impl EmptyMethodDelegator {
    pub const fn new(_: &'static str, _: bool) -> Self {
        Self("", false)
    }
}

/// Types with a fixed layout (primarily primitives) implement OsiType automatically,
/// allowing them to be used with [`osi_static`](super::osi_static) seamlessly.
impl<T: GuestType> OsiType for T {
    type MethodDispatcher = EmptyMethodDelegator;

    fn osi_read(cpu: &mut CPUState, base_ptr: target_ptr_t) -> Result<Self, GuestReadFail> {
        T::read_from_guest(cpu, base_ptr)
    }
}

/// A type used internally by [`osi_static`](panda::plugins::osi2::osi_static) in order
/// to provide a value that can be read wholesale or one field at a time.
#[doc(hidden)]
pub struct PerCpu<T: OsiType>(pub &'static str, pub T::MethodDispatcher);

impl<T: OsiType> PerCpu<T> {
    pub fn read(&self, cpu: &mut CPUState) -> Result<T, GuestReadFail> {
        let ptr = find_per_cpu_address(cpu, self.0)?;

        T::osi_read(cpu, ptr)
    }
}

impl<T: OsiType> Deref for PerCpu<T> {
    type Target = T::MethodDispatcher;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

/// A type used internally by [`osi_static`](panda::plugins::osi2::osi_static) in order
/// to provide a value that can be read wholesale or one field at a time.
#[doc(hidden)]
pub struct OsiGlobal<T: OsiType>(pub &'static str, pub T::MethodDispatcher);

impl<T: OsiType> OsiGlobal<T> {
    pub fn read(&self, cpu: &mut CPUState) -> Result<T, GuestReadFail> {
        let symbol_offset = symbol_addr_from_name(self.0);
        let ptr = kaslr_offset(cpu) + symbol_offset;

        T::osi_read(cpu, ptr)
    }
}

impl<T: OsiType> Deref for OsiGlobal<T> {
    type Target = T::MethodDispatcher;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
