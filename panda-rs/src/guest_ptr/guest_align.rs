pub(crate) trait GuestAlign {
    const ALIGN: usize;
}

macro_rules! align {
    ($ident:ident = $align:literal) => {
        impl GuestAlign for $ident {
            const ALIGN: usize = $align / 8;
        }
    };
}

#[cfg(any(
    feature = "x86_64",
    feature = "i386",
    feature = "arm",
    feature = "aarch64",
    feature = "mips",
    feature = "mipsel",
    feature = "mips64",
    feature = "ppc",
))]
macro_rules! alignments {
    () => {
        align!(bool = 8);

        align!(f32 = 32);
        align!(f64 = 64);

        align!(u8 = 8);
        align!(u16 = 16);
        align!(u32 = 32);
        align!(u64 = 64);
        align!(u128 = 128);

        align!(i8 = 8);
        align!(i16 = 16);
        align!(i32 = 32);
        align!(i64 = 64);
        align!(i128 = 128);
    };
}

alignments!();
