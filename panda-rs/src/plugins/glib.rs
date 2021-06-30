//! glib wrappers for supporting glib-based plugins

use std::mem::size_of;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use glib_sys::{g_malloc, g_free, gpointer, g_array_free, GArray};

/// An owned glib-allocated value that will be freed using glib's allocator on drop.
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

    pub fn as_ptr(&self) -> *const T {
        self.0
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

#[repr(transparent)]
pub struct GBoxedSlice<T>(pub *mut GArray, PhantomData<T>);

impl<T> GBoxedSlice<T> {
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

impl<T> Deref for GBoxedSlice<T> {
    type Target = [T];
    
    fn deref(&self) -> &Self::Target {
        if self.0.is_null() {
            panic!("Invalid GBoxedSlice: null");
        } else {
            let g_array = unsafe { &*self.0 };

            if g_array.data.is_null() {
                &[]
            } else {
                unsafe {
                    std::slice::from_raw_parts(g_array.data as _, g_array.len as usize)
                }
            }
        }
    }
}

impl<T> DerefMut for GBoxedSlice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.0.is_null() {
            panic!("Invalid GBoxedSlice: null");
        }
        let g_array = unsafe { &mut *self.0 };

        if g_array.data.is_null() {
            &mut []
        } else {
            unsafe {
                std::slice::from_raw_parts_mut(g_array.data as *mut T, g_array.len as usize)
            }
        }
    }
}

impl<T> Drop for GBoxedSlice<T> {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                g_array_free(self.0, true as _);
            }
        }
    }
}
