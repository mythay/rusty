// FORK(no_std): without std the float methods come from num's libm-backed `Float`.
#[cfg(not(any(test, feature = "std")))]
use num::Float;

/// .
/// Rounds a REAL (f32) value
///
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn ROUND__REAL(input: f32) -> f32 {
    input.round()
}

/// .
/// Rounds a LREAL (f64) value
///
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn ROUND__LREAL(input: f64) -> f64 {
    input.round()
}
