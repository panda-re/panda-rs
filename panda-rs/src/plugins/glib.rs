use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use glib_sys::{g_malloc, g_free, gpointer};

#[repr(transparent)]
pub struct GBox<T>(*mut T);

impl<T: Sized> GBox<T> {
    pub fn new(val: T) -> Self {
        unsafe {
            let ptr = g_malloc(size_of::<T>());
            *(ptr as *mut T) = val;
            Self(ptr as *mut T)
        }
    }
}

impl<T> Deref for GBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> DerefMut for GBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl<T> Drop for GBox<T> {
    fn drop(&mut self) {
        unsafe {
            g_free(self.0 as gpointer);
        }
    }
}
