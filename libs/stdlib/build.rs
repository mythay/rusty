use std::env;
use std::path::Path;
use std::process::Command;

// FORK(no_std): these *.st sources declare `{external}` functions whose implementations
// live in std/chrono-only Rust modules, so they are excluded from mcu-wamr builds.
const MCU_EXCLUDED_SOURCES: &[&str] = &[
    "date_time_conversion.st",
    "date_time_extra_functions.st",
    "date_time_numeric_functions.st",
    "extra_functions.st",
    "string_conversion.st",
    "string_functions.st",
    "string_to_conversions.st",
    "to_bit.st",
    "to_date_time.st",
    "to_string.st",
    "to_string_conversions.st",
];

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Collect the IEC sources, leaving out the std-only ones on mcu-wamr builds.
    let exclude_mcu = env::var("CARGO_FEATURE_MCU_WAMR").is_ok();
    let mut sources = Vec::new();
    for entry in Path::new("iec61131-st").read_dir().unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "st")
            && !(exclude_mcu && MCU_EXCLUDED_SOURCES.contains(&path.file_name().unwrap().to_str().unwrap()))
        {
            sources.push(path);
        }
    }
    sources.sort();

    let mut args: Vec<_> = sources.iter().map(|path| path.to_string_lossy().into_owned()).collect();
    args.extend([
        "-c".to_owned(),
        "--generate-external-constructors".to_owned(),
        "-o".to_owned(),
        format!("{out_dir}/st.o"),
    ]);
    if let Ok(target) = env::var("TARGET") {
        args.push("--target".to_owned());
        args.push(target);
    }
    if let Ok(optimization) = env::var("PROFILE") {
        args.push("-O".to_owned());
        if optimization == "release" {
            args.push("default".to_owned());
        } else {
            args.push("none".to_owned());
        }
    }
    // FORK(no_std): keep the wasm artifacts identical no matter which host OS builds
    // them: wasm targets always compile as a single module, so CI (Linux) and a local
    // Windows build produce the same objects; Windows host targets keep upstream's
    // lld-link workaround.
    if env::var("TARGET").is_ok_and(|target| target.starts_with("wasm32")) || cfg!(target_os = "windows") {
        args.push("--single-module".to_owned());
    }

    // FORK(no_std): the IEC compiler runs in-process through the `plc_driver` build
    // dependency for every build, so std and mcu-wamr behave identically and there is no
    // reliance on a `plc` binary in the PATH (whose version can drift from the sources).
    // Point `RUSTY_PLC` at a prebuilt `plc` executable to skip building the compiler
    // toolchain entirely and run that binary instead.
    println!("cargo:rerun-if-env-changed=RUSTY_PLC");
    if let Ok(plc) = env::var("RUSTY_PLC") {
        let status = Command::new(&plc)
            .args(&args)
            .status()
            .unwrap_or_else(|err| panic!("failed to run the IEC compiler `{plc}` ({err})"));
        if !status.success() {
            panic!("the IEC compiler `{plc}` failed while compiling the stdlib sources");
        }
    } else {
        let mut driver_args = vec!["plc".to_owned()];
        driver_args.extend(args.iter().cloned());
        if let Err(err) = plc_driver::compile(&driver_args) {
            panic!("the IEC compiler failed while compiling the stdlib sources: {err:#}");
        }
    }

    // Archive the object file into a static library so rustc can find it.
    // FORK(no_std): the archiver is picked by TARGET, not by the host: wasm objects are
    // always archived with `ar` (e.g. llvm-ar), `lld-link` only handles MSVC host builds.
    let archiver_args: (&str, &[&str]) =
        if env::var("TARGET").is_ok_and(|target| target.starts_with("wasm32")) {
            ("ar", &["crs", "libst.a", "st.o"][..])
        } else {
            #[cfg(target_os = "windows")]
            {
                ("lld-link", &["/LIB", "st.o"][..])
            }
            #[cfg(not(target_os = "windows"))]
            {
                ("ar", &["crs", "libst.a", "st.o"][..])
            }
        };
    Command::new(archiver_args.0).args(archiver_args.1).current_dir(Path::new(&out_dir)).status().unwrap();

    //link the object file
    println!("cargo:rustc-link-search=native={out_dir}");
    println!("cargo:rerun-if-changed=iec61131-st/");
    println!("cargo:rustc-link-lib=static:+whole-archive=st"); //create a whole archive without throwing away any object files (https://github.com/rust-lang/rust/pull/105601).
}
