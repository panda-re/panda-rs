use std::iter;
use proc_macro::TokenStream;
use quote::quote;
use darling::{FromMeta, FromField};

/// (**Required** Callback) Called when the plugin is being uninitialized
///
///### Example
///
/// ```rust
///use panda::PluginHandle;
///
/// #[panda::init]
/// fn start(_: &mut PluginHandle) {
///     println!("Plugin started up!");
/// }
/// ```
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

/// (Callback) Called when the plugin is being uninitialized
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

#[derive(FromField)]
#[darling(attributes(arg))]
struct DeriveArgs {
    #[darling(default)]
    about: Option<String>,
    #[darling(default)]
    default: Option<syn::Lit>,
    #[darling(default)]
    required: bool,
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

fn derive_args_to_mappings(
    DeriveArgs { about, default, ident, ty, required }: DeriveArgs
) -> (syn::Stmt, syn::Ident) {
    let name = &ident;
    let default = if let Some(default) = default {
        match default {
            syn::Lit::Str(string) => quote!(::std::string::String::from(#string)),
            default => quote!(#default)
        }
    } else {
        quote!(Default::default())
    };
    let about = about.unwrap_or_default();
    (
        syn::parse_quote!(
            let #name = <#ty as ::panda::panda_arg::GetPandaArg>::get_panda_arg(
                __args_ptr,
                stringify!(#name),
                #default,
                #about,
                #required
            );
        ),
        ident.unwrap()
    )
}

fn get_field_statements(fields: &syn::Fields) -> Result<(Vec<syn::Stmt>, Vec<syn::Ident>), darling::Error> {
    Ok(fields
        .iter()
        .map(DeriveArgs::from_field)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(derive_args_to_mappings)
        .unzip())
}

fn get_name(attrs: &[syn::Attribute]) -> Option<String> {
    attrs.iter()
        .find(|attr| attr.path.get_ident().map(|x| x.to_string() == "name").unwrap_or(false))
        .map(|attr| attr.parse_meta().ok())
        .flatten()
        .map(|meta| if let syn::Meta::NameValue(syn::MetaNameValue { lit: syn::Lit::Str(s), .. }) = meta {
            Some(s.value())
        } else {
            None
        })
        .flatten()
}

#[proc_macro_derive(PandaArgs, attributes(name, arg))]
pub fn derive_panda_args(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemStruct);

    let name = match get_name(&input.attrs) {
        Some(name) => name,
        None => return quote!(compiler_error!("Missing plugin name, add `#[name = ...]` above struct")).into()
    };

    let ident = &input.ident;

    match get_field_statements(&input.fields) {
        Ok((statements, fields)) => {
            let format_args =
                iter::repeat("{}={}")
                    .take(statements.len())
                    .collect::<Vec<_>>()
                    .join(",");
            quote!(
                impl ::panda::PandaArgs for #ident {
                    fn from_panda_args() -> Self {
                        let name = ::std::ffi::CString::new(#name).unwrap();
                        
                        unsafe {
                            let __args_ptr = ::panda::sys::panda_get_args(name.as_ptr());

                            #(
                                #statements
                            )*
                            
                            ::panda::sys::panda_free_args(__args_ptr);

                            Self {
                                #(#fields),*
                            }
                        }
                    }

                    fn to_panda_args_str(&self) -> ::std::string::String {
                        format!(
                            concat!(#name, ":", #format_args),
                            #(
                                stringify!(#fields), self.#fields
                            ),*
                       )
                    }
                }
            )
        },
        Err(err) => err.write_errors()
    }.into()
}

macro_rules! define_callback_attributes {
    ($(
        $($doc:literal)*
        ($attr_name:ident, $const_name:ident, ($($arg:ty),*))
    ),*) => {
        $(
            doc_comment::doc_comment!{
                concat!("(Callback) ", $($doc, "\n",)* "\n\nCallback arguments: (", $("`", stringify!($arg), "`, ",)* ")\n### Example\n```rust\nuse panda::{sys::*, PluginHandle};\n\n#[panda::", stringify!($attr_name),"]\nfn callback(", $("_: ", stringify!($arg), ", ", )* ") {\n    // do stuff\n}\n```"),
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

                        ::panda::inventory::submit! {
                            #![crate = ::panda]
                            ::panda::Callback::new(
                                ::panda::sys::$const_name,
                                #func as *const ()
                            )
                        }

                        #function
                    ).into()
                }
            }
        )*
    }
}

define_callback_attributes!(
    "Called before translation of each basic block.

    Callback ID: PANDA_CB_BEFORE_BLOCK_TRANSLATE
    
    Arguments:
     CPUState *env:   the current CPU state
     target_ptr_t pc: the guest PC we are about to translate
    
    Helper call location: cpu-exec.c
    
    Return value:
     none
    "
    (before_block_translate, panda_cb_type_PANDA_CB_BEFORE_BLOCK_TRANSLATE, (&mut CPUState, target_ptr_t)),
    "Called after execution of every basic block.
    If exitCode > TB_EXIT_IDX1, then the block exited early.

    Callback ID: PANDA_CB_AFTER_BLOCK_EXEC

       after_block_exec:

       Arguments:
        CPUState *env:        the current CPU state
        TranslationBlock *tb: the TB we just executed
        uint8_t exitCode:     why the block execution exited

       Helper call location: cpu-exec.c

       Return value:
        none
    "
    (after_block_translate, panda_cb_type_PANDA_CB_AFTER_BLOCK_TRANSLATE, (&mut CPUState, &mut TranslationBlock)),
    "Called before execution of every basic block, with the option
        to invalidate the TB.

    Callback ID: PANDA_CB_BEFORE_BLOCK_EXEC_INVALIDATE_OPT

       before_block_exec_invalidate_opt:

       Arguments:
        CPUState *env:        the current CPU state
        TranslationBlock *tb: the TB we are about to execute

       Helper call location: cpu-exec.c (indirectly)

       Return value:
        true if we should invalidate the current translation block
        and retranslate, false otherwise.
    "
    (before_block_exec_invalidate_opt, panda_cb_type_PANDA_CB_BEFORE_BLOCK_EXEC_INVALIDATE_OPT, (&mut CPUState, &mut TranslationBlock)),
    "Called before execution of every basic block.

    Callback ID: PANDA_CB_BEFORE_BLOCK_EXEC

       Arguments:
        CPUState *env:        the current CPU state
        TranslationBlock *tb: the TB we are about to execute

       Helper call location: cpu-exec.c

       Return value:
        none
    "
    (before_block_exec, panda_cb_type_PANDA_CB_BEFORE_BLOCK_EXEC, (&mut CPUState, &mut TranslationBlock)),
    "Called after execution of every basic block.
        If exitCode > TB_EXIT_IDX1, then the block exited early.

    Callback ID: PANDA_CB_AFTER_BLOCK_EXEC

       Arguments:
        CPUState *env:        the current CPU state
        TranslationBlock *tb: the TB we just executed
        uint8_t exitCode:     why the block execution exited

       Helper call location: cpu-exec.c

       Return value:
        none
    "
    (after_block_exec, panda_cb_type_PANDA_CB_AFTER_BLOCK_EXEC, (&mut CPUState, &mut TranslationBlock, u8)),
    "Called before the translation of each instruction.

    Callback ID: PANDA_CB_INSN_TRANSLATE

       Arguments:
        CPUState *env:   the current CPU state
        target_ptr_t pc: the guest PC we are about to translate

       Helper call location: panda/target/ARCH/translate.c

       Return value:
        true if PANDA should insert instrumentation into the generated code,
        false otherwise

       Notes:
        This allows a plugin writer to instrument only a small number of
        instructions, avoiding the performance hit of instrumenting everything.
        If you do want to instrument every single instruction, just return
        true. See the documentation for PANDA_CB_INSN_EXEC for more detail.
    "
    (insn_translate, panda_cb_type_PANDA_CB_INSN_TRANSLATE, (&mut CPUState, target_ptr_t)),
    "Called before execution of any instruction identified by the
        PANDA_CB_INSN_TRANSLATE callback.

    Callback ID: PANDA_CB_INSN_EXEC

       Arguments:
        CPUState *env:   the current CPU state
        target_ptr_t pc: the guest PC we are about to execute

       Helper call location: TBA

       Return value:
        unused

       Notes:
        This instrumentation is implemented by generating a call to a
        helper function just before the instruction itself is generated.
        This is fairly expensive, which is why it's only enabled via
        the PANDA_CB_INSN_TRANSLATE callback.
    "
    (insn_exec, panda_cb_type_PANDA_CB_INSN_EXEC, (&mut CPUState, target_ptr_t)),
    "Called after the translation of each instruction.

    Callback ID: PANDA_CB_AFTER_INSN_TRANSLATE

       Arguments:
        CPUState *env:   the current CPU state
        target_ptr_t pc: the next guest PC we've translated

       Helper call location: panda/target/ARCH/translate.c

       Return value:
        true if PANDA should insert instrumentation into the generated code,
        false otherwise

       Notes:
        See `insn_translate`, callbacks are registered via PANDA_CB_AFTER_INSN_EXEC
    "
    (after_insn_translate, panda_cb_type_PANDA_CB_AFTER_INSN_TRANSLATE, (&mut CPUState, target_ptr_t)),
    "Called after execution of an instruction identified by the
        PANDA_CB_AFTER_INSN_TRANSLATE callback

    Callback ID: PANDA_CB_AFTER_INSN_EXEC

       Arguments:
        CPUState *env:   the current CPU state
        target_ptr_t pc: the next guest PC already executed

       Helper call location: TBA

       Return value:
        unused

       Notes:
        See `insn_exec`. Enabled via the PANDA_CB_AFTER_INSN_TRANSLATE callback.
    "
    (after_insn_exec, panda_cb_type_PANDA_CB_AFTER_INSN_EXEC, (&mut CPUState, target_ptr_t)),
    "Called before memory is read.

    Callback ID: PANDA_CB_VIRT_MEM_BEFORE_READ

       Arguments:
        CPUState *env:     the current CPU state
        target_ptr_t pc:   the guest PC doing the read
        target_ptr_t addr: the (virtual) address being read
        size_t size:       the size of the read

       Helper call location: TBA

       Return value:
        none
    "
    (virt_mem_before_read, panda_cb_type_PANDA_CB_VIRT_MEM_BEFORE_READ, (&mut CPUState, target_ptr_t, target_ptr_t, usize)),
    "Called before memory is written.

    Callback ID: PANDA_CB_VIRT_MEM_BEFORE_WRITE

       Arguments:
        CPUState *env:     the current CPU state
        target_ptr_t pc:   the guest PC doing the write
        target_ptr_t addr: the (virtual) address being written
        size_t size:       the size of the write
        uint8_t *buf:      pointer to the data that is to be written

       Helper call location: TBA

       Return value:
        none
    "
    (virt_mem_before_write, panda_cb_type_PANDA_CB_VIRT_MEM_BEFORE_WRITE, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),
    "Called after memory is read.

    Callback ID: PANDA_CB_PHYS_MEM_BEFORE_READ

       Arguments:
        CPUState *env:     the current CPU state
        target_ptr_t pc:   the guest PC doing the read
        target_ptr_t addr: the (physical) address being read
        size_t size:       the size of the read

       Helper call location: TBA

       Return value:
        none
    "
    (phys_mem_before_read, panda_cb_type_PANDA_CB_PHYS_MEM_BEFORE_READ, (&mut CPUState, target_ptr_t, target_ptr_t, usize)),
    "Called before memory is written.

    Callback ID: PANDA_CB_PHYS_MEM_BEFORE_WRITE

       Arguments:
        CPUState *env:     the current CPU state
        target_ptr_t pc:   the guest PC doing the write
        target_ptr_t addr: the (physical) address being written
        size_t size:       the size of the write
        uint8_t *buf:      pointer to the data that is to be written

       Helper call location: TBA

       Return value:
        none
    "
    (phys_mem_before_write, panda_cb_type_PANDA_CB_PHYS_MEM_BEFORE_WRITE, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),
    "Called after memory is read.

    Callback ID: PANDA_CB_VIRT_MEM_AFTER_READ

       Arguments:
        CPUState *env:     the current CPU state
        target_ptr_t pc:   the guest PC doing the read
        target_ptr_t addr: the (virtual) address being read
        size_t size:       the size of the read
        uint8_t *buf:      pointer to data just read

       Helper call location: TBA

       Return value:
        none
    "
    (virt_mem_after_read, panda_cb_type_PANDA_CB_VIRT_MEM_AFTER_READ, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),
    "Called after memory is written.

    Callback ID: PANDA_CB_VIRT_MEM_AFTER_WRITE

       Arguments:
        CPUState *env:     the current CPU state
        target_ptr_t pc:   the guest PC doing the write
        target_ptr_t addr: the (virtual) address being written
        size_t size:       the size of the write
        uint8_t *buf:      pointer to the data that was written

       Helper call location: TBA

       Return value:
        none
    "
    (virt_mem_after_write, panda_cb_type_PANDA_CB_VIRT_MEM_AFTER_WRITE, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),

    "Called after memory is read.

    Callback ID: PANDA_CB_PHYS_MEM_AFTER_READ

       Arguments:
        CPUState *env:     the current CPU state
        target_ptr_t pc:   the guest PC doing the read
        target_ptr_t addr: the (physical) address being read
        size_t size:       the size of the read
        uint8_t *buf:      pointer to data just read

       Helper call location: TBA

       Return value:
        none
    "
    (phys_mem_after_read, panda_cb_type_PANDA_CB_PHYS_MEM_AFTER_READ, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),
    "Called after memory is written.

    Callback ID: PANDA_CB_PHYS_MEM_AFTER_WRITE

       Arguments:
        CPUState *env:     the current CPU state
        target_ptr_t pc:   the guest PC doing the write
        target_ptr_t addr: the (physical) address being written
        size_t size:       the size of the write
        uint8_t *buf:      pointer to the data that was written

       Helper call location: TBA

       Return value:
        none
    "
    (phys_mem_after_write, panda_cb_type_PANDA_CB_PHYS_MEM_AFTER_WRITE, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u8)),
    "Called after MMIO memory is read.

