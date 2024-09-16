
fn take<const COUNT: usize>(b: &[u8]) -> (&[u8], [u8;COUNT]) {
    (&b[COUNT..], b[..COUNT].try_into().unwrap())
}

fn take_u8(b: &[u8]) -> (&[u8], u8) {
    (&b[1..], b[0])
}

fn take_u16(b: &[u8]) -> (&[u8], u16) {
    (&b[2..], u16::from_le_bytes(b[..2].try_into().unwrap()))
}

fn take_u32(b: &[u8]) -> (&[u8], u32) {
    (&b[4..], u32::from_le_bytes(b[..4].try_into().unwrap()))
}

fn main() {
    let bytes = std::fs::read("../3g_firmware/Firmware-26.9.1.3").unwrap();
    //let bytes = std::fs::read("../5g_firmware/wtf.fw").unwrap();

    let b = &bytes;

    // Parse the header (84/0x54 bytes)
    let (b, magic) = take::<4>(b);
    let (b, version) = take::<3>(b);
    let (b, format) = take_u8(b);
    let (b, entrypoint) = take_u32(b);
    let (b, body_len) = take_u32(b);
    let (b, data_len) = take_u32(b);
    let (b, footer_cert_offset) = take_u32(b);
    let (b, footer_cert_len) = take_u32(b);
    let (b, salt) = take::<32>(b);
    let (b, unk1) = take_u16(b);
    let (b, unk2) = take_u16(b);
    let (b, header_sign) = take::<16>(b);
    let (b, header_left_over) = take::<4>(b);
    
    println!("Magic = {magic:?}");
    println!("Version = {version:?}");
    println!("Format = {format:?}");
    println!("Entrypoint = {entrypoint:?}");
    println!("Body length = {body_len:X}");
    println!("Data length = {data_len:X}");
    println!("Footer cert offset = {footer_cert_offset:X}");
    println!("Footer cert len = {footer_cert_len}");

    // Read rest of padding, offset is now 0x600
    //TODO: this size changes based on magic
    // TODO: wiki says 0x600, iphone wiki say 0x800, file looks very 0x800
    //TODO: wtf looks very 0x600
    //const PADDING_SIZE: usize = 0x800/* 0x600*/ - 0x54;
    //const PADDING_SIZE: usize =  0x600 - 0x54;
    const PADDING_SIZE: usize =  5000 - 0x54;

    let (b, _padding) = take::<{PADDING_SIZE}>(b);

    // After the padding is the body
    /*let body = &b[..body_len as usize];
    let b = &b[body_len as usize..];

    std::fs::write("body.bin", &body);*/

    //TODO: this only applies to x509 `format`
    // Get the certificate signature
    let (b, _cert_sig) = take::<0x80>(b);


 //   let c = &bytes[0x680 + body_len as usize..];
 //   assert_eq!(b, c);

    // Everything else should be the certificate bundle
    
    // Quick check, should start with a ASN.1 sequence
     assert_eq!(&b[..2], &[0x30, 0x82]);
    std::fs::write("./cert_bundle.bin", b);

 //   assert_eq!(data_len, body_len + 0x80 + footer_cert_len);
  //  assert_eq!(bytes.len() as u32, 0x54 + body_len + footer_cert_len + 0x80);
}
