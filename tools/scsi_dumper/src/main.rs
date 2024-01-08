use std::error::Error;
use std::time::Duration;

use std::process::Command;
use std::process::Stdio;

fn main() -> Result<(), Box<dyn Error>> {
    sudo::escalate_if_needed()?;
    const START_ADDR: u32 = 0x0800_0000;
    const END_ADDR: u32 = 0x0b00_0000;

    const CHUNK_SIZE: u32 = 512;
    let mut full_read = Vec::new();

    let mut addr: u32 = 0x0880_0000;
    for _ in progression::bar(0..((END_ADDR - START_ADDR) / CHUNK_SIZE)) {
        //println!("Reading {addr:08x}={:08x}, prog", addr+0x200);
        let b: [u8; 4] = addr.to_be_bytes();

        Command::new("sg_raw")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("-o")
            .arg("out.bin")
            .arg("-r")
            .arg(&format!("{CHUNK_SIZE}"))
            .arg("-v")
            .arg("/dev/sdc")
            .arg("c6")
            .arg("96")
            .arg("02")
            .arg(&format!("{:02x}", b[0]))
            .arg(&format!("{:02x}", b[1]))
            .arg(&format!("{:02x}", b[2]))
            .arg(&format!("{:02x}", b[3]))
            .status()
            .unwrap();

        let chunk = std::fs::read("./out.bin").unwrap();
        full_read.extend_from_slice(&chunk);

        addr += 0x200;
        std::fs::write("./full_dump.bin", &full_read).unwrap();

        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}
