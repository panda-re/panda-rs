use super::GuestAlign;
use crate::prelude::*;
use crate::{enums::Endian, mem::*, GuestType, ARCH_ENDIAN};

use std::alloc::Layout;

macro_rules! impl_for_num {
    ($($ty:ty),*) => {
        $(
            impl GuestType for $ty {
                fn guest_layout() -> Option<Layout> {
                    Layout::from_size_align(
                        core::mem::size_of::<$ty>(),
                        <$ty as GuestAlign>::ALIGN
                    ).ok()
                }

                fn read_from_guest(cpu: &mut CPUState, ptr: target_ptr_t) -> Self {
                    let mut bytes = [0u8; core::mem::size_of::<$ty>()];
                    virtual_memory_read_into(cpu, ptr, &mut bytes)
                        .expect("Virtual memory read for GuestType failed.");

                    match ARCH_ENDIAN {
                        Endian::Big => <$ty>::from_be_bytes(bytes),
                        Endian::Little => <$ty>::from_le_bytes(bytes),
                    }
                }

                fn read_from_guest_phys(ptr: target_ptr_t) -> Self {
                    let mut bytes = [0u8; core::mem::size_of::<$ty>()];
                    physical_memory_read_into(ptr, &mut bytes)
                        .expect("Physical memory read for GuestType failed.");

                    match ARCH_ENDIAN {
                        Endian::Big => <$ty>::from_be_bytes(bytes),
                        Endian::Little => <$ty>::from_le_bytes(bytes),
                    }
                }

                fn write_to_guest(&self, cpu: &mut CPUState, ptr: target_ptr_t) {
                    let bytes = match ARCH_ENDIAN {
                        Endian::Big => <$ty>::to_be_bytes(*self),
                        Endian::Little => <$ty>::to_le_bytes(*self),
                    };

                    virtual_memory_write(cpu, ptr, &bytes);
                }

                fn write_to_guest_phys(&self, ptr: target_ptr_t) {
                    let bytes = match ARCH_ENDIAN {
                        Endian::Big => <$ty>::to_be_bytes(*self),
                        Endian::Little => <$ty>::to_le_bytes(*self),
                    };

                    physical_memory_write(ptr, &bytes);
                }
            }
        )*
    };
}

impl_for_num!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
