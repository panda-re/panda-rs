/// A trait representing types that can be used as the return value for a `#[panda::init]`
/// function
pub trait InitReturn {
    fn into_init_bool(self) -> bool;
}

impl InitReturn for bool {
    fn into_init_bool(self) -> bool {
        self
    }
}

impl InitReturn for () {
    fn into_init_bool(self) -> bool {
        true
    }
}

impl InitReturn for i32 {
    fn into_init_bool(self) -> bool {
        self == 0
    }
}

impl<I: InitReturn, E: core::fmt::Debug> InitReturn for Result<I, E> {
    fn into_init_bool(self) -> bool {
        match self {
            Ok(x) => x.into_init_bool(),
            Err(err) => {
                eprintln!("Error initializing plugin: {:?}", err);

                false
            }
        }
    }
}
