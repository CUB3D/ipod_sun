use std::error::Error;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    // First upload the decrypter via scsi
    //sudo sg_raw -i dec.bin -s 40 -v /dev/sda c6 96 01 08 37 98 b4 -vvv
    Command::new("sg_raw")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("-i")
        .arg("dec.bin")
        .arg("-s")
        .arg("40")
        .arg("/dev/sda")
        .arg("c6")
        .arg("96")
        .arg("01")
        .arg("08")
        .arg("37")
        .arg("98")
        .arg("b4")
        .status()
        .unwrap();

    const CHUNK_SIZE: usize = 512;
    const DEC_CHUNK_SZ: usize = 0x100;

    let enc_fw = std::fs::read("n6g_bootloader.img1").unwrap();
    let mut dec_fw = Vec::new();

    let prg = progression::Bar::new(
        enc_fw.len() as u64 + DEC_CHUNK_SZ as u64 * 2,
        progression::Config::unicode(),
    );
    let mut off = 0;
    loop {
        let chk = &enc_fw[off..];

        let chk = if chk.len() > 512 { &chk[..512] } else { chk };

        let mut tmp = vec![0u8; CHUNK_SIZE];
        tmp[..chk.len()].copy_from_slice(chk);

        // Write chunk to mem
        std::fs::write("temp.bin", &tmp).unwrap();

        Command::new("sg_raw")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("-i")
            .arg("temp.bin")
            .arg("-s")
            .arg(&format!("{}", CHUNK_SIZE))
            .arg("/dev/sda")
            .arg("c6")
            .arg("96")
            .arg("01")
            .arg("08")
            .arg("2c")
            .arg("47")
            .arg("54")
            .status()
            .unwrap();

        // Call decryption func
        // sudo sg_raw -vvvv /dev/sda c6 96 03 08 37 98 b5

        Command::new("sg_raw")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("/dev/sda")
            .arg("c6")
            .arg("96")
            .arg("03")
            .arg("08")
            .arg("37")
            .arg("98")
            .arg("b5")
            .status()
            .unwrap();

        // Read decrypted data back
        // sudo sg_raw -o dump.bin -r 64k -v /dev/sda c6 96 02 08 2c 47 54 && xxd dump.bin

        Command::new("sg_raw")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("-o")
            .arg("temp-read.bin")
            .arg("-r")
            .arg(&format!("{}", CHUNK_SIZE))
            .arg("/dev/sda")
            .arg("c6")
            .arg("96")
            .arg("02")
            .arg("08")
            .arg("2c")
            .arg("47")
            .arg("54")
            .status()
            .unwrap();

        let dec = std::fs::read("./temp-read.bin").unwrap();
        let dec = if off == 0 {
            dec[..DEC_CHUNK_SZ + 0x10].to_vec()
        } else {
            dec[0x10..][..DEC_CHUNK_SZ].to_vec()
        };

        dec_fw.extend_from_slice(&dec);

        std::fs::write("./bl_dec.bin", &dec_fw).unwrap();
        std::thread::sleep(Duration::from_millis(1));
        off += DEC_CHUNK_SZ;
        prg.inc(DEC_CHUNK_SZ as _);

        if off > enc_fw.len() {
            break;
        }
    }
    prg.finish();

    Ok(())
}
