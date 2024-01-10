#![allow(unused_macros)]
#![allow(dead_code)]
mod cff;
mod exploit;
mod img1;
mod mse;
mod payload;

use crate::exploit::create_cff;
use crate::payload::exploit_config::{ExploitConfigN6G, ExploitConfigN7G};
use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use isahc::ReadResponseExt;
use std::io::{Cursor, Read};
use std::path::Path;
use std::process::Command;
use tracing::{info, Level};
use zip::ZipArchive;

#[derive(Debug, ValueEnum, Copy, Clone)]
pub enum Device {
    Nano6,
    Nano7Refresh,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Which device to build a payload for
    #[arg(short, long)]
    device: Device,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let args = Args::parse();

    // Generate exploit font
    info!("Building CFF exploit");
    let bytes = match args.device {
        Device::Nano6 => create_cff::<ExploitConfigN6G>()?,
        Device::Nano7Refresh => create_cff::<ExploitConfigN7G>()?,
    };

    std::fs::write("./in-cff.bin", bytes)?;

    info!("Converting font to OTF");
    Command::new("python")
        .arg("./helpers/cff_to_otf.py")
        .status()
        .unwrap();

    std::fs::remove_file("./in-cff.bin")?;
    let otf_bytes = std::fs::read("./out-otf.bin")?;
    std::fs::remove_file("./out-otf.bin")?;

    info!("Unpacking MSE");
    let mut mse = if let Device::Nano6 = args.device {
        if !Path::new("./Firmware-36B10147.MSE").try_exists()? {
            let mut ipsw = isahc::get("http://appldnld.apple.com/iPod/SBML/osx/bundles/041-1920.20111004.CpeEw/iPod_1.2_36B10147.ipsw")?;
            let mut zip = ZipArchive::new(Cursor::new(ipsw.bytes().unwrap()))?;
            let mut mse = zip.by_name("Firmware.MSE")?;
            let mut out = Vec::new();
            mse.read_to_end(&mut out)?;
            std::fs::write("./Firmware-36B10147.MSE", &out)?;
        }

        mse::unpack("./Firmware-36B10147.MSE", &args.device)
    } else {
        if !Path::new("./Firmware-39A10023.MSE").try_exists()? {
            let mut ipsw = isahc::get("http://appldnld.apple.com/ipod/sbml/osx/bundles/031-59796-20160525-8E6A5D46-21FF-11E6-89D1-C5D3662719FC/iPod_1.1.2_39A10023.ipsw")?;
            let mut zip = ZipArchive::new(Cursor::new(ipsw.bytes().unwrap()))?;
            let mut mse = zip.by_name("Firmware.MSE")?;
            let mut out = Vec::new();
            mse.read_to_end(&mut out)?;
            std::fs::write("./Firmware-39A10023.MSE", &out)?;
        }

        mse::unpack("./Firmware-39A10023.MSE", &args.device)
    };

    let rsrc = mse
        .sections
        .iter_mut()
        .find(|s| &s.name == b"crsr")
        .ok_or(anyhow!("Failed to find rsrc section in MSE"))?;
    {
        info!("Unpacking RSRC Img1");
        let mut img1 = img1::img1_parse(&rsrc.body, &args.device);
        {
            info!("Patching FATFS");
            std::fs::write("rsrc.bin", &img1.body)?;
            std::fs::write("in-otf.bin", otf_bytes)?;

            Command::new("python")
                .arg("./helpers/fat_patch.py")
                .status()?;

            let rsrc_data = std::fs::read("./rsrc.bin")?;
            std::fs::remove_file("./rsrc.bin")?;
            std::fs::remove_file("./in-otf.bin")?;
            img1.body = rsrc_data;
        }
        info!("Repacking RSRC Img1");
        rsrc.body.clear();
        img1.write(&mut rsrc.body);
    }

    info!("Repacking MSE");
    let mut mse_out = Vec::new();
    mse.write(&mut mse_out);

    // Disk swap
    info!("Doing disk swap");

    if let Device::Nano6 = args.device {
        mse_out[0x5004..][..4].copy_from_slice(b"soso");
        mse_out[0x5144..][..4].copy_from_slice(b"ksid");
    } else {
        mse_out[0x5004..][..4].copy_from_slice(b"soso");
        mse_out[0x5194..][..4].copy_from_slice(b"ksid");
    }

    std::fs::write("./Firmware-repack.MSE", &mse_out)?;

    Ok(())
}
