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
- `build.rs`: every build compiles the IEC sources in-process through the `plc_driver`
  build dependency (like upstream), so std and mcu-wamr behave identically; point
  `RUSTY_PLC` at a prebuilt `plc` binary to skip building the compiler toolchain.
  mcu-wamr builds exclude the std-only `iec61131-st/*.st` sources.
- `Cargo.toml`: self dev-dependency so integration tests get the mock clock exports.
- `.cargo/config.toml` (`cargo mcu-release` / `cargo mcu-debug` aliases) and
  `.github/workflows/mcu-wasm.yml` (keeps the wasm build path green).
- Root build, `wasm-only` feature: SDK builds link only the WebAssembly backend.
  inkwell runs llvm-sys with `no-llvm-linking`, and the root `build.rs` re-emits
  llvm-config's static library list minus the other backends; `llvm_target_stubs.c`
  no-ops the dropped targets' initializers, and `src/codegen.rs` initializes
  WebAssembly instead of all targets. Build the SDK compiler with
  `cargo build --release --features plc-compiler/wasm-only`. Regular builds are
  upstream-equivalent (all backends, working host JIT); compiling for any non-wasm
  triple in a wasm-only build is a fork limitation.

There is no allocator on the MCU: nothing in a mcu-wamr build may depend on `alloc`.

## Syncing with upstream (PLC-lang/rusty)

Branch layout: `master-wamr` is the fork's mainline (upstream master + the fork
delta, ~10 commits). Upstream release branches are **not** pre-emptively mirrored.

| Situation | Action |
| --------- | ------ |
| upstream cuts a release (e.g. v1.1.0) | `git fetch upstream --tags && git merge v1.1.0` into the mainline; the delta is small, and the builtins-rework adaptation is already done |
| an upstream fix is needed before the next release | `git cherry-pick <commit>` onto the mainline |
| a fork fix is upstream-worthy | small branch + PR to PLC-lang/rusty; take it back through the merge — a smaller delta makes every future sync cheaper |
| a consumer is pinned to an old release line and needs a patch | only then branch from that tag (e.g. `v1.0.6`) and cherry-pick the delta; note the 1.0.x codebase predates the builtins rework, so the port needs real adaptation work |

The SDK artifact is built from the mainline (`cargo build --release --features
plc-compiler/wasm-only`); its version string shows the upstream crate version
(`1.1.0-dev`) plus the fork commit hash, which identifies the actual source.
