# StandardFunctions
Standard functions for PLC as described by IEC61131

## Structure
The project is managed as a rust project but contains sections implemented directly in ST

### ST
The ST implementation is used for simple functions that are described by the IEC61131-3 standard
It can be compiled using [RuSTy](https://github.com/PLC-lang/rusty)

### Rust
The Rust implementation contains the more complex functions required by the IEC61131-3 standard
These include string or array functions as well as function blocks dealing with Time or other system calls

## Tests
Testing the library is done using rust, the ST parts of the tests are achieved using ruSTy as a library
To test an ST based function, you can use the `use_std` macro included in the tests' `common` module
Rust based ST functions cannot be tested yet, this will be extended later

## Fork differences (no_std / mcu-wamr)

This fork builds the library for WAMR (WebAssembly Micro Runtime) on MCUs in addition to the
regular host builds. Every divergence from upstream is marked with a `FORK(no_std):` comment;
this section is the index of all of them.

Build matrix:

| Target | Command | Notes |
| ------ | ------- | ----- |
| host (std) | `cargo build` / `cargo test` | identical to upstream, full module set |
| WAMR / MCU (release) | `cargo mcu-release` (inside this directory) | `--no-default-features --features mcu-wamr --target wasm32-unknown-unknown`, size settings inlined via `--config` so the workspace `[profile.release]` stays untouched |
| WAMR / MCU (debug) | `cargo mcu-debug` (inside this directory) | same flags, default dev profile with debug info |

Features:

- `std` (default): gates every module that needs std or chrono.
- `mock_time`: upstream test feature, swaps the timer clock for a controllable mock.
- `mcu-wamr`: the MCU build switch. Implies no_std and a halting panic handler (the WAMR
  host links `libst.a` directly, there is no Rust firmware crate to own it).

Divergence points:

- `src/lib.rs`: `no_std` / panic handler / module gating (`date_time_*`, `extra_functions`,
  `string_*` are `std`-only).
- `src/timers.rs` + `src/timers/test_time_helpers.rs`: the mock clock doubles as the
  production clock on mcu-wamr builds; the WAMR host advances it once per PLC cycle by
  calling `__mock_time_advance_ns`. Date and time functions are not available on MCU.
  Also carries an upstream-worthy bug fix: the timer preset edge compares with `<`
  instead of `<=`.
- `src/arithmetic_functions.rs`, `src/numerical_functions.rs`: float math goes through
  num's libm-backed `Float` on no_std builds.
- `build.rs`: std builds compile the IEC sources in-process through the `plc_driver`
  build dependency (like upstream); mcu-wamr builds invoke an external `plc` binary
  (override with the `PLC` environment variable) and exclude the std-only
  `iec61131-st/*.st` sources.
- `Cargo.toml`: self dev-dependency so integration tests get the mock clock exports.
- `.cargo/config.toml` (`cargo mcu-release` / `cargo mcu-debug` aliases) and
  `.github/workflows/mcu-wasm.yml` (keeps the wasm build path green).

There is no allocator on the MCU: nothing in a mcu-wamr build may depend on `alloc`.
