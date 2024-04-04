pub mod registers;
pub mod instructions;
pub mod CPU;
pub mod memory;


#[cfg(feature = "debug")]
fn debug_print(args: std::fmt::Arguments) {
    println!("{}", args);
}

#[cfg(not(feature = "debug"))]
fn debug_print(_args: std::fmt::Arguments) {
    // Do nothing
}

const SYSTEM_FREQUENCY_CLOCK: u64 = 1_048_576;
