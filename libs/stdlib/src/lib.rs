// FORK(no_std): `std` builds are ordinary std crates; mcu-wamr builds compile without std.
#![cfg_attr(not(any(test, feature = "std")), no_std)]

// Definitions of the core standard function modules for IEC61131-3

// FORK(no_std): mcu-wamr builds are linked directly into a WAMR host that has no Rust
// firmware crate, so the halt behaviour must live here instead of a downstream binary.
#[cfg(all(not(any(test, feature = "std")), feature = "mcu-wamr"))]
#[panic_handler]
fn halt(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub mod arithmetic_functions;
pub mod bistable_functionblocks;
pub mod bit_num_conversion;
pub mod bit_shift_functions;
pub mod counters;
// FORK(no_std): chrono is not no_std-compatible, these modules only build with `std`.
#[cfg(feature = "std")]
pub mod date_time_conversion;
#[cfg(feature = "std")]
pub mod date_time_extra_functions;
#[cfg(feature = "std")]
pub mod date_time_numeric_functions;
pub mod endianness_conversion_functions;
// FORK(no_std): uses chrono::Local and std::io, only builds with `std`.
#[cfg(feature = "std")]
pub mod extra_functions;
pub mod flanks;
pub mod numerical_functions;
// FORK(no_std): the string modules rely on std's allocating `String`, they are cut from
// mcu-wamr builds (no allocator on the MCU).
#[cfg(feature = "std")]
pub mod string_conversion;
#[cfg(feature = "std")]
pub mod string_functions;
pub mod timers;
pub mod types;
pub mod utils;
pub mod validation_functions;
