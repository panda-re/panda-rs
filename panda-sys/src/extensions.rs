use std::mem::{size_of, MaybeUninit};
use crate::bindings::*;

impl CPUState {
    pub fn mem_read(&mut self, addr: target_ulong, len: usize) -> Vec<u8> {
        let mut temp = vec![0; len];

        unsafe {
            if panda_virtual_memory_read_external(self, addr, temp.as_mut_ptr() as *mut i8, len as _) != 0 {
                panic!("Virtual memory read failed");
            }
        }

        temp
    }
    
    pub fn mem_read_val<T: Sized>(&mut self, addr: target_ulong) -> T {
        let mut temp = MaybeUninit::uninit();

        unsafe {
            if panda_virtual_memory_read_external(self, addr, temp.as_mut_ptr() as *mut i8, size_of::<T>() as _) != 0 {
                panic!("Virtual memory read failed");
            }

            temp.assume_init()
        }
    }
}
