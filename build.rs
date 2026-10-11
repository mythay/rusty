// FORK(no_std): this build script does two things, both only in `wasm-only` builds.
//
// 1. Compiles `llvm_target_stubs.c` (see that file): no-op stand-ins for the LLVM target
//    initializers of the backends those builds do not link.
// 2. Picks the LLVM libraries to link. With the `wasm-only` feature inkwell runs llvm-sys
//    with `no-llvm-linking`, so llvm-sys emits no link directives at all; instead we ask
//    the real llvm-config for its static library list and re-emit it minus every backend
//    except WebAssembly. That keeps the SDK compiler binary at ~35 MB instead of ~122 MB,
//    at the cost of only being able to emit code for wasm32. Re-add a backend by removing
//    it from `EXCLUDED_PREFIXES` and from the stub file.
//
// Regular builds compile and link like upstream, through llvm-sys itself.
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const EXCLUDED_PREFIXES: &[&str] = &[
    // target backends other than WebAssembly
    "LLVMAArch64",
    "LLVMAMDGPU",
    "LLVMARM",
    "LLVMAVR",
    "LLVMBPF",
    "LLVMHexagon",
    "LLVMLanai",
    "LLVMLoongArch",
    "LLVMMips",
    "LLVMMSP430",
    "LLVMNVPTX",
    "LLVMPowerPC",
    "LLVMRISCV",
    "LLVMSparc",
    "LLVMSPIRV",
    "LLVMSystemZ",
    "LLVMVE",
    "LLVMXCore",
    "LLVMX86",
    // tool front-end libraries nothing here links
    "LLVMWindowsManifest",
    "LLVMXRay",
    "LLVMLibDriver",
    "LLVMDlltoolDriver",
    "LLVMTelemetry",
    "LLVMCoverage",
    "LLVMLineEditor",
    "LLVMTextAPIBinaryReader",
];

fn main() {
    let wasm_only = env::var_os("CARGO_FEATURE_WASM_ONLY").is_some();
    if !wasm_only {
        return;
    }

    cc::Build::new().file("llvm_target_stubs.c").compile("llvm_target_stubs");
    println!("cargo:rerun-if-changed=llvm_target_stubs.c");

    let llvm_config = locate_llvm_config().unwrap_or_else(|| {
        panic!(
            "llvm-config not found; set LLVM_SYS_211_PREFIX to an LLVM 21.1 installation \
             (like the PLC-lang llvm-package-windows release) or put it on the PATH"
        )
    });
    println!("cargo:rerun-if-env-changed=LLVM_SYS_211_PREFIX");

    // The library list keeps llvm-config's own order, which is safe for static linking.
    let libdir = run(&llvm_config, &["--libdir"]);
    println!("cargo:rustc-link-search=native={}", libdir.trim());

    let libs = run(&llvm_config, &["--libnames", "--link-static"]);
    for name in libs.split(&[' ', '\r', '\n']).filter(|name| !name.is_empty()) {
        let stem = name.rsplit(['\\', '/']).next().unwrap();
        let stem = stem.strip_suffix(".lib").unwrap_or(stem);
        if EXCLUDED_PREFIXES.iter().any(|prefix| stem.starts_with(prefix)) {
            continue;
        }
        println!("cargo:rustc-link-lib=static={stem}");
    }

    let system_libs = run(&llvm_config, &["--system-libs", "--link-static"]);
    for name in system_libs.split(&[' ', '\r', '\n']).filter(|name| !name.is_empty()) {
        let stem = name.strip_suffix(".lib").unwrap_or(name);
        println!("cargo:rustc-link-lib={stem}");
    }
}

// Mirrors llvm-sys's lookup so both agree on the same installation.
fn locate_llvm_config() -> Option<PathBuf> {
    if let Some(prefix) = env::var_os("LLVM_SYS_211_PREFIX") {
        let candidate =
            PathBuf::from(prefix).join("bin").join(format!("llvm-config{}", env::consts::EXE_SUFFIX));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    if Command::new("llvm-config").arg("--version").output().is_ok() {
        return Some(PathBuf::from("llvm-config"));
    }
    None
}

fn run(llvm_config: &Path, args: &[&str]) -> String {
    let output = Command::new(llvm_config)
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", llvm_config.display()));
    if !output.status.success() {
        panic!("llvm-config {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr));
    }
    String::from_utf8_lossy(&output.stdout).into_owned()
}
