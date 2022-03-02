use super::{GuestAlign, GuestPtr, GuestReadFail, GuestWriteFail};
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

                fn read_from_guest(cpu: &mut CPUState, ptr: target_ptr_t) -> Result<Self, GuestReadFail> {
                    let mut bytes = [0u8; core::mem::size_of::<$ty>()];
                    virtual_memory_read_into(cpu, ptr, &mut bytes).or(Err(GuestReadFail))?;

                    Ok(match ARCH_ENDIAN {
                        Endian::Big => <$ty>::from_be_bytes(bytes),
                        Endian::Little => <$ty>::from_le_bytes(bytes),
                    })
                }

                fn read_from_guest_phys(ptr: target_ptr_t) -> Result<Self, GuestReadFail> {
                    let mut bytes = [0u8; core::mem::size_of::<$ty>()];
                    physical_memory_read_into(ptr, &mut bytes).or(Err(GuestReadFail))?;

                    Ok(match ARCH_ENDIAN {
                        Endian::Big => <$ty>::from_be_bytes(bytes),
                        Endian::Little => <$ty>::from_le_bytes(bytes),
                    })
                }

                fn write_to_guest(&self, cpu: &mut CPUState, ptr: target_ptr_t) -> Result<(), GuestWriteFail> {
                    let bytes = match ARCH_ENDIAN {
                        Endian::Big => <$ty>::to_be_bytes(*self),
                        Endian::Little => <$ty>::to_le_bytes(*self),
                    };

                    virtual_memory_write(cpu, ptr, &bytes);

                    Ok(())
                }

                fn write_to_guest_phys(&self, ptr: target_ptr_t) -> Result<(), GuestWriteFail> {
                    let bytes = match ARCH_ENDIAN {
                        Endian::Big => <$ty>::to_be_bytes(*self),
                        Endian::Little => <$ty>::to_le_bytes(*self),
                    };

                    physical_memory_write(ptr, &bytes);

                    Ok(())
                }
            }
        )*
    };
}

impl_for_num!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl<T: GuestType> GuestType for GuestPtr<T> {
    fn guest_layout() -> Option<Layout> {
        target_ptr_t::guest_layout()
    }

    fn read_from_guest(cpu: &mut CPUState, ptr: target_ptr_t) -> Result<Self, GuestReadFail> {
        target_ptr_t::read_from_guest(cpu, ptr).map(Self::from)
    }

    fn write_to_guest(&self, cpu: &mut CPUState, ptr: target_ptr_t) -> Result<(), GuestWriteFail> {
        self.pointer.write_to_guest(cpu, ptr)
    }

    fn read_from_guest_phys(ptr: target_ptr_t) -> Result<Self, GuestReadFail> {
        target_ptr_t::read_from_guest_phys(ptr).map(Self::from)
    }

    fn write_to_guest_phys(&self, ptr: target_ptr_t) -> Result<(), GuestWriteFail> {
        self.pointer.write_to_guest_phys(ptr)
    }
}

fn padding_needed_for(layout: &Layout, align: usize) -> usize {
    let len = layout.size();

    let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
    len_rounded_up.wrapping_sub(len)
}

fn padded_size(layout: &Layout) -> usize {
    layout.size() + padding_needed_for(&layout, layout.align())
}

fn repeat(layout: &Layout, n: usize) -> Layout {
    let alloc_size = padded_size(layout)
        .checked_mul(n)
        .expect("Layout of guest array overflow");

    Layout::from_size_align(alloc_size, layout.align()).expect("Layout of guest array invalid")
}

impl<T: GuestType, const N: usize> GuestType for [T; N] {
    fn guest_layout() -> Option<Layout> {
        T::guest_layout().map(|layout| repeat(&layout, N))
    }

    fn read_from_guest(cpu: &mut CPUState, ptr: target_ptr_t) -> Result<Self, GuestReadFail> {
        let padded_size = padded_size(
            &T::guest_layout().expect("Cannot read array of unsized types from guest."),
        );

        array_init::from_iter(
            (ptr..)
                .step_by(padded_size)
                .take(N)
                .filter_map(|ptr| T::read_from_guest(cpu, ptr).ok()),
        )
        .ok_or(GuestReadFail)
    }

    fn write_to_guest(&self, cpu: &mut CPUState, ptr: target_ptr_t) -> Result<(), GuestWriteFail> {
        let padded_size = padded_size(
            &T::guest_layout().expect("Cannot write array of unsized types to the guest."),
        );

        for (ptr, item) in (ptr..).step_by(padded_size).zip(self.iter()) {
            item.write_to_guest(cpu, ptr)?;
        }

        Ok(())
    }

    fn read_from_guest_phys(ptr: target_ptr_t) -> Result<Self, GuestReadFail> {
        let padded_size = padded_size(
            &T::guest_layout().expect("Cannot read array of unsized types from guest."),
        );

        array_init::from_iter(
            (ptr..)
                .step_by(padded_size)
                .take(N)
                .filter_map(|ptr| T::read_from_guest_phys(ptr).ok()),
        )
        .ok_or(GuestReadFail)
    }

    fn write_to_guest_phys(&self, ptr: target_ptr_t) -> Result<(), GuestWriteFail> {
        let padded_size = padded_size(
            &T::guest_layout().expect("Cannot write array of unsized types to the guest."),
        );

        for (ptr, item) in (ptr..).step_by(padded_size).zip(self.iter()) {
            item.write_to_guest_phys(ptr)?;
        }

        Ok(())
    }
}
