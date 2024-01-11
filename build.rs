use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=decrypt_shellcode/");
    println!("cargo:rerun-if-changed=scsi_shellcode/");
    println!("cargo:rerun-if-changed=nanoboot/");

    {
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
    }

    {
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

    {
        //std::fs::remove_dir_all("./nanoboot/target").unwrap();
        let _ = std::fs::remove_file("./nanoboot/nanoboot.bin");

        assert!(Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--target")
            .arg("armv6-none-eabi.json")
            .current_dir("./nanoboot/")
            .status()
            .unwrap()
            .success());

        Command::new("arm-none-eabi-objcopy")
            .arg("-O")
            .arg("binary")
            .arg("target/armv6-none-eabi/release/nanoboot")
            .arg("nanoboot.bin")
            .current_dir("./nanoboot/")
            .status()
            .unwrap();
    }
}