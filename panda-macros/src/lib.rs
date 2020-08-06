use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn init(_: TokenStream, function: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(function as syn::ItemFn);

    let args = if func.sig.inputs.is_empty() {
        None
    } else {
        Some(quote!( unsafe { &mut *plugin } ))
    };

    let func_name = &func.sig.ident;

    quote!(
        mod __panda_internal {
            use super::*;

            #[no_mangle]
            unsafe extern "C" fn init_plugin(plugin: *mut ::panda::PluginHandle) {
                for cb in ::panda::inventory::iter::<::panda::Callback> {
                    ::panda::sys::panda_register_callback(plugin as _, cb.cb_type, ::core::mem::transmute(cb.fn_pointer));
                }

                #func_name(#args);
            }
            
            #[no_mangle]
            unsafe extern "C" fn uninit_plugin(plugin: *mut ::panda::PluginHandle) {
                for cb in ::panda::inventory::iter::<::panda::UninitCallback> {
                    cb.0(unsafe { &mut *plugin });
                }
            }
        }

        #func
    ).into()
}

#[proc_macro_attribute]
pub fn uninit(_: TokenStream, function: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(function as syn::ItemFn);
    let func_name = &func.sig.ident;

    quote!(
        ::panda::inventory::submit! {
            ::panda::UninitCallback(#func_name)
        }

        #func
    ).into()
}

macro_rules! define_callback_attributes {
    ($(
        ($attr_name:ident, $const_name:ident, ($($arg:ty),*))
    ),*) => {
        $(
            #[proc_macro_attribute]
            pub fn $attr_name(_: TokenStream, function: TokenStream) -> TokenStream {
                let function = syn::parse_macro_input!(function as syn::ItemFn);
                let func = &function.sig.ident;

                quote!(
                    const _: fn() = || {
                        use ::panda::sys::*;
                        fn assert_callback_arg_types<T: ?Sized + Fn($($arg),*)>(_ :&T) {}
                        assert_callback_arg_types(&#func);
                    };

                    use ::panda::inventory;
                    ::panda::inventory::submit! {
                        ::panda::Callback::new(
                            ::panda::sys::$const_name,
                            #func as *const ()
                        )
                    }

                    #function
                ).into()
            }
        )*
    }
}

// TODO: Add docstrings to all the callbacks
define_callback_attributes!(
    (before_block_translate, panda_cb_type_PANDA_CB_BEFORE_BLOCK_TRANSLATE, (&mut CPUState, target_ptr_t)),
    (after_block_translate, panda_cb_type_PANDA_CB_AFTER_BLOCK_TRANSLATE, (&mut CPUState, &mut TranslationBlock)),
    (before_block_exec_invalidate_opt, panda_cb_type_PANDA_CB_BEFORE_BLOCK_EXEC_INVALIDATE_OPT, (&mut CPUState, &mut TranslationBlock)),
    (before_block_exec, panda_cb_type_PANDA_CB_BEFORE_BLOCK_EXEC, (&mut CPUState, &mut TranslationBlock)),
    (after_block_exec, panda_cb_type_PANDA_CB_AFTER_BLOCK_EXEC, (&mut CPUState, &mut TranslationBlock, u8)),
    (insn_translate, panda_cb_type_PANDA_CB_INSN_TRANSLATE, (&mut CPUState, target_ptr_t)),
    (insn_exec, panda_cb_type_PANDA_CB_INSN_EXEC, (&mut CPUState, target_ptr_t)),
    (after_insn_translate, panda_cb_type_PANDA_CB_AFTER_INSN_TRANSLATE, (&mut CPUState, target_ptr_t)),
    (after_insn_exec, panda_cb_type_PANDA_CB_AFTER_INSN_EXEC, (&mut CPUState, target_ptr_t)),
    (virt_mem_before_read, panda_cb_type_PANDA_CB_VIRT_MEM_BEFORE_READ, (&mut CPUState, target_ptr_t, target_ptr_t, usize)),
    (virt_mem_before_write, panda_cb_type_PANDA_CB_VIRT_MEM_BEFORE_WRITE, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),
    (phys_mem_before_read, panda_cb_type_PANDA_CB_PHYS_MEM_BEFORE_READ, (&mut CPUState, target_ptr_t, target_ptr_t, usize)),
    (phys_mem_before_write, panda_cb_type_PANDA_CB_PHYS_MEM_BEFORE_WRITE, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),
    (virt_mem_after_read, panda_cb_type_PANDA_CB_VIRT_MEM_AFTER_READ, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),
    (virt_mem_after_write, panda_cb_type_PANDA_CB_VIRT_MEM_AFTER_WRITE, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),
    (phys_mem_after_read, panda_cb_type_PANDA_CB_PHYS_MEM_AFTER_READ, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),
    (phys_mem_after_write, panda_cb_type_PANDA_CB_PHYS_MEM_AFTER_WRITE, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),
    (mmio_after_read, panda_cb_type_PANDA_CB_MMIO_AFTER_READ, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u64)),
    (mmio_before_write, panda_cb_type_PANDA_CB_MMIO_BEFORE_WRITE, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u64)),
    (hd_read, panda_cb_type_PANDA_CB_HD_READ, (&mut CPUState)),
    (hd_write, panda_cb_type_PANDA_CB_HD_WRITE, (&mut CPUState)),
    (guest_hypercall, panda_cb_type_PANDA_CB_GUEST_HYPERCALL, (&mut CPUState)),
    (monitor, panda_cb_type_PANDA_CB_MONITOR, (&mut Monitor, *const u8)),
    (cpu_restore_state, panda_cb_type_PANDA_CB_CPU_RESTORE_STATE, (&mut CPUState, &mut TranslationBlock)),
    (before_loadvm, panda_cb_type_PANDA_CB_BEFORE_LOADVM, ()),
    (asid_changed, panda_cb_type_PANDA_CB_ASID_CHANGED, (&mut CPUState, target_ptr_t, target_ptr_t)),
    (replay_hd_transfer, panda_cb_type_PANDA_CB_REPLAY_HD_TRANSFER, (&mut CPUState, u32, target_ptr_t, target_ptr_t, usize)),
    (replay_net_transfer, panda_cb_type_PANDA_CB_REPLAY_NET_TRANSFER, (&mut CPUState, u32, u64, u64, usize)),
    (replay_serial_receive, panda_cb_type_PANDA_CB_REPLAY_SERIAL_RECEIVE, (&mut CPUState, target_ptr_t, u8)),
    (replay_serial_read, panda_cb_type_PANDA_CB_REPLAY_SERIAL_READ, (&mut CPUState, target_ptr_t, u32, u8)),
    (replay_serial_send, panda_cb_type_PANDA_CB_REPLAY_SERIAL_SEND, (&mut CPUState, target_ptr_t, u8)),
    (replay_serial_write, panda_cb_type_PANDA_CB_REPLAY_SERIAL_WRITE, (&mut CPUState, target_ptr_t, u32, u8)),
    (replay_before_dma, panda_cb_type_PANDA_CB_REPLAY_BEFORE_DMA, (&mut CPUState, *const u8, hwaddr, usize, bool)),
    (replay_after_dma, panda_cb_type_PANDA_CB_REPLAY_AFTER_DMA, (&mut CPUState, *mut u8, hwaddr, usize, bool)),
    (replay_handle_packet, panda_cb_type_PANDA_CB_REPLAY_HANDLE_PACKET, (&mut CPUState, *mut u8, usize, u8, u64)),
    (after_cpu_exec_enter, panda_cb_type_PANDA_CB_AFTER_CPU_EXEC_ENTER, (&mut CPUState)),
    (before_cpu_exec_exit, panda_cb_type_PANDA_CB_BEFORE_CPU_EXEC_EXIT, (&mut CPUState, bool)),
    (after_machine_init, panda_cb_type_PANDA_CB_AFTER_MACHINE_INIT, (&mut CPUState)),
    (after_loadvm, panda_cb_type_PANDA_CB_AFTER_LOADVM, (&mut CPUState)),
    (top_loop, panda_cb_type_PANDA_CB_TOP_LOOP, (&mut CPUState)),
    (during_machine_init, panda_cb_type_PANDA_CB_DURING_MACHINE_INIT, (&mut MachineState)),
    (main_loop_wait, panda_cb_type_PANDA_CB_MAIN_LOOP_WAIT, ()),
    (pre_shutdown, panda_cb_type_PANDA_CB_PRE_SHUTDOWN, ()),
    (unassigned_io_read, panda_cb_type_PANDA_CB_UNASSIGNED_IO_READ, (&mut CPUState, target_ptr_t, hwaddr, usize, u64)),
    (unassigned_io_write, panda_cb_type_PANDA_CB_UNASSIGNED_IO_WRITE, (&mut CPUState, target_ptr_t, hwaddr, usize, u64)),
    (before_handle_exception, panda_cb_type_PANDA_CB_BEFORE_HANDLE_EXCEPTION, (&mut CPUState, i32)),
    (before_handle_interrupt, panda_cb_type_PANDA_CB_BEFORE_HANDLE_INTERRUPT, (&mut CPUState, i32)),
    (last, panda_cb_type_PANDA_CB_LAST, ())
);
