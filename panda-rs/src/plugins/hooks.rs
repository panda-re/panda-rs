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

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum KernelMode {
    Any = 0,
    KernelOnly = 1,
    UserOnly = 2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Hook {
    pub addr: target_ulong,
    pub asid: target_ulong,
    pub cb: HooksPandaCallback,
    pub km: KernelMode,
    pub enabled: bool,
    pub sym: Symbol,
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

pub struct HookBuilder<T> {
    hook: T,
    callback: HooksPandaCallback,
    only_kernel: Option<bool>,
    enabled: bool,
    asid: Option<target_ulong>,
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
        }
    }
}
