use std::error::Error;
use std::time::Duration;

use std::process::Command;
use std::process::Stdio;

fn main() -> Result<(), Box<dyn Error>> {
    sudo::escalate_if_needed()?;
    const START_ADDR: u32 = 0x0800_0000;
    const CHUNK_SIZE: u32 = 512;

    let mut addr = START_ADDR;

    let file = std::fs::read("../../nanoboot/nanoboot.bin")?;

    for chunk in progression::bar(file.chunks(CHUNK_SIZE as usize)) {
        std::fs::write("./send.bin", &chunk)?;


        //println!("Reading {addr:08x}={:08x}, prog", addr+0x200);
        let b: [u8; 4] = addr.to_be_bytes();

        Command::new("sg_raw")
            // .stdout(Stdio::null())
            // .stderr(Stdio::null())
            .arg("-s")
            .arg("send.bin")
            .arg("-r")
            .arg(&format!("{}", chunk.len()))
            .arg("-vvv")
            .arg("/dev/sdc")
            .arg("c6")
            .arg("96")
            .arg("01")
            .arg(&format!("{:02x}", b[0]))
            .arg(&format!("{:02x}", b[1]))
            .arg(&format!("{:02x}", b[2]))
            .arg(&format!("{:02x}", b[3]))
            .status()
            .unwrap();

        addr += CHUNK_SIZE;

        std::thread::sleep(Duration::from_millis(1));
    }

    let b: [u8; 4] = START_ADDR.to_be_bytes();

    Command::new("sg_raw")
        // .stdout(Stdio::null())
        // .stderr(Stdio::null())
        .arg("-o")
        .arg("null.bin")
        .arg("-r")
        .arg(&format!("{}", 512))
        .arg("-vvv")
        .arg("/dev/sdc")
        .arg("c6")
        .arg("96")
        .arg("03")
        .arg(&format!("{:02x}", b[0]))
        .arg(&format!("{:02x}", b[1]))
        .arg(&format!("{:02x}", b[2]))
        .arg(&format!("{:02x}", b[3]))
        .status()
        .unwrap();

    Ok(())
}
