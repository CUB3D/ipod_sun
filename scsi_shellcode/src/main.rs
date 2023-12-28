#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}


#[repr(C)]
pub struct ScsiResponse {
    a: u8,
    b: u8,
    ascq: u16,
}

#[repr(C)]
pub struct ScsiPacket {
    vtbl: *mut ScsiPacketVtable
}

#[repr(C)]
pub struct ScsiPacketVtable {
    unk1: u32,
    unk2: u32,
    get_buffer: extern "C" fn(*mut ScsiPacket) -> *mut u8,
    rec: u32,
    send: extern "C" fn (*mut ScsiPacket, u32, *mut u8) -> u32,
}

#[repr(C)]
pub struct ScsiBuffer {
    buf: *mut u8,
    size: u32,
}

#[no_mangle]
#[link_section = ".text.prologue"]
#[export_name = "_start"]
pub extern "C" fn custom_handler(resp: *mut ScsiResponse, _lun: *mut u32, pkt: *mut ScsiPacket) {
    unsafe {
        (*resp).a = 0;
        (*resp).b = 0;
        (*resp).ascq = 0;
    }

    let in_buf = unsafe { ((*(*pkt).vtbl).get_buffer)(pkt) };

    let tgt_addr = u32::from_be_bytes([
        unsafe { in_buf.add(3).read_unaligned() },
        unsafe { in_buf.add(4).read_unaligned() },
        unsafe { in_buf.add(5).read_unaligned() },
        unsafe { in_buf.add(6).read_unaligned() },
    ]);

    let mut buf = ScsiBuffer {
        buf: tgt_addr as *mut u8,
        size: 0x200,
    };
    unsafe { ((*(*pkt).vtbl).send)(pkt, 0x200, &mut buf as *mut ScsiBuffer as *mut u8) };
}
