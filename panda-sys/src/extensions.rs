use std::mem::{size_of, MaybeUninit, transmute};
use crate::{target_ulong, target_ptr_t, panda_physical_memory_read_external, panda_virtual_memory_read_external, panda_virtual_memory_write_external, CPUState};

const READ_CHUNK_SIZE: target_ptr_t = 0x10;

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

    pub fn mem_write(&mut self, addr: target_ulong, data: &[u8]) {
        unsafe {
            if panda_virtual_memory_write_external(self, addr, transmute(data.as_ptr()), data.len() as _) != 0 {
                panic!("Virtual memory write failed");
            }
        }
    }
    
    pub fn try_mem_read(&mut self, addr: target_ulong, len: usize) -> Option<Vec<u8>> {
        let mut temp = vec![0; len];

        let ret = unsafe {
            panda_virtual_memory_read_external(self, addr, temp.as_mut_ptr() as *mut i8, len as _)
        };

        if ret == 0 {
            Some(temp)
        } else {
            None
        }
    }
    
    pub fn try_mem_read_phys(&mut self, addr: target_ptr_t, len: usize) -> Option<Vec<u8>> {
        let mut temp = vec![0; len];

        unsafe {
            if panda_physical_memory_read_external(addr as _, temp.as_mut_ptr(), len as _) == 0 {
                Some(temp)
            } else {
                None
            }
        }
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

    pub fn mem_read_string(&mut self, mut addr: target_ptr_t) -> String {
        let mut buf = vec![];
        let mut temp = [0; READ_CHUNK_SIZE as usize];
        loop {
            unsafe {
                panda_virtual_memory_read_external(self, addr, temp.as_mut_ptr() as *mut i8, READ_CHUNK_SIZE as _);
            }

            let null_index = temp.iter().position(|x| x == &0);
            match null_index {
                Some(index) => {
                    // A null exists in the current chunk
                    buf.extend_from_slice(&temp[0..index]);
                    break
                }
                None => {
                    // No null byte found yet
                    buf.extend_from_slice(&temp);
                    addr += READ_CHUNK_SIZE;
                }
            }
        }

        String::from_utf8_lossy(&buf).into_owned()
    }
}
