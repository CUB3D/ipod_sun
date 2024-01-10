use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=decrypt_shellcode/");
    println!("cargo:rerun-if-changed=scsi_shellcode/");

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("thumbv6m-none-eabi")
        .current_dir("./decrypt_shellcode/")
        .status()
        .unwrap();

    Command::new("arm-none-eabi-objcopy")
        .arg("-O")
        .arg("binary")
        .arg("target/thumbv6m-none-eabi/release/scsi_shellcode")
        .arg("scsi-stub.bin")
        .current_dir("./decrypt_shellcode/")
        .status()
        .unwrap();

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("thumbv6m-none-eabi")
        .current_dir("./scsi_shellcode/")
        .status()
        .unwrap();

    Command::new("arm-none-eabi-objcopy")
        .arg("-O")
        .arg("binary")
        .arg("target/thumbv6m-none-eabi/release/scsi_shellcode")
        .arg("scsi-stub.bin")
        .current_dir("./scsi_shellcode/")
        .status()
        .unwrap();
}