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
    receive: extern "C" fn(*mut ScsiPacket, u32, *mut *mut u8),
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

    const SIZE: u32 = 0x200;

    match unsafe { in_buf.add(2).read_unaligned() } {
        // Write to `tgt_addr`
        1 => {
            let mut ptr = tgt_addr as *mut u8;
            unsafe { ((*(*pkt).vtbl).receive)(pkt, SIZE, &mut ptr as *mut *mut u8) };
        }
        // Read `SIZE` from `tgt_addr`
        2 => {
            let mut buf = ScsiBuffer {
                buf: tgt_addr as *mut u8,
                size: SIZE,
            };
            unsafe { ((*(*pkt).vtbl).send)(pkt, SIZE, &mut buf as *mut ScsiBuffer as *mut u8) };
        }
        // Call `tgt` as a function
        3 => {
            let f = unsafe { core::mem::transmute::<u32, extern "C" fn()>(tgt_addr)};
            f();
        }
        // Enable VROM access
        4 => {
            unsafe { (0x3C500048 as *mut u32).write_volatile(0); }
        }
        _ => unsafe {
            (*resp).a = 0x70;
            (*resp).ascq = 0x2137;
        }
    }
}
