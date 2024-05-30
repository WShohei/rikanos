use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

fn main() {
    let current_dir = env::current_dir().unwrap();
    let out_dir = Path::new(&current_dir).join("lib");
    let out_dir = out_dir.to_str().unwrap();

    build_usb(&out_dir, &current_dir);
    build_asm(&out_dir, &current_dir);

    // It allows Rust to include libusb.a in the elf.
    println!("cargo:rustc-link-search=native={}", out_dir);
}

fn build_asm(out_dir: &str, current_dir: &Path) {
    let asm_dir = Path::new(current_dir).join("asm");
    let o_path = Path::new(asm_dir.to_str().unwrap()).join("asmfunc.o");
    let o_path = o_path.to_str().unwrap();
    
    Command::new("nasm")
        .current_dir(&asm_dir)
        .args(&["-f", "elf64"])
        .arg("-o")
        .arg(Path::new(asm_dir.to_str().unwrap()).join("asmfunc.o"))
        .arg("asmfunc.asm")
        .status()
        .unwrap();

    Command::new("ar")
        .current_dir(out_dir)
        .args(&["crus", "libasmfunc.a", o_path])
        .status()
        .unwrap();

    Command::new("rm")
        .current_dir(out_dir)
        .arg("asmfunc.o")
        .status()
        .unwrap();

    println!("cargo:rerun-if-changed=asm");
    println!("cargo:rustc-link-lib=static=asmfunc");
}

fn build_usb(out_dir: &str, current_dir: &Path) {
    let usb_dir = Path::new(current_dir).join("usb");

    Command::new("make")
        .arg("clean")
        .current_dir(&usb_dir)
        .status()
        .unwrap();
    Command::new("make").current_dir(&usb_dir).status().unwrap();

    fs::copy(
        PathBuf::from(&usb_dir).join("libusb.a"),
        Path::new(out_dir).join("libusb.a"),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=usb");
    println!("cargo:rustc-link-lib=static=usb");
}
