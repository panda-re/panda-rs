use crate::prelude::*;
use once_cell::sync::OnceCell;

use std::alloc::Layout;
use std::ops::Deref;

mod guest_align;
mod impls;

pub(crate) use guest_align::GuestAlign;

#[derive(Copy, Clone, Debug)]
pub struct GuestReadFail;

#[derive(Copy, Clone, Debug)]
pub struct GuestWriteFail;

/// A type which can be converted to and from a guest memory representation, allowing
/// it to be used with [`GuestPtr`].
pub trait GuestType: Sized {
    fn guest_layout() -> Option<Layout>;

    /// The size of the type in the guest, `None` if the type is dynamically sized
    fn guest_size() -> Option<usize> {
        Self::guest_layout().map(|layout| layout.size())
    }

    /// The required minimum alignment of the type in the guest
    fn guest_align() -> usize {
        Self::guest_layout()
            .map(|layout| layout.align())
            .unwrap_or(1)
    }

    fn read_from_guest(cpu: &mut CPUState, ptr: target_ptr_t) -> Result<Self, GuestReadFail>;
    fn write_to_guest(&self, cpu: &mut CPUState, ptr: target_ptr_t) -> Result<(), GuestWriteFail>;

    fn read_from_guest_phys(ptr: target_ptr_t) -> Result<Self, GuestReadFail>;
    fn write_to_guest_phys(&self, ptr: target_ptr_t) -> Result<(), GuestWriteFail>;
}

pub struct GuestPtr<T: GuestType> {
    pointer: target_ptr_t,
    guest_type: OnceCell<Box<T>>,
}

impl<T: GuestType> From<target_ptr_t> for GuestPtr<T> {
    fn from(pointer: target_ptr_t) -> Self {
        GuestPtr {
            pointer,
            guest_type: OnceCell::new(),
        }
    }
}

impl<T: GuestType> Clone for GuestPtr<T> {
    fn clone(&self) -> Self {
        Self::from(self.pointer)
    }
}

impl<T: GuestType> GuestPtr<T> {
    /// Reads the value from the guest to be accessed later. This is a no-op if a value
    /// has already been cached. This is only needed if you need to read at a different
    /// time than you intend to.
    ///
    /// If you want read a value and replace the cache if it exists, use
    /// [`GuestPtr::update`] instead. If you wish to read at time of first access,
    /// the `GuestPtr` only needs to be dereferenced without calling `read` ahead of
    /// time.
    pub fn read(&self) -> Result<&T, GuestReadFail> {
        let cpu = unsafe { &mut *crate::sys::get_cpu() };

        self.guest_type
            .get_or_try_init(|| T::read_from_guest(cpu, self.pointer).map(Box::new))
            .map(|x| &**x) // &Box<T> -> &T
    }

    /// Reads the value from the guest, replacing it if any exists.
    pub fn update(&mut self) {
        self.clear_cache();
        self.read().unwrap();
    }

    /// Clear the cached value, if any exists.
    pub fn clear_cache(&mut self) {
        self.guest_type = OnceCell::new();
    }

    /// Returns a reference to the cached value if one exists.
    pub fn get_cached(&self) -> Option<&T> {
        self.guest_type.get().map(Box::as_ref)
    }

    /// Creates a copy of the pointer offset by N items.
    ///
    /// **Note:** Similar to normal pointer arithmatic the actual value of the offset
    /// will be multiplied by the size of the object.
    pub fn offset(&self, off: usize) -> Self {
        let size =
            T::guest_size().expect("Attempted to offset an unsized GuestType") as target_ptr_t;
        GuestPtr {
            pointer: self.pointer + (size * (off as target_ptr_t)),
            guest_type: OnceCell::new(),
        }
    }

    /// Creates a copy of the pointer offset by N bytes.
    pub fn offset_bytes(&self, bytes: usize) -> Self {
        GuestPtr {
            pointer: self.pointer + (bytes as target_ptr_t),
            guest_type: OnceCell::new(),
        }
    }

    /// Casts the GuestPtr to another type of GuestPtr
    pub fn cast<U: GuestType>(&self) -> GuestPtr<U> {
        GuestPtr {
            pointer: self.pointer,
            guest_type: OnceCell::new(),
        }
    }

    /// Write to the GuestPtr, with all modifications flushed at the end of the scope of
    /// the function provided to `write`.
    pub fn write(&mut self, func: impl FnOnce(&mut T)) -> Result<(), GuestWriteFail> {
        if self.guest_type.get().is_none() {
            self.read().unwrap();
        }

        let mut inner = self.guest_type.get_mut();
        let inner = inner.as_mut().unwrap();

        func(inner);

        let cpu = unsafe { &mut *crate::sys::get_cpu() };
        inner.write_to_guest(cpu, self.pointer)
    }
}

impl<T: GuestType> Deref for GuestPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.read().unwrap();
        self.get_cached()
            .expect("Failed to read cached value from GuestPtr")
    }
}
