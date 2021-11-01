/// Declare and export a plugin-to-plugin (PPP) callback, which allows other plugins
/// to add callbacks for this plugin to run.
///
/// ## Example
///
/// For example, if you were writing a Rust plugin that would use a callback that 
/// runs every other basic block, you could declare and use two callbacks like so:
///
/// ```
/// use panda::{Callback, export_ppp_callback};
/// use panda::prelude::*;
/// 
/// export_ppp_callback! {
///     pub(crate) fn on_every_even_block(cpu: &mut CPUState);
///     pub(crate) fn on_every_odd_block(cpu: &mut CPUState);
/// }
/// 
/// #[panda::init]
/// fn init() {
///     let mut i = 0;
///     Callback::new().before_block_exec(move |cpu, _| {
///         if i % 2 == 0 {
///             on_every_even_block::trigger(cpu);
///         } else {
///             on_every_odd_block::trigger(cpu);
///         }
///
///         i += 1;
///     });
/// }
/// ```
///
/// (For further usage see `panda-rs/examples/ppp_callback_export.rs`)
///
/// The return type of each callback can be any which implements [`CallbackReturn`], a 
/// trait which describes how to fold all the return values into a single return value
/// to be returned by `<callback_name>::trigger(...)`. For example a callback that returns
/// a `bool` will return `true` if any of the callbacks return `true`, and will only return
/// false if every registered callback returns false.
///
/// If you wish to alter this behavior for existing types, use the [newtype pattern], 
/// which will allow you to provide your own implementation by implementing the trait.
///
/// [newtype pattern]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html
///
/// **Note:** All callback arguments and return values are expected to be FFI safe. If rustc 
/// emits a warning for a given type, it is very likely it is not compatible with the 
/// C ABI. In order to have your own custom types be FFI-safe, they should be marked 
/// either `#[repr(transparent)]` or `#[repr(C)]` or should be treated as opaque types 
/// (and thus should only be created and accessed within the same plugin and passed as
/// references).
#[macro_export]
macro_rules! export_ppp_callback {
    {
        $(
            $vis:vis fn $cb_name:ident (
                $(
                    $arg:ident : $arg_ty:ty
                ),* $(,)?
            ) $(-> $ret_ty:ty)?;
        )*
    } => {$(
        $vis mod $cb_name {
            use super::*;

            use ::std::ffi::c_void;

            #[derive(PartialEq)]
            struct PppContextInternal(*mut c_void);

            unsafe impl Sync for PppContextInternal {}
            unsafe impl Send for PppContextInternal {}

            $vis type CallbackType = extern "C" fn($( $arg_ty ),*) $(-> $ret_ty)?;
            $vis type CallbackTypeWithContext = extern "C" fn(*mut c_void, $( $arg_ty ),*) $(-> $ret_ty)?;

            extern "C" fn trampoline(context: *mut c_void, $($arg : $arg_ty),*) $(-> $ret_ty)? {
                let cb: CallbackType = unsafe {
                    ::core::mem::transmute(context)
                };

                cb($($arg),*)
            }

            #[export_name = concat!("ppp_add_cb_", stringify!($cb_name))]
            $vis extern "C" fn add_callback(callback: CallbackType) {
                unsafe {
                    add_callback_with_context(trampoline, ::core::mem::transmute(callback))
                }
            }

            #[export_name = concat!("ppp_add_cb_", stringify!($cb_name), "_with_context")]
            $vis extern "C" fn add_callback_with_context(
                callback: CallbackTypeWithContext,
                context: *mut c_void,
            ) {
                CALLBACKS
                    .lock()
                    .unwrap()
                    .push((callback, PppContextInternal(context)));
            }

            #[export_name = concat!("ppp_remove_cb_", stringify!($cb_name))]
            $vis extern "C" fn remove_callback(callback: CallbackType) -> bool {
                unsafe {
                    remove_callback_with_context(trampoline, ::core::mem::transmute(callback))
                }
            }

            #[export_name = concat!("ppp_remove_cb_", stringify!($cb_name), "_with_context")]
            $vis extern "C" fn remove_callback_with_context(
                callback: CallbackTypeWithContext,
                context: *mut c_void,
            ) -> bool {
                let context = PppContextInternal(context);
                let mut callbacks = CALLBACKS.lock().unwrap();
                let old_len = callbacks.len();

                callbacks.retain(
                    |(cb, cb_ctxt)| (*cb as usize, cb_ctxt) != (callback as _, &context)
                );

                callbacks.len() != old_len
            }

            $crate::lazy_static::lazy_static! {
                static ref CALLBACKS: ::std::sync::Mutex<
                    Vec<(CallbackTypeWithContext, PppContextInternal)>
                > = ::std::sync::Mutex::new(Vec::new());
            }

            $vis fn trigger($($arg : $arg_ty),*) $(-> $ret_ty)? {
                CALLBACKS.lock()
                    .unwrap()
                    .iter_mut()
                    .map(|(callback, PppContextInternal(context))| callback(
                        *context,
                        $($arg),*
                    ))
                    .fold(
                        $crate::__callback_fold_default!($($ret_ty)?),
                        $crate::__callback_fold_fn!($($ret_ty)?)
                    )
            }
        }
    )*};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __callback_fold_default {
    () => {
        ()
    };
    ($ty:ty) => {
        <$ty as $crate::CallbackReturn>::callback_fold_default()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __callback_fold_fn {
    () => {
        (|(), _| ())
    };
    ($ty:ty) => {
        <$ty as $crate::CallbackReturn>::fold_callback_return
    };
}

/// A type which can be returned from a callback and folded into a single value
///
/// As an example, here's the provided implementation for `bool`:
///
/// ```no_run
/// /// Returns true if any of the callbacks returned true without short circuiting
/// impl CallbackReturn for bool {
///     type FoldType = bool;
/// 
///     fn fold_callback_return(folded: Self::FoldType, ret: Self) -> Self::FoldType {
///         folded | ret
///     }
/// }
/// ```
///
/// The way this is used is by taking the `FoldType` and creating a default instance. For
/// a `bool` this will be `false`. Then, for each callback return value it will take the 
/// previous instance (starting with `false`) and do `previous | current_callback_return`.
///
/// The result will mean that if callbacks `a`, `b`, and `c` are registered, the resulting
/// value returned from `<callback>::trigger(...)` is `((false | a) | b) | c`. (Parenthesis
/// added to demonstrate folding order)
pub trait CallbackReturn {
    type FoldType: Default;

    /// Function for folding each callback return value into a single value
    fn fold_callback_return(folded: Self::FoldType, ret: Self) -> Self::FoldType;

    /// Get the default value for folding the callback returns into a single value
    fn callback_fold_default() -> Self::FoldType {
        Self::FoldType::default()
    }
}

/// Returns true if any of the callbacks returned true without short circuiting
impl CallbackReturn for bool {
    type FoldType = bool;

    fn fold_callback_return(folded: Self::FoldType, ret: Self) -> Self::FoldType {
        folded | ret
    }
}

macro_rules! impl_for_ints {
    ($($ty:ty)*) => {
        $(
            /// Returns the first non-zero value without short-circuiting
            impl CallbackReturn for $ty {
                type FoldType = $ty;

                fn fold_callback_return(folded: Self::FoldType, ret: Self) -> Self::FoldType {
                    if folded != 0 {
                        folded
                    } else {
                        ret
                    }
                }
            }
        )*
    };
}

impl_for_ints!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize);
