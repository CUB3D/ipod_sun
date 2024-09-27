use crate::Device;
use parse::{le_u32, take};
use tracing::trace;

#[derive(Clone, Default, Debug)]
pub struct MseSection {
    tag: [u8; 4],
    pub name: [u8; 4],

    _idk: u32,
    dev_offset: u32,
    length: u32,
    address: u32,
    entry_offset: u32,
    _idk2: u32,
    version: u32,
    load_addr: u32,

    head: Vec<u8>,
    pub body: Vec<u8>,
}

pub struct Mse {
    header: [u8; 0x5000],
    pub sections: Vec<MseSection>,
}
impl Mse {
    pub fn write(&self, file: &mut Vec<u8>) {
        file.extend_from_slice(&self.header);

        for sec in &self.sections {
            file.extend_from_slice(&sec.tag);
            file.extend_from_slice(&sec.name);
            file.extend_from_slice(&sec._idk.to_le_bytes());
            file.extend_from_slice(&sec.dev_offset.to_le_bytes());
            file.extend_from_slice(&sec.length.to_le_bytes());
            file.extend_from_slice(&sec.address.to_le_bytes());
            file.extend_from_slice(&sec.entry_offset.to_le_bytes());
            file.extend_from_slice(&sec._idk2.to_le_bytes());
            file.extend_from_slice(&sec.version.to_le_bytes());
            file.extend_from_slice(&sec.load_addr.to_le_bytes());
        }

        for _ in 0..(16 - self.sections.len()) {
            file.extend_from_slice(&[0x0; 36]);
            file.extend_from_slice(&[0xFF; 4]);
        }

        let mut sections = self.sections.clone();
        sections.sort_by_key(|a| a.dev_offset);

        for sec in &sections {
            while (file.len() as u64) < sec.dev_offset as u64 {
                file.push(0);
            }

            file.extend_from_slice(&sec.head);
            file.extend_from_slice(&sec.body);
        }

        while file.len() % 0x1000 != 0 {
            file.push(0);
        }
    }
}

pub fn unpack(path: &str, device: &Device) -> Mse {
    let firm_data = std::fs::read(path).unwrap();

    let (f, header) = take::<0x5000>(&firm_data);

    let mut i = f;

    let mut sections = Vec::new();

    for _idx in 0..16 {
        let (f, tag) = take::<4>(i);

        if tag == [0, 0, 0, 0] {
            let (f, _) = take::<40>(i);
            i = f;
            continue;
        }

        let (f, name) = take::<4>(f);

        let (f, _idk) = le_u32(f);
        let (f, dev_offset) = le_u32(f);
        let (f, length) = le_u32(f);
        let (f, address) = le_u32(f);
        let (f, entry_offset) = le_u32(f);
        let (f, _idk2) = le_u32(f);
        let (f, version) = le_u32(f);
        let (_f, load_addr) = le_u32(f);

        trace!(
            "name = {:?}, len = {:x}, off={}, end={:x}, entry={:x}, address={:x}",
            String::from_utf8(name.to_vec()),
            length,
            dev_offset,
            dev_offset + length + 0x800,
            entry_offset,
            address
        );

        let data_len = match device {
            Device::Nano6 => (length + 0x800) as usize,
            Device::Nano7Refresh => length as usize,
            Device::Nano7 => length as usize,
        };

        let section_header = &firm_data[dev_offset as usize..][..0x1000];
        let section_data = &firm_data[(dev_offset + 0x1000) as usize..];
        let section_data = &section_data[..data_len];

        std::fs::write(&format!("./tmp-{:?}.bin", name.iter().rev().map(|s| *s as char).collect::<String>()), section_data).unwrap();

        let sec = MseSection {
            tag,
            name,
            _idk,
            dev_offset,
            length,
            address,
            entry_offset,
            _idk2,
            version,
            load_addr,
            head: section_header.to_vec(),
            body: section_data.to_vec(),
        };

        sections.push(sec);

        let (f, _) = take::<40>(i);
        i = f;
    }

    Mse { header, sections }
}
