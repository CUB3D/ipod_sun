use std::error::Error;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

const CODE_ADDR_N6G: [&'static str; 4] = ["08", "37", "98", "b4"];
const CALL_ADDR_N6G: [&'static str; 4] = ["08", "37", "98", "b5"];
const INPUT_ADDR_N6G: [&'static str; 4] = ["08", "2c", "47", "54"];

const CODE_ADDR_N7G: [&'static str; 4] = ["08", "49", "69", "6c"];
const CALL_ADDR_N7G: [&'static str; 4] = ["08", "49", "69", "6d"];
const INPUT_ADDR_N7G: [&'static str; 4] = ["08", "49", "2a", "50"];

const CODE_ADDR: [&'static str; 4] = CODE_ADDR_N7G;
const INPUT_ADDR: [&'static str; 4] = INPUT_ADDR_N7G;
const CALL_ADDR: [&'static str; 4] = CALL_ADDR_N7G;

fn main() -> Result<(), Box<dyn Error>> {
    sudo::escalate_if_needed()?;

    // First upload the decrypter via scsi
    //sudo sg_raw -i dec.bin -s 40 -v /dev/sda c6 96 01 08 37 98 b4 -vvv
    Command::new("sg_raw")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("-i")
        .arg("dec.bin")
        .arg("-s")
        .arg("44")
        .arg("/dev/sdc")
        .arg("c6")
        .arg("96")
        .arg("01")
        .arg(CODE_ADDR[0])
        .arg(CODE_ADDR[1])
        .arg(CODE_ADDR[2])
        .arg(CODE_ADDR[3])
        .status()
        .unwrap();

    const CHUNK_SIZE: usize = 512;
    const DEC_CHUNK_SZ: usize = 0x100;

    let enc_fw = std::fs::read("n7g_bootloader.img1").unwrap();
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
            .arg("/dev/sdc")
            .arg("c6")
            .arg("96")
            .arg("01")
            .arg(INPUT_ADDR[0])
            .arg(INPUT_ADDR[1])
            .arg(INPUT_ADDR[2])
            .arg(INPUT_ADDR[3])
            .status()
            .unwrap();

        // Call decryption func
        // sudo sg_raw -vvvv /dev/sda c6 96 03 08 37 98 b5

        Command::new("sg_raw")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("/dev/sdc")
            .arg("c6")
            .arg("96")
            .arg("03")
            .arg(CALL_ADDR[0])
            .arg(CALL_ADDR[1])
            .arg(CALL_ADDR[2])
            .arg(CALL_ADDR[3])
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
            .arg("/dev/sdc")
            .arg("c6")
            .arg("96")
            .arg("02")
            .arg(INPUT_ADDR[0])
            .arg(INPUT_ADDR[1])
            .arg(INPUT_ADDR[2])
            .arg(INPUT_ADDR[3])
            .status()
            .unwrap();

        let dec = std::fs::read("./temp-read.bin").unwrap();
        let dec = if off == 0 {
            dec[..DEC_CHUNK_SZ + 0x10].to_vec()
        } else {
            dec[0x10..][..DEC_CHUNK_SZ].to_vec()
        };

        dec_fw.extend_from_slice(&dec);

        std::fs::write("./n7g_bootloader.img1.dec", &dec_fw).unwrap();
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
