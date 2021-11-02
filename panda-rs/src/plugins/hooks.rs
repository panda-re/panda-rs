//! Bindings for the PANDA 'hooks' plugin, enabling the ability
//! to add callbacks for when a certain instruction is hit.
//!
//! Recommended usage is via either the [`#[panda::hook]`](macro@crate::hook) macro or
//! the [`hook`](mod@crate::hook) module.
//!
//! ## Example
//!
//! ```
//! use panda::plugins::proc_start_linux::AuxvValues;
//! use panda::plugins::hooks::Hook;
//! use panda::prelude::*;
//!
//! #[panda::hook]
//! fn entry_hook(_: &mut CPUState, _: &mut TranslationBlock, _: u8, hook: &mut Hook) {
//!     println!("\n\nHit entry hook!\n");
//!
//!     // only run hook once
//!     hook.enabled = false;
//! }
//!
//! #[panda::on_rec_auxv]
//! fn on_proc_start(_: &mut CPUState, _: &mut TranslationBlock, auxv: &AuxvValues) {
//!     // when a process starts, hook the entrypoint
//!     entry_hook::hook()
//!         .after_block_exec()
//!         .at_addr(auxv.entry)
//! }
//!
//! Panda::new()
//!     .generic("x86_64")
//!     .replay("test")
//!     .run();
//! ```
use std::ffi::c_void;

use crate::plugin_import;
use crate::prelude::*;
use crate::sys::{self, panda_cb_type};

