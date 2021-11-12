use super::GuestAlign;
use crate::prelude::*;
use crate::{enums::Endian, mem::*, GuestType, ARCH_ENDIAN};

macro_rules! impl_for_num {
    ($($ty:ty),*) => {
        $(
            impl GuestType for $ty {
                fn guest_size() -> Option<usize> {
                    Some(core::mem::size_of::<$ty>())
                }

                fn guest_align() -> usize {
                    <$ty as GuestAlign>::ALIGN
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

                fn write_to_guest(&self, _cpu: &mut CPUState, _ptr: target_ptr_t) {
                    todo!()
                }

                fn write_to_guest_phys(&self, _ptr: target_ptr_t) {
                    todo!()
                }
            }
        )*
    };
}

impl_for_num!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
