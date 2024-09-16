use std::time::Duration;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread::sleep;
use libusb::{Direction, Recipient, request_type, RequestType};
use log::info;
use tracing::Level;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // builds the subscriber.
        .init();

    let context = libusb::Context::new().unwrap();
    let mut idx: u32 = 0;
    const BASE_ADDR: u32 = 0x0800_0000;
    const BASE_OFFSET: u32 = 0x6ee0;


    loop {
            for device in context.devices().unwrap().iter() {
                let device_desc = device.device_descriptor().unwrap();

                if device_desc.vendor_id() == 0x05ac && [0x1266].contains(&device_desc.product_id()) {

                    let man = device_desc.manufacturer_string_index().unwrap();
                    let d = device.open().unwrap();
                    let l = d.read_languages(Duration::from_secs(1)).unwrap();

                    let mut buf = [0u8; 256];

                    let _len = d.read_control(request_type(Direction::In, RequestType::Standard, Recipient::Device),
                                             0x06,
                                             (0x3_u16) << 8 | man as u16,
                                             l.first().unwrap().lang_id(),
                                             &mut buf,
                                             Duration::from_millis(500)).unwrap();


                    let buf = buf.chunks(2).skip(1).map(|f| f[0]).collect::<Vec<u8>>();
                    //println!("dat = {buf:x?}");

                    // let mut f = OpenOptions::new().create(true).append(true).write(true).open("./buf-append.bin").unwrap();
                    // f.write(&buf[1..2]).unwrap();
                    // drop(f);

                    let mut out_buf = std::fs::read("./out.bin").unwrap();

                    let b0 = buf[1];

                    out_buf[idx as usize + BASE_OFFSET as usize] = b0;
                    info!("*(0x{:08X}) = {:02x}, idx = {idx}", BASE_ADDR + BASE_OFFSET + idx, b0);
                    idx += 1;

                    // If first byte is non-null then there will be a second byte
                    if b0 != 0 {
                        let b1 = buf[2];

                        out_buf[idx as usize + BASE_OFFSET as usize] = b1;
                        info!("*(0x{:08X}) = {:02x}, idx = {idx}", BASE_ADDR + BASE_OFFSET + idx, b1);
                        idx += 1;

                        // If second byte is non-null then there will be a third byte
                        if b1 != 0 {
                            let b2 = buf[3];

                            out_buf[idx as usize + BASE_OFFSET as usize] = b2;
                            info!("*(0x{:08X}) = {:02x}, idx = {idx}", BASE_ADDR + BASE_OFFSET + idx, b2);
                            idx += 1;

                            if b2 != 0 {
                                let b3 = buf[4];

                                out_buf[idx as usize + BASE_OFFSET as usize] = b3;
                                info!("*(0x{:08X}) = {:02x}, idx = {idx}", BASE_ADDR + BASE_OFFSET + idx, b3);
                                idx += 1;

                                if b3 != 0 {
                                    let b4 = buf[5];

                                    out_buf[idx as usize + BASE_OFFSET as usize] = b4;
                                    info!("*(0x{:08X}) = {:02x}, idx = {idx}", BASE_ADDR + BASE_OFFSET + idx, b4);
                                    idx += 1;

                                    if b4 != 0 {
                                        let b5 = buf[6];

                                        out_buf[idx as usize + BASE_OFFSET as usize] = b5;
                                        info!("*(0x{:08X}) = {:02x}, idx = {idx}", BASE_ADDR + BASE_OFFSET + idx, b5);
                                        idx += 1;

                                        if b5 != 0 {
                                            let b6 = buf[7];

                                            out_buf[idx as usize + BASE_OFFSET as usize] = b6;
                                            info!("*(0x{:08X}) = {:02x}, idx = {idx}", BASE_ADDR + BASE_OFFSET + idx, b6);
                                            idx += 1;

                                            if b6 != 0 {
                                                let b7 = buf[8];

                                                out_buf[idx as usize + BASE_OFFSET as usize] = b7;
                                                info!("*(0x{:08X}) = {:02x}, idx = {idx}", BASE_ADDR + BASE_OFFSET + idx, b7);
                                                idx += 1;

                                                if b7 != 0 {
                                                    let b8 = buf[9];

                                                    out_buf[idx as usize + BASE_OFFSET as usize] = b8;
                                                    info!("*(0x{:08X}) = {:02x}, idx = {idx}", BASE_ADDR + BASE_OFFSET + idx, b8);
                                                    idx += 1;

                                                    if b8 != 0 {
                                                        let b9 = buf[10];

                                                        out_buf[idx as usize + BASE_OFFSET as usize] = b9;
                                                        info!("*(0x{:08X}) = {:02x}, idx = {idx}", BASE_ADDR + BASE_OFFSET + idx, b9);
                                                        idx += 1;

                                                        if b9 != 0 {
                                                            let b10 = buf[11];

                                                            out_buf[idx as usize + BASE_OFFSET as usize] = b10;
                                                            info!("*(0x{:08X}) = {:02x}, idx = {idx}", BASE_ADDR + BASE_OFFSET + idx, b10);
                                                            idx += 1;

                                                        }

                                                    }

                                                }

                                            }

                                        }

                                    }

                                }

                            }

                        }

                    }

                    std::fs::write("./out.bin", &out_buf).unwrap();

                    info!("Unbind");
                    std::fs::write("/sys/bus/usb/drivers/usb/unbind", "1-3").unwrap();
                    sleep(Duration::from_millis(4500));
                    info!("Bind");
                    std::fs::write("/sys/bus/usb/drivers/usb/bind", "1-3").unwrap();
                    sleep(Duration::from_millis(4500));

                    break;
                }
            };
        }

    Ok(())

}
