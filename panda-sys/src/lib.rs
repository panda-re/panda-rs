

macro_rules! if_any_two_features {
    ($first:literal $(, $feature:literal)+ { $item:item }) => {
        #[cfg(all(feature = $first, any($(feature = $feature),*)))]
        $item

        if_any_two_features!($($feature),* { $item });
    };
    ($first:literal { $item:item }) => {};
}

macro_rules! if_not_any_two_features {
    (@inner $first:literal $(, $feature:literal)+ { $item:item }) => {
        #[cfg(not(all(feature = $first, any($(feature = $feature),*))))]
        if_not_any_two_features!(@inner $($feature),* {
            $item
        });
    };
    (@inner $first:literal { $item:item }) => {
        $item
    };
    ($($features:literal),* { $first:item $($items:item)* }) => {
        if_not_any_two_features!(@inner $($features),* { $first });

        if_not_any_two_features!($($features),* { $($items)* });
    };
    ($($features:literal),* {  }) => {};
}

if_any_two_features!("x86_64", "i386", "arm", "ppc", "mips", "mipsel", "mips64" {
    compile_error!("Cannot enable two features at once, make sure you are using `default-features = false`");
});

if_not_any_two_features!("x86_64", "i386", "arm", "ppc", "mips", "mipsel", "mips64" {

    #[allow(nonstandard_style)]
    #[allow(improper_ctypes)] // TODO!!! need to actually fix these FFI issues...
    mod bindings;

    mod extensions;
    
    pub use bindings::*;
});
