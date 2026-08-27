use std::process::Command;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Use local zig from build_deps if present
    let local_zig = format!("{}/build_deps/zig-windows-x86_64-0.13.0/zig.exe", manifest_dir);
    let zig_cmd = if std::path::Path::new(&local_zig).exists() {
        &local_zig
    } else {
        "zig"
    };

    // 1. Compile sumerian_cuda_core.zig to static library
    // We do NOT link cuda or nvrtc at compile-time since they are dynamically loaded by Zig at runtime
    let zig_status = Command::new(zig_cmd)
        .args(&[
            "build-lib",
            "sumerian_cuda_core.zig",
            "-O", "ReleaseFast",
            "-lc",
            "-target",
            "x86_64-windows-msvc",
            &format!("-femit-bin={}/sumerian_cuda_core.lib", out_dir),
        ])
        .current_dir(&manifest_dir)
        .status();

    if zig_status.is_ok() && zig_status.unwrap().success() {
        println!("cargo:rustc-link-search=native={}", out_dir);
    } else {
        println!("cargo:warning=Zig compilation failed or Zig was not found. Seeking precompiled sumerian_cuda_core.lib in workspace root.");
        println!("cargo:rustc-link-search=native={}", manifest_dir);
    }

    // Link the Zig static library
    println!("cargo:rustc-link-lib=static=sumerian_cuda_core");

    // Tell cargo where to find LibTorch import libraries
    if std::env::var("LIBTORCH_USE_PYTORCH").is_err() && std::env::var("LIBTORCH").is_err() {
        println!("cargo:rustc-link-search=native={}/build_deps/libtorch/lib", manifest_dir);
    }
    
    // Rerun build script only if files change
    println!("cargo:rerun-if-changed=sumerian_cuda_core.zig");
}
