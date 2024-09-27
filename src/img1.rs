use crate::Device;
use parse::{le_u16, le_u32, ne_u8, take, take_n};
use tracing::trace;

pub struct Img1 {
    head: Vec<u8>,
    pub body: Vec<u8>,
    pub padding: [u8; 940],
    pub cert: Vec<u8>,
}

impl Img1 {
    pub fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.head);
        out.extend_from_slice(&self.padding);
        out.extend_from_slice(&self.body);
        out.extend_from_slice(&self.cert);
    }
}

pub fn img1_parse(orig_data: &[u8], device: &Device) -> Img1 {
    // Parse the header (84/0x54 bytes)
    let (b, magic) = take::<4>(orig_data);
    let (b, version) = take::<3>(b);
    let (b, format) = ne_u8(b);
    let (b, entrypoint) = le_u32(b);
    let (b, body_len) = le_u32(b);
    let (b, data_len) = le_u32(b);
    let (b, footer_cert_offset) = le_u32(b);
    let (b, footer_cert_len) = le_u32(b);
    let (b, _salt) = take::<32>(b);
    let (b, _unk1) = le_u16(b);
    let (b, _unk2) = le_u16(b);
    let (b, _header_sign) = take::<16>(b);
    let (b, _header_left_over) = take::<4>(b);

    trace!("Magic = {magic:?}");
    trace!("Version = {version:?}");
    trace!("Format = {format:?}");
    trace!("Entrypoint = {entrypoint:?}");
    trace!("Body length = {body_len:X}");
    trace!("Data length = {data_len:X}");
    trace!("Footer cert offset = {footer_cert_offset:X}");
    trace!("Footer cert len = {footer_cert_len}");

    // Read rest of padding, offset is now 0x600
    //TODO: this size changes based on magic
    // TODO: wiki says 0x600, iphone wiki say 0x800, file looks very 0x800
    // RSRC is 0x400
    const PADDING_SIZE: usize = 0x400/* 0x600*/ - 0x54;

    let (b, padding) = take::<{ PADDING_SIZE }>(b);

    // After the padding is the body
    let (b, body) = take_n(b, body_len as usize);

    let cert = match device {
        Device::Nano6 => Vec::new(),
        Device::Nano7Refresh => b.to_vec(),
        Device::Nano7 => b.to_vec(),
    };

    Img1 {
        head: orig_data[..0x54].to_vec(),
        padding,
        body: body.to_vec(),
        cert,
    }
}
