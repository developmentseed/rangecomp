//! Enables conversions from the `core::range` types on Rust 1.96 and later.

use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let Some(version) = rustc_version() else {
        return;
    };
    let minor = minor_version(&version).unwrap_or(0);
    if minor >= 80 {
        println!("cargo:rustc-check-cfg=cfg(core_range)");
    }
    // A 1.96 nightly may predate the types' stabilization, so only trust
    // 1.96 releases and betas, or any later version.
    let is_nightly = version.contains("nightly") || version.contains("dev");
    if minor > 96 || (minor == 96 && !is_nightly) {
        println!("cargo:rustc-cfg=core_range");
    }
}

/// The output of `rustc --version`, such as `rustc 1.96.0 (abc123 2026-05-28)`.
fn rustc_version() -> Option<String> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    String::from_utf8(output.stdout).ok()
}

/// The minor version, such as 96 for `rustc 1.96.0`.
fn minor_version(version: &str) -> Option<u32> {
    version.split('.').nth(1)?.parse().ok()
}
