#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;
#[cfg(feature = "axstd")]
macro_rules! println {
    ($($arg:tt)*) => {
        axstd::println!("\x1b[34m{}\x1b[0m", format_args!($($arg)*));
    };
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("[WithColor]: Hello, Arceos!");
}