    Callback ID: PANDA_CB_MMIO_AFTER_READ

       Arguments:
        CPUState *env:          the current CPU state
        target_ptr_t physaddr:  the physical address being read from
        target_ptr_t vaddr:     the virtual address being read from
        size_t size:            the size of the read
        uin64_t *val:           the value being read

       Helper call location: cputlb.c

       Return value:
        none
    "
    (mmio_after_read, panda_cb_type_PANDA_CB_MMIO_AFTER_READ, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u64)),
    "Called after MMIO memory is written to.

    Callback ID: PANDA_CB_MMIO_BEFORE_WRITE

       Arguments:
        CPUState *env:          the current CPU state
        target_ptr_t physaddr:  the physical address being written to
        target_ptr_t vaddr:     the virtual address being written to
        size_t size:            the size of the write
        uin64_t *val:           the value being written

       Helper call location: cputlb.c

       Return value:
        none
    "
    (mmio_before_write, panda_cb_type_PANDA_CB_MMIO_BEFORE_WRITE, (&mut CPUState, target_ptr_t, target_ptr_t, usize, *mut u64)),
    "Called when there is a hard drive read

    Callback ID: PANDA_CB_HD_READ

       Note: this was added to panda_cb_type enum but no callback prototype inserted
       Here is a stub.  I'm not sure what the args should be.
       Arguments
       CPUState *env
    "
    (hd_read, panda_cb_type_PANDA_CB_HD_READ, (&mut CPUState)),
    "Called when there is a hard drive write

    Callback ID: PANDA_CB_HD_WRITE

       Note: this was added to panda_cb_type enum but no callback prototype inserted
       Here is a stub.  I'm not sure what the args should be.
       Arguments
       CPUState *env
    "
    (hd_write, panda_cb_type_PANDA_CB_HD_WRITE, (&mut CPUState)),
    "Called when a program inside the guest makes a hypercall to pass
        information from inside the guest to a plugin

    Callback ID: PANDA_CB_GUEST_HYPERCALL

       Arguments:
        CPUState *env: the current CPU state

       Helper call location: target/i386/misc_helper.c

       Return value:
        true if the callback has processed the hypercall, false if the
        hypercall has been ignored.

       Notes:
        On x86, this is called whenever CPUID is executed. On ARM, the
        MCR instructions is used. Plugins should check for magic values
        in the registers to determine if it really is a guest hypercall.
        Parameters can be passed in other registers. If the plugin
        processes the hypercall, it should return true so the execution
        of the normal instruction is skipped.
    "
    (guest_hypercall, panda_cb_type_PANDA_CB_GUEST_HYPERCALL, (&mut CPUState)),
    "Called when someone uses the plugin_cmd monitor command.

    Callback ID: PANDA_CB_MONITOR

       Arguments:
        Monitor *mon:    a pointer to the Monitor
        const char *cmd: the command string passed to plugin_cmd

       Helper call location: TBA

       Return value:
        unused

       Notes:
        The command is passed as a single string. No parsing is performed
        on the string before it is passed to the plugin, so each plugin
        must parse the string as it deems appropriate (e.g. by using strtok
        and getopt) to do more complex option processing.
        It is recommended that each plugin implementing this callback respond
        to the \"help\" message by listing the commands supported by the plugin.
        Note that every loaded plugin will have the opportunity to respond to
        each plugin_cmd; thus it is a good idea to ensure that your plugin's
        monitor commands are uniquely named, e.g. by using the plugin name
        as a prefix (\"sample_do_foo\" rather than \"do_foo\").
    "
    (monitor, panda_cb_type_PANDA_CB_MONITOR, (&mut Monitor, *const u8)),
    "Called inside of cpu_restore_state(), when there is a CPU
        fault/exception.

    Callback ID: PANDA_CB_CPU_RESTORE_STATE

       Arguments:
        CPUState *env:        the current CPU state
        TranslationBlock *tb: the current translation block

       Helper call location: translate-all.c

       Return value:
        none
    "
    (cpu_restore_state, panda_cb_type_PANDA_CB_CPU_RESTORE_STATE, (&mut CPUState, &mut TranslationBlock)),
    "Called at start of replay, before loadvm is called. This allows
        us to hook devices' loadvm handlers. Remember to unregister the
        existing handler for the device first. See the example in the
        sample plugin.

    Callback ID: PANDA_CB_BEFORE_LOADVM

       Arguments:
        none

       Helper call location: TBA

       Return value:
        unused
    "
    (before_loadvm, panda_cb_type_PANDA_CB_BEFORE_LOADVM, ()),
    "Called when asid changes.

    Callback ID: PANDA_CB_ASID_CHANGED

       Arguments:
        CPUState *env:       pointer to CPUState
        target_ptr_t oldval: old asid value
        target_ptr_t newval: new asid value

       Helper call location: target/i386/helper.c, target/arm/helper.c

       Return value:
        true if the asid should be prevented from being changed
        false otherwise

       Notes:
        The callback is only invoked implemented for x86 and ARM.
        This should break plugins which rely on it to detect context
        switches in any other architecture.
    "
    (asid_changed, panda_cb_type_PANDA_CB_ASID_CHANGED, (&mut CPUState, target_ptr_t, target_ptr_t)),
    "In replay only. Some kind of data transfer involving hard drive.

    Callback ID:     PANDA_CB_REPLAY_HD_TRANSFER,

       Arguments:
        CPUState *env:          pointer to CPUState
        uint32_t type:          type of transfer  (Hd_transfer_type)
        target_ptr_t src_addr:  address for src
        target_ptr_t dest_addr: address for dest
        size_t num_bytes:       size of transfer in bytes

       Helper call location: panda/src/rr/rr_log.c

       Return value:
        none

       Helper call location: TBA

       Notes:
        Unlike most callbacks, this is neither a \"before\" or \"after\" callback.
        In replay the transfer doesn't really happen. We are *at* the point at
        which it happened, really.
    "
    (replay_hd_transfer, panda_cb_type_PANDA_CB_REPLAY_HD_TRANSFER, (&mut CPUState, u32, target_ptr_t, target_ptr_t, usize)),
    "In replay only, some kind of data transfer within the network card
       (currently, only the E1000 is supported).

    Callback ID:     PANDA_CB_REPLAY_NET_TRANSFER,

       Arguments:
        CPUState *env:          pointer to CPUState
        uint32_t type:          type of transfer  (Net_transfer_type)
        uint64_t src_addr:      address for src
        uint64_t dest_addr:     address for dest
        size_t num_bytes:       size of transfer in bytes

       Helper call location: panda/src/rr/rr_log.c

       Return value:
        none

       Notes:
        Unlike most callbacks, this is neither a \"before\" or \"after\" callback.
        In replay the transfer doesn't really happen. We are *at* the point at
        which it happened, really.
        Also, the src_addr and dest_addr may be for either host (ie. a location
        in the emulated network device) or guest, depending upon the type.
    "
    (replay_net_transfer, panda_cb_type_PANDA_CB_REPLAY_NET_TRANSFER, (&mut CPUState, u32, u64, u64, usize)),
    "In replay only, called when a byte is received on the serial port.

    Callback ID:     PANDA_CB_REPLAY_SERIAL_RECEIVE,

       Arguments:
        CPUState *env:          pointer to CPUState
        target_ptr_t fifo_addr: address of the data within the fifo
        uint8_t value:          value received

       Helper call location: panda/src/rr/rr_log.c

       Return value:
        unused
    "
    (replay_serial_receive, panda_cb_type_PANDA_CB_REPLAY_SERIAL_RECEIVE, (&mut CPUState, target_ptr_t, u8)),
    "In replay only, called when a byte read from the serial RX FIFO

    Callback ID:     PANDA_CB_REPLAY_SERIAL_READ,

       Arguments:
        CPUState *env:          pointer to CPUState
        target_ptr_t fifo_addr: address of the data within the fifo (source)
        uint32_t port_addr:     address of the IO port where data is being read (destination)
        uint8_t value:          value read

       Helper call location: panda/src/rr/rr_log.c

       Return value:
        none
    "
    (replay_serial_read, panda_cb_type_PANDA_CB_REPLAY_SERIAL_READ, (&mut CPUState, target_ptr_t, u32, u8)),
    "In replay only, called when a byte is sent on the serial port.

    Callback ID:     PANDA_CB_REPLAY_SERIAL_SEND,

       Arguments:
        CPUState *env:          pointer to CPUState
        target_ptr_t fifo_addr: address of the data within the fifo
        uint8_t value:          value received

       Helper call location: panda/src/rr/rr_log.c

       Return value:
        none
    "
    (replay_serial_send, panda_cb_type_PANDA_CB_REPLAY_SERIAL_SEND, (&mut CPUState, target_ptr_t, u8)),
    "In replay only, called when a byte written to the serial TX FIFO

    Callback ID:     PANDA_CB_REPLAY_SERIAL_WRITE,


       Arguments:
        CPUState *env:          pointer to CPUState
        target_ptr_t fifo_addr: address of the data within the fifo (source)
        uint32_t port_addr:     address of the IO port where data is being read (destination)
        uint8_t value:          value read

       Helper call location: panda/src/rr/rr_log.c

       Return value:
        none
    "
    (replay_serial_write, panda_cb_type_PANDA_CB_REPLAY_SERIAL_WRITE, (&mut CPUState, target_ptr_t, u32, u8)),
    "In replay only. We are about to dma between qemu buffer and
        guest memory.

    Callback ID:     PANDA_CB_REPLAY_BEFORE_DMA,

       Arguments:
        CPUState *env:      pointer to CPUState
        const uint8_t *buf: pointer to the QEMU's device buffer ussed in the transfer
        hwaddr addr:        address written to in the guest RAM
        size_t size:        size of transfer
        bool is_write:      indicates whether the DMA transfer writes to memory

       Helper call location: exec.c

       Return value:
        none
    "
    (replay_before_dma, panda_cb_type_PANDA_CB_REPLAY_BEFORE_DMA, (&mut CPUState, *const u8, hwaddr, usize, bool)),
    "In replay only, we are about to dma between qemu buffer and guest memory

    Callback ID:     PANDA_CB_REPLAY_AFTER_DMA,

       Arguments:
        CPUState *env:      pointer to CPUState
        const uint8_t *buf: pointer to the QEMU's device buffer ussed in the transfer
        hwaddr addr:        address written to in the guest RAM
        size_t size:        size of transfer
        bool is_write:      indicates whether the DMA transfer writes to memory

       Helper call location: exec.c

       Return value:
        none
    "
    (replay_after_dma, panda_cb_type_PANDA_CB_REPLAY_AFTER_DMA, (&mut CPUState, *mut u8, hwaddr, usize, bool)),
    "In replay only, we have a packet (incoming / outgoing) in hand.

    Callback ID:   PANDA_CB_REPLAY_HANDLE_PACKET,

       Arguments:
        CPUState *env:         pointer to CPUState
        uint8_t *buf:          buffer containing packet data
        size_t size:           num bytes in buffer
        uint8_t direction:     either `PANDA_NET_RX` or `PANDA_NET_TX`
        uint64_t buf_addr_rec: the address of `buf` at the time of recording

       Helper call location: panda/src/rr/rr_log.c

       Return value:
        none

       Notes:
        `buf_addr_rec` corresponds to the address of the device buffer of
        the emulated NIC. I.e. it is the address of a VM-host-side buffer.
        It is useful for implementing network tainting in an OS-agnostic
        way, in conjunction with taint2_label_io().

        FIXME: The `buf_addr_rec` maps to the `uint8_t *buf` field of the
        internal `RR_handle_packet_args` struct. The field is dumped/loaded
        to/from the trace without proper serialization/deserialization. As
        a result, a 64bit build of PANDA will not be able to process traces
        produced by a 32bit of PANDA, and vice-versa.
        There are more internal structs that suffer from the same issue.
        This is an oversight that will eventually be fixed. But as the
        real impact is minimal (virtually nobody uses 32bit builds),
        the fix has a very low priority in the bugfix list.
    "
    (replay_handle_packet, panda_cb_type_PANDA_CB_REPLAY_HANDLE_PACKET, (&mut CPUState, *mut u8, usize, u8, u64)),
    "Called after cpu_exec calls cpu_exec_enter function.

    Callback ID: PANDA_CB_AFTER_CPU_EXEC_ENTER

       Arguments:
        CPUState *env: the current CPU state

       Helper call location: cpu-exec.c

       Return value:
        none
    "
    (after_cpu_exec_enter, panda_cb_type_PANDA_CB_AFTER_CPU_EXEC_ENTER, (&mut CPUState)),
    "Called before cpu_exec calls cpu_exec_exit function.

    Callback ID: PANDA_CB_BEFORE_CPU_EXEC_EXIT

       Arguments:
        CPUState *env: the current CPU state
        bool ranBlock: true if ran a block since previous cpu_exec_enter

       Helper call location: cpu-exec.c

       Return value:
        none
    "
    (before_cpu_exec_exit, panda_cb_type_PANDA_CB_BEFORE_CPU_EXEC_EXIT, (&mut CPUState, bool)),
    "Called right after the machine has been initialized, but before
        any guest code runs.

    Callback ID:     PANDA_CB_AFTER_MACHINE_INIT

       Arguments:
        void *cpu_env: pointer to CPUState

       Helper call location: TBA

       Return value:
        none

       Notes:
        This callback allows initialization of components that need
        access to the RAM, CPU object, etc. E.g. for the taint2 plugin,
        this is the appropriate place to call taint2_enable_taint().
    "
    (after_machine_init, panda_cb_type_PANDA_CB_AFTER_MACHINE_INIT, (&mut CPUState)),
    "Called right after a snapshot has been loaded (either with loadvm or replay initialization),
        but before any guest code runs.

    Callback ID:     PANDA_CB_AFTER_LOADVM

       Arguments:
        void *cpu_env: pointer to CPUState

       Return value:
        none

    "
    (after_loadvm, panda_cb_type_PANDA_CB_AFTER_LOADVM, (&mut CPUState)),
    "Called at the top of the loop that manages emulation.

    Callback ID:     PANDA_CB_TOP_LOOP

       Arguments:
        CPUState *env:          pointer to CPUState

       Helper call location: cpus.c

       Return value:
        unused
     "
    (top_loop, panda_cb_type_PANDA_CB_TOP_LOOP, (&mut CPUState)),
    "Called in the middle of machine initialization

    Callback ID:     PANDA_CB_DURING_MACHINE_INIT

       Arguments:
         MachineState *machine: pointer to the machine state

       Return value:
         None
     "
    (during_machine_init, panda_cb_type_PANDA_CB_DURING_MACHINE_INIT, (&mut MachineState)),
    "Called in IO thread in place where monitor cmds are processed

    Callback ID:     PANDA_CB_MAIN_LOOP_WAIT

       Arguments:
         None

       Return value:
         None
     "
    (main_loop_wait, panda_cb_type_PANDA_CB_MAIN_LOOP_WAIT, ()),
    "Called just before qemu shuts down

    Callback ID:     PANDA_CB_PRE_SHUTDOWN


       Arguments:
         None

       Return value:
         None
     "
    (pre_shutdown, panda_cb_type_PANDA_CB_PRE_SHUTDOWN, ()),
    "Called when the guest attempts to read from an unmapped peripheral via MMIO

    Callback ID:     PANDA_CB_UNASSIGNED_IO_WRITE

       Arguments:
         pc: Guest program counter at time of write
         addr: Physical address written to
         size: Size of write
         val: Pointer to a buffer that will be passed to the guest as the result of the read

       Return value:
         True if value read was changed by a PANDA plugin and should be returned
         False if error-logic (invalid write) should be run
     "
    (unassigned_io_read, panda_cb_type_PANDA_CB_UNASSIGNED_IO_READ, (&mut CPUState, target_ptr_t, hwaddr, usize, u64)),
    "Called when the guest attempts to write to an unmapped peripheral via MMIO

    Callback ID:     PANDA_CB_UNASSIGNED_IO_WRITE

       Arguments:
         pc: Guest program counter at time of write
         addr: Physical address written to
         size: Size of write
         val: Data being written, up to 8 bytes

       Return value:
         True if the write should be allowed without error
         False if normal behavior should be used (error-logic)
     "
    (unassigned_io_write, panda_cb_type_PANDA_CB_UNASSIGNED_IO_WRITE, (&mut CPUState, target_ptr_t, hwaddr, usize, u64)),
    "Called just before we are about to handle an exception.
    
    Callback ID:     PANDA_CB_BEFORE_HANDLE_EXCEPTION 

       Note: only called for cpu->exception_index > 0

       Aguments:
         exception_index (the current exception number)

       Return value:
         a new exception_index.

       Note: There might be more than one callback for this location.
       First callback that returns an exception index that *differs*
       from the one passed as an arg wins. That is what we return as
       the new exception index, which will replace
       cpu->exception_index
     "
    (before_handle_exception, panda_cb_type_PANDA_CB_BEFORE_HANDLE_EXCEPTION, (&mut CPUState, i32)),
    (before_handle_interrupt, panda_cb_type_PANDA_CB_BEFORE_HANDLE_INTERRUPT, (&mut CPUState, i32))
);
