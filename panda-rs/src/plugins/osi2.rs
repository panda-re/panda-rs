//! Bindings and helpers for working with the OSI2 plugin, allowing kernel
//! introspection via Volatility profiles.
use crate::plugin_import;
use crate::prelude::*;

use std::ffi::CString;
use std::os::raw::c_char;

plugin_import! {
    static OSI2: Osi2 = extern "osi2" {
        fn kaslr_offset(cpu: &mut CPUState) -> target_ptr_t;
        fn current_cpu_offset(cpu: &mut CPUState) -> target_ulong;

        fn enum_from_name(name: *const c_char) -> Option<&'static VolatilityEnum>;
        fn base_type_from_name(name: *const c_char) -> Option<&'static VolatilityBaseType>;
        fn symbol_from_name(name: *const c_char) -> Option<&'static VolatilitySymbol>;
        fn type_from_name(name: *const c_char) -> Option<&'static VolatilityStruct>;
        fn symbol_addr_from_name(name: *const c_char) -> target_ptr_t;
        fn addr_of_symbol(symbol: &VolatilitySymbol) -> target_ptr_t;
        fn offset_of_field(
            vol_struct: &VolatilityStruct,
            name: *const c_char
        ) -> target_long;
        fn size_of_struct(vol_struct: &VolatilityStruct) -> target_ulong;
    };
}

// See https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs for
// more info on why this is the way it is.
macro_rules! opaque_types {
    ($($name:ident),*) => {
        $(
            pub struct $name {
                _data: [u8; 0],
                _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
            }
        )*
    };
}

opaque_types!(
    VolatilityEnum,
    VolatilityBaseType,
    VolatilitySymbol,
    VolatilityStruct
);

impl VolatilitySymbol {
    /// Get the address of the given symbol relative to the KASLR offset. Note that
    /// additional calculations may be required afterwards to handle per-CPU structs.
    pub fn addr(&self) -> target_ptr_t {
        OSI2.addr_of_symbol(self)
    }
}

impl VolatilityStruct {
    /// Get the size of the given type in bytes
    pub fn size(&self) -> target_ulong {
        OSI2.size_of_struct(self)
    }

    /// Get the offset of a given field within the structure given the name of the field
    pub fn offset_of(&self, field: &str) -> target_long {
        let field_name = CString::new(field).unwrap();

        OSI2.offset_of_field(self, field_name.as_ptr())
    }
}

/// Get a reference to an opaque object for accessing information about a given enum based
/// on the volatility symbols currently loaded by OSI2
pub fn enum_from_name(name: &str) -> Option<&'static VolatilityEnum> {
    let name = CString::new(name).unwrap();

    OSI2.enum_from_name(name.as_ptr())
}

/// Get a reference to an opaque object for accessing information about a given base type
/// from the volatility symbols currently loaded by OSI2
pub fn base_type_from_name(name: &str) -> Option<&'static VolatilityBaseType> {
    let name = CString::new(name).unwrap();

    OSI2.base_type_from_name(name.as_ptr())
}

/// Get a reference to an opaque object for accessing information about a given symbol
/// present in the volatility symbols currently loaded by OSI2
pub fn symbol_from_name(name: &str) -> Option<&'static VolatilitySymbol> {
    let name = CString::new(name).unwrap();

    OSI2.symbol_from_name(name.as_ptr())
}

/// Get a reference to an opaque object for accessing information about a given type
/// present in the volatility symbols currently loaded by OSI2
pub fn type_from_name(name: &str) -> Option<&'static VolatilityStruct> {
    let name = CString::new(name).unwrap();

    OSI2.type_from_name(name.as_ptr())
}

// Get the symbol of a type relative to the KASLR base offset from the volatility profile
// currently loaded by OSI2. This offset may need additional modification if it points
// to a per-CPU structure.
pub fn symbol_addr_from_name(name: &str) -> target_ptr_t {
    let name = CString::new(name).unwrap();

    OSI2.symbol_addr_from_name(name.as_ptr())
}

/// Get the KASLR offset of the system, calculating and caching it if it has not already
/// been found. For systems without KASLR this will be 0.
pub fn kaslr_offset(cpu: &mut CPUState) -> target_ptr_t {
    OSI2.kaslr_offset(cpu)
}

/// Get the current per-CPU offset for kernel data structures such as the current task
/// struct
pub fn current_cpu_offset(cpu: &mut CPUState) -> target_ulong {
    OSI2.current_cpu_offset(cpu)
}

/// Get the address from a given symbol
pub fn addr_of_symbol(symbol: &VolatilitySymbol) -> target_ptr_t {
    OSI2.addr_of_symbol(symbol)
}

/// Get the offset of a field given the structure it is within and the name of the field
pub fn offset_of_field(vol_struct: &VolatilityStruct, name: &str) -> target_long {
    let name = CString::new(name).unwrap();

    OSI2.offset_of_field(vol_struct, name.as_ptr())
}

/// Get the size of a given structure
pub fn size_of_struct(vol_struct: &VolatilityStruct) -> target_ulong {
    OSI2.size_of_struct(vol_struct)
}
