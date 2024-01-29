use std::env::args;
use std::error::Error;
use std::time::Duration;

use std::process::Command;
use std::process::Stdio;

fn main() -> Result<(), Box<dyn Error>> {
    sudo::escalate_if_needed()?;

    if args().len() != 4 {
        println!("cargo r --release -- <start> <size> <out_file>");
        return Ok(());
    }

    let start = u32::from_str_radix(&args().nth(1).unwrap()[2..], 16)?;
    let size = u32::from_str_radix(&args().nth(2).unwrap()[2..], 16)?;
    let out_file = args().nth(3).unwrap();

    const CHUNK_SIZE: u32 = 512;
    let mut full_read = Vec::new();

    let mut addr: u32 = start;
    for _ in progression::bar(0..(size / CHUNK_SIZE)) {
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
        std::fs::write(&out_file, &full_read).unwrap();

        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}
