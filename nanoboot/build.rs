use std::process::Command;

fn main() {
    // Required because .cargo/config.toml is ignored when building a project from main build.rs
    println!("cargo:rustc-link-arg=-nostdlib");
    println!("cargo:rustc-link-arg=-static");
    println!("cargo:rustc-link-arg=-Wl,-Tshellcode.ld");
}