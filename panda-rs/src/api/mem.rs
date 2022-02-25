use crate::enums::MemRWStatus;
use crate::prelude::*;
use crate::GuestType;
use crate::{sys, Error};
use crate::{GuestReadFail, GuestWriteFail};

use std::ffi::CString;
use std::os::raw::c_char;

// Public API ----------------------------------------------------------------------------------------------------------

/// Read a structure or value from guest memory using the guest endianess and
/// layout for the given type.
///
/// ## Example
///
/// ```
/// use panda::mem::read_guest_type;
/// use panda::prelude::*;
///
/// # let cpu: &mut CPUState = todo!();
/// let pid: u32 = read_guest_type(cpu, 0x55550000).unwrap();
/// ```
///
/// To use custom structures and types, derive the [`GuestType`] trait for your
/// given structure.
pub fn read_guest_type<T: GuestType>(
    cpu: &mut CPUState,
    addr: target_ptr_t,
) -> Result<T, GuestReadFail> {
    T::read_from_guest(cpu, addr)
}

/// Write a given type to guest memory using the guest endianess and layout for the
/// given type.
///
/// ## Example
///
/// ```
/// use panda::mem::write_guest_type;
/// use panda::prelude::*;
///
/// # let cpu: &mut CPUState = todo!();
/// let pid = 1234_u32;
///
/// write_guest_type(cpu, 0x55550000, pid);
/// ```
pub fn write_guest_type<T: GuestType>(
    cpu: &mut CPUState,
    addr: target_ptr_t,
    val: &T,
) -> Result<(), GuestWriteFail> {
    val.write_to_guest(cpu, addr)
}

/// Read a structure or value from physical guest memory using the guest endianess and layout
/// for the given type.
///
/// ## Example
///
/// ```
/// use panda::mem::read_guest_type_phys;
/// use panda::prelude::*;
///
/// # let cpu: &mut CPUState = todo!();
/// let ptr = 0xF8000010;
/// let pid: u32 = read_guest_type_phys(ptr).unwrap();
/// ```
pub fn read_guest_type_phys<T: GuestType>(addr: target_ptr_t) -> Result<T, GuestReadFail> {
    T::read_from_guest_phys(addr)
}

/// Write a given type to guest physical memory using the guest endianess and layout for the
/// given type.
///
/// ## Example
///
/// ```
/// use panda::mem::write_guest_type_phys;
/// use panda::prelude::*;
///
/// # let cpu: &mut CPUState = todo!();
/// let pid = 1234_u32;
///
/// write_guest_type_phys(0xF8000010, pid);
/// ```
pub fn write_guest_type_phys<T: GuestType>(
    addr: target_ptr_t,
    val: &T,
) -> Result<(), GuestWriteFail> {
    val.write_to_guest_phys(addr)
}

/// Read from guest virtual memory
pub fn virtual_memory_read(
    cpu: &mut CPUState,
    addr: target_ulong,
    len: usize,
) -> Result<Vec<u8>, MemRWStatus> {
    let mut buf: Vec<c_char> = Vec::with_capacity(len);

    unsafe {
        let res =
            panda_sys::panda_virtual_memory_read_external(cpu, addr, buf.as_mut_ptr(), len as i32)
                .into();

        match res {
            MemRWStatus::MemTxOk => {
                buf.set_len(len);
                Ok(vec_i8_into_u8(buf))
            }
            _ => Err(res),
        }
    }
}

/// Read from guest virtual memory into a buffer
pub fn virtual_memory_read_into(
    cpu: &mut CPUState,
    addr: target_ulong,
    buf: &mut [u8],
) -> Result<(), MemRWStatus> {
    let res = unsafe {
        panda_sys::panda_virtual_memory_read_external(
            cpu,
            addr,
            buf.as_mut_ptr() as _,
            buf.len() as i32,
        )
        .into()
    };

    match res {
        MemRWStatus::MemTxOk => Ok(()),
        _ => Err(res),
    }
}

/// Read from guest physical memory
pub fn physical_memory_read(addr: target_ulong, len: usize) -> Result<Vec<u8>, MemRWStatus> {
    let mut buf: Vec<u8> = Vec::with_capacity(len);

    unsafe {
        let res = panda_sys::panda_physical_memory_read_external(
            addr as u64,
            buf.as_mut_ptr(),
            len as i32,
        )
        .into();

        match res {
            MemRWStatus::MemTxOk => {
                buf.set_len(len);
                Ok(buf)
            }
            _ => Err(res),
        }
    }
}

