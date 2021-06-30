//! Bingings for the OSI (Operating System Introspection) plugin
use crate::sys::{target_ptr_t, target_pid_t, target_ulong, CPUState};
use crate::plugins::glib::{GBox, GBoxedSlice};
use crate::plugin_import;

use std::ffi::CStr;
use std::borrow::Cow;

use glib_sys::GArray;

plugin_import!{
    static OSI: Osi = extern "osi" {
        fn get_process_handles(cpu: *mut CPUState) -> GBoxedSlice<OsiProcHandle>;
        fn get_current_thread(cpu: *mut CPUState) -> GBox<OsiThread>;
        fn get_modules(cpu: *mut CPUState) -> GBoxedSlice<OsiModule>;
        fn get_mappings(cpu: *mut CPUState, p: *mut OsiProc) -> GBoxedSlice<OsiModule>;
        fn get_processes(cpu: *mut CPUState) -> GBoxedSlice<OsiProc>;
        fn get_current_process(cpu: *mut CPUState) -> GBox<OsiProc>;
        fn get_one_module(osimodules: *mut GArray, idx: ::std::os::raw::c_uint) -> *mut OsiModule;
        fn get_one_proc(osiprocs: *mut GArray, idx: ::std::os::raw::c_uint) -> *mut OsiProc;
        fn cleanup_garray(g: *mut GArray);
        fn get_current_process_handle(cpu: *mut CPUState) -> GBox<OsiProcHandle>;
        fn get_process(cpu: *mut CPUState, h: *const OsiProcHandle) -> GBox<OsiProc>;
        fn get_process_pid(cpu: *mut CPUState, h: *const OsiProcHandle) -> target_pid_t;
        fn get_process_ppid(cpu: *mut CPUState, h: *const OsiProcHandle) -> target_pid_t;
        fn in_shared_object(cpu: *mut CPUState, h: *const OsiProc) -> bool;
    };
}

#[doc = " Minimal handle for a process. Contains a unique identifier \\p asid"]
#[doc = " and a task descriptor pointer \\p taskd that can be used to retrieve the full"]
#[doc = " details of the process."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct osi_proc_handle_struct {
    pub taskd: target_ptr_t,
    pub asid: target_ptr_t,
}
#[doc = " Minimal handle for a process. Contains a unique identifier \\p asid"]
#[doc = " and a task descriptor pointer \\p taskd that can be used to retrieve the full"]
#[doc = " details of the process."]
pub type OsiProcHandle = osi_proc_handle_struct;
#[doc = " Minimal information about a process thread."]
#[doc = " Address space and open resources are shared between threads"]
#[doc = " of the same process. This information is stored in OsiProc."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct osi_thread_struct {
    pub pid: target_pid_t,
    pub tid: target_pid_t,
}
#[doc = " Minimal information about a process thread."]
#[doc = " Address space and open resources are shared between threads"]
#[doc = " of the same process. This information is stored in OsiProc."]
pub type OsiThread = osi_thread_struct;
#[doc = " Represents a page in the address space of a process."]
#[doc = ""]
#[doc = " This has not been implemented/used so far."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct osi_page_struct {
    pub start: target_ptr_t,
    pub len: target_ulong,
}
#[doc = " Represents a page in the address space of a process."]
#[doc = ""]
#[doc = " This has not been implemented/used so far."]
pub type OsiPage = osi_page_struct;
#[doc = " Represents information about a guest OS module (kernel module"]
#[doc = " or shared library)."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct osi_module_struct {
    pub modd: target_ptr_t,
    pub base: target_ptr_t,
    pub size: target_ptr_t,
    pub file: *mut ::std::os::raw::c_char,
    pub name: *mut ::std::os::raw::c_char,
}
#[doc = " Represents information about a guest OS module (kernel module"]
#[doc = " or shared library)."]
pub type OsiModule = osi_module_struct;
#[doc = " Detailed information for a process."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct osi_proc_struct {
    pub taskd: target_ptr_t,
    pub asid: target_ptr_t,
    pub pid: target_pid_t,
    pub ppid: target_pid_t,
    pub name: *mut ::std::os::raw::c_char,
    pub pages: *mut OsiPage,
    pub create_time: u64,
}
#[doc = " Detailed information for a process."]
pub type OsiProc = osi_proc_struct;

impl osi_proc_struct {
    pub fn get_name(&self) -> Cow<str> {
        unsafe { CStr::from_ptr(self.name) }.to_string_lossy()
    }
}