plugin_import! {
    static HOOKS: Hooks = extern "hooks" {
        fn add_hook(hook: &Hook);
        fn enable_hooking();
        fn disable_hooking();
        fn add_symbol_hook(hook: &SymbolHook);
    };
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Symbol {
    pub address: target_ulong,
    pub name: [u8; 256usize],
    pub section: [u8; 256usize],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HooksPandaCallback(panda_cb_type, *const ());

type NormalHookType = extern "C" fn(env: &mut CPUState, tb: &mut TranslationBlock, hook: &mut Hook);
type BeforeTranslateHook = extern "C" fn(env: &mut CPUState, pc: target_ptr_t, hook: &mut Hook);
type AfterBlockHook =
    extern "C" fn(env: &mut CPUState, tb: &mut TranslationBlock, exitCode: u8, hook: &mut Hook);
type InvalidateOpHook =
    extern "C" fn(env: &mut CPUState, tb: &mut TranslationBlock, hook: &mut Hook) -> bool;

impl HooksPandaCallback {
    pub fn from_before_tcg_codegen(cb: NormalHookType) -> Self {
        Self(sys::panda_cb_type_PANDA_CB_BEFORE_TCG_CODEGEN, cb as _)
    }

    pub fn from_after_block_translate(cb: NormalHookType) -> Self {
        Self(sys::panda_cb_type_PANDA_CB_AFTER_BLOCK_TRANSLATE, cb as _)
    }

    pub fn from_before_block_exec(cb: NormalHookType) -> Self {
        Self(sys::panda_cb_type_PANDA_CB_BEFORE_BLOCK_EXEC, cb as _)
    }

    pub fn from_start_block_exec(cb: NormalHookType) -> Self {
        Self(sys::panda_cb_type_PANDA_CB_START_BLOCK_EXEC, cb as _)
    }

    pub fn from_end_block_exec(cb: NormalHookType) -> Self {
        Self(sys::panda_cb_type_PANDA_CB_END_BLOCK_EXEC, cb as _)
    }

    pub fn from_before_block_exec_invalidate_opt(cb: InvalidateOpHook) -> Self {
        Self(
            sys::panda_cb_type_PANDA_CB_BEFORE_BLOCK_EXEC_INVALIDATE_OPT,
            cb as _,
        )
    }

    pub fn from_before_block_translate(cb: BeforeTranslateHook) -> Self {
        Self(sys::panda_cb_type_PANDA_CB_BEFORE_BLOCK_TRANSLATE, cb as _)
    }

    pub fn from_after_block_exec(cb: AfterBlockHook) -> Self {
        Self(sys::panda_cb_type_PANDA_CB_AFTER_BLOCK_EXEC, cb as _)
    }
}

/// A set of functions for building hooks out of closures.
///
/// ## Example
///
/// ```
/// use panda::{hook, prelude::*};
///
/// hook::before_block_exec(|_, _, hook| {
///     println!("hook hit!");
///     hook.enabled = false;
/// })
/// .at_addr(0x5555500ca);
/// ```
///
/// For free functions, it may be easier to use [`#[panda::hook]`](macro@crate::hook)
pub mod hook {
    use super::*;

    macro_rules! define_hook_builders {
        ($(
            fn $name:ident ( $($arg:ident : $arg_ty:ty ),* ) $(-> $ret_ty:ty)?;
        )*) => {
            $(
                pub fn $name<CallbackFn>(
                    callback: CallbackFn
                ) -> HookBuilder<extern "C" fn(
                    $( $arg : $arg_ty, )*
                    hook: &mut Hook,
                ) $( -> $ret_ty )?>
                    where CallbackFn: FnMut($($arg_ty,)* &mut Hook) $( -> $ret_ty )? + 'static,
                {
                    extern "C" fn trampoline(
                        $( $arg : $arg_ty, )*
                        hook: &mut Hook,
                    ) $( -> $ret_ty )? {
                        let callback: &mut &mut dyn FnMut(
                            $($arg_ty,)*
                            &mut Hook,
                        ) $( -> $ret_ty )? = unsafe {
                            std::mem::transmute(hook.context)
                        };

                        callback($($arg, )* hook)
                    }

                    let cb: &mut &mut dyn FnMut(
                        $($arg_ty,)* &mut Hook
                    ) $( -> $ret_ty )? = Box::leak(Box::new(
                        Box::leak(Box::new(callback) as _)
                    ));

                    $crate::paste::paste! {
                        HookBuilder {
                            hook: trampoline,
                            callback: HooksPandaCallback::[< from_ $name  >](trampoline),
                            only_kernel: None,
                            enabled: true,
                            asid: None,
                            context: cb as *mut _ as *mut _,
                        }
                    }
                }
            )*
        };
    }

    define_hook_builders! {
        fn before_block_exec(env: &mut CPUState, tb: &mut TranslationBlock);
        fn before_tcg_codegen(env: &mut CPUState, tb: &mut TranslationBlock);
        fn after_block_translate(env: &mut CPUState, tb: &mut TranslationBlock);
        fn start_block_exec(env: &mut CPUState, tb: &mut TranslationBlock);
        fn end_block_exec(env: &mut CPUState, tb: &mut TranslationBlock);

        fn after_block_exec(env: &mut CPUState, tb: &mut TranslationBlock, exit_code: u8);
        fn before_block_translate(env: &mut CPUState, pc: target_ptr_t);
        fn before_block_exec_invalidate_opt(env: &mut CPUState, tb: &mut TranslationBlock) -> bool;
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum KernelMode {
    Any = 0,
    KernelOnly = 1,
    UserOnly = 2,
}

/// A hook provided by the hooks plugin, describing the address,
/// asid/process, symbol, etc to hook.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Hook {
    /// The address to hook
    pub addr: target_ulong,

    /// The address space identifier to hook, with zero representing any asid.
    /// Defaults to zero.
    pub asid: target_ulong,

    /// The callback to trigger when the hook is hit
    pub cb: HooksPandaCallback,

    /// Whether to hook in kernel mode only, user mode only, or neither.
    /// Defaults to neither.
    pub km: KernelMode,

    /// Whether the hook is enabled. Defaults to `true`
    pub enabled: bool,

    /// The symbol of the function to hook
    pub sym: Symbol,

    /// User-provided context variable
    pub context: *mut c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SymbolHook {
    pub name: [u8; 256usize],
    pub offset: target_ulong,
    pub hook_offset: bool,
    pub section: [u8; 256usize],
    pub cb: HooksPandaCallback,
}

pub trait IntoHookBuilder {
    type BuilderType;

    fn hook(self) -> Self::BuilderType;
}

impl IntoHookBuilder for NormalHookType {
    type BuilderType = HookBuilder<NormalHookType>;

    fn hook(self) -> Self::BuilderType {
        HookBuilder {
            hook: self,
            callback: HooksPandaCallback::from_start_block_exec(self),
            only_kernel: None,
            enabled: true,
            asid: None,
            context: std::ptr::null_mut(),
        }
    }
}

impl IntoHookBuilder for BeforeTranslateHook {
    type BuilderType = HookBuilderCallbackTypeNeeded<Self>;

    fn hook(self) -> Self::BuilderType {
        HookBuilderCallbackTypeNeeded(self)
    }
}

impl IntoHookBuilder for AfterBlockHook {
    type BuilderType = HookBuilderCallbackTypeNeeded<Self>;

    fn hook(self) -> Self::BuilderType {
        HookBuilderCallbackTypeNeeded(self)
    }
}

impl IntoHookBuilder for InvalidateOpHook {
    type BuilderType = HookBuilderCallbackTypeNeeded<Self>;

    fn hook(self) -> Self::BuilderType {
        HookBuilderCallbackTypeNeeded(self)
    }
}

/// A builder type for helping construct and install a [`Hook`].
pub struct HookBuilder<T> {
    hook: T,
    callback: HooksPandaCallback,
    only_kernel: Option<bool>,
    enabled: bool,
    asid: Option<target_ulong>,
    context: *mut c_void,
}

impl<T> HookBuilder<T> {
    /// Sets if kernel mode should be used. `true` for kernel-only hooking, `false` for user-only
    /// hooking. By default the generated hook will hook either.
    pub fn kernel(mut self, only_kernel: bool) -> Self {
        self.only_kernel = Some(only_kernel);
        self
    }

    /// Sets if the hook is enabled. Defaults to `true`.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the asid to hook. Defaults to any.
    pub fn asid(mut self, asid: target_ulong) -> Self {
        self.asid = Some(asid);
        self
    }

    /// Installs the hook at a given address
    pub fn at_addr(self, addr: target_ulong) {
        HOOKS.add_hook(&Hook {
            addr,
            asid: self.asid.unwrap_or(0),
            enabled: self.enabled,
            km: match self.only_kernel {
                Some(true) => KernelMode::KernelOnly,
                Some(false) => KernelMode::UserOnly,
                None => KernelMode::Any,
            },
            cb: self.callback,
            sym: unsafe { std::mem::zeroed() },
            context: self.context,
        });
    }
}

impl HookBuilder<NormalHookType> {
    pub fn before_tcg_codegen(mut self) -> Self {
        self.callback = HooksPandaCallback::from_before_tcg_codegen(self.hook);
        self
    }

    pub fn after_block_translate(mut self) -> Self {
        self.callback = HooksPandaCallback::from_after_block_translate(self.hook);
        self
    }

    pub fn before_block_exec(mut self) -> Self {
        self.callback = HooksPandaCallback::from_before_block_exec(self.hook);
        self
    }

    pub fn start_block_exec(mut self) -> Self {
        self.callback = HooksPandaCallback::from_start_block_exec(self.hook);
        self
    }

    pub fn end_block_exec(mut self) -> Self {
        self.callback = HooksPandaCallback::from_end_block_exec(self.hook);
        self
    }
}

pub struct HookBuilderCallbackTypeNeeded<T>(T);

impl HookBuilderCallbackTypeNeeded<BeforeTranslateHook> {
    pub fn before_block_translate(self) -> HookBuilder<BeforeTranslateHook> {
        HookBuilder {
            hook: self.0,
            callback: HooksPandaCallback::from_before_block_translate(self.0),
            only_kernel: None,
            enabled: true,
            asid: None,
            context: std::ptr::null_mut(),
        }
    }
}

impl HookBuilderCallbackTypeNeeded<AfterBlockHook> {
    pub fn after_block_exec(self) -> HookBuilder<AfterBlockHook> {
        HookBuilder {
            hook: self.0,
            callback: HooksPandaCallback::from_after_block_exec(self.0),
            only_kernel: None,
            enabled: true,
            asid: None,
            context: std::ptr::null_mut(),
        }
    }
}

impl HookBuilderCallbackTypeNeeded<InvalidateOpHook> {
    pub fn before_block_exec_invalidate_opt(self) -> HookBuilder<InvalidateOpHook> {
        HookBuilder {
            hook: self.0,
            callback: HooksPandaCallback::from_before_block_exec_invalidate_opt(self.0),
            only_kernel: None,
            enabled: true,
            asid: None,
            context: std::ptr::null_mut(),
        }
    }
}