/// Read from guest physical memory into a pre-allocated buffer
pub fn physical_memory_read_into(addr: target_ulong, buf: &mut [u8]) -> Result<(), MemRWStatus> {
    let res = unsafe {
        panda_sys::panda_physical_memory_read_external(
            addr as u64,
            buf.as_mut_ptr() as _,
            buf.len() as i32,
        )
        .into()
    };

    match res {
        MemRWStatus::MemTxOk => Ok(()),
        _ => Err(res),
    }
}

/// Write to guest virtual memory
pub fn virtual_memory_write(cpu: &mut CPUState, addr: target_ulong, data: &[u8]) -> MemRWStatus {
    let mut c_data = data.to_vec(); // Alloc b/c C API wants mut
    unsafe {
        panda_sys::panda_virtual_memory_write_external(
            cpu,
            addr,
            c_data.as_mut_ptr() as *mut i8,
            c_data.len() as i32,
        )
        .into()
    }
}

/// Write to guest physical memory
pub fn physical_memory_write(addr: target_ulong, data: &[u8]) -> MemRWStatus {
    let mut c_data = data.to_vec(); // Alloc b/c C API wants mut
    unsafe {
        panda_sys::panda_physical_memory_write_external(
            addr as _,
            c_data.as_mut_ptr(),
            c_data.len() as i32,
        )
        .into()
    }
}

/// Translate guest virtual address to physical address, returning `None` if no mapping
/// can be found.
pub fn virt_to_phys(cpu: &mut CPUState, addr: target_ulong) -> Option<target_ulong> {
    match unsafe { panda_sys::panda_virt_to_phys_external(cpu, addr) } {
        target_ulong::MAX => None,
        phys_addr => Some(phys_addr),
    }
}

pub const PAGE_SIZE: target_ulong = 1024;

/// Map RAM into the system at a given physical address
pub fn map_memory(name: &str, size: target_ulong, addr: target_ptr_t) -> Result<(), Error> {
    let name = CString::new(name)?;

    if size % PAGE_SIZE != 0 {
        Err(Error::UnalignedPageSize)
    } else {
        unsafe {
            sys::map_memory(name.as_ptr() as _, size as _, addr as _);
        }

        drop(name);

        Ok(())
    }
}

const IS_32_BIT: bool = std::mem::size_of::<target_ptr_t>() == 4;
const TARGET_BITS: usize = std::mem::size_of::<target_ptr_t>() * 8;

fn hex_addr(addr: target_ptr_t) -> impl std::fmt::Display {
    if IS_32_BIT {
        format!("{:08x}", addr)
    } else {
        format!("{:016x}", addr)
    }
}

pub fn virt_memory_dump(cpu: &mut CPUState, addr: target_ptr_t, len: usize) {
    let memory = virtual_memory_read(cpu, addr, len).unwrap();

    let start_addr_aligned = addr & !0xf;
    let end_addr_aligned = (addr + (len as target_ptr_t)) & !0xf;

    let bytes_offset = addr - start_addr_aligned;

    let hex_dump = (start_addr_aligned..=end_addr_aligned)
        .step_by(0x10)
        .enumerate()
        .map(|(line_num, line_addr)| {
            let hex_data = (0..0x10)
                .map(|offset_in_line| {
                    if line_num == 0 && offset_in_line < (bytes_offset as usize) {
                        "  ".into()
                    } else {
                        let byte_index =
                            ((line_num * 0x10) + offset_in_line) - (bytes_offset as usize);

                        if let Some(byte) = memory.get(byte_index) {
                            format!("{:02x}", byte).into()
                        } else {
                            "  ".into()
                        }
                    }
                })
                .collect::<Vec<std::borrow::Cow<'static, str>>>()
                .join(" ");

            format!("{}║{}\n", hex_addr(line_addr), hex_data)
        })
        .collect::<String>();

    println!(
        "{} 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f",
        " ".repeat(TARGET_BITS / 4)
    );
    println!(
        "{}╦═══════════════════════════════════════════════",
        "═".repeat(TARGET_BITS / 4)
    );
    println!("{}", hex_dump);
}

// Private API ---------------------------------------------------------------------------------------------------------

// https://stackoverflow.com/questions/59707349/cast-vector-of-i8-to-vector-of-u8-in-rust/59707887#59707887
// TODO: replace with https://doc.rust-lang.org/std/vec/struct.Vec.html#method.into_raw_parts, once on stable
fn vec_i8_into_u8(v: Vec<i8>) -> Vec<u8> {
    // Make sure v's destructor doesn't free the data it thinks it owns when it goes out of scope
    let mut v = std::mem::ManuallyDrop::new(v);

    // Pick apart the existing Vec
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    // Adopt the data into a new Vec
    unsafe { Vec::from_raw_parts(p as *mut u8, len, cap) }
}
