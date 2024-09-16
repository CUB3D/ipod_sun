#![allow(internal_features, unused_variables)]
#![feature(lang_items, start, rustc_private)]
#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(abi_unadjusted)]

use core::arch::asm;

//22310b48

#[no_mangle]
#[naked]
pub unsafe extern "C" fn _start() {
asm!("
ldr r1, [sp, #0x48]
push {{r0, lr}}
push {{r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12}}
mov r0, sp
sub sp, 0x100
blx {}
add sp, 0x100
pop {{r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12}}
pop {{r0, lr}}
mov r0, 1
bx lr
",  sym  _init, options(noreturn))
}


#[no_mangle]
pub extern "C" fn _init(regs: &mut [u32; 7], ptr: u32) {
    let r0 = regs[0];
    let r1 = regs[1];
    let r2 = regs[2];
    let r3 = regs[3];
    let r4 = regs[4];
    let r5 = regs[5];
    let r6 = regs[6];


let alloc_align2 = unsafe { core::mem::transmute::<_, fn(u32) -> u32>(0x0818eb44 + 1) };
let file_stream_ctor = unsafe { core::mem::transmute::<_, fn(u32, *mut Str, u32, u32, u32, u32, u32)>(0x0819878c + 1) };
let file_stream_write = unsafe { core::mem::transmute::<_, fn(u32, u32, *const u8, *mut u32)>(0x0819888c + 1) };
let file_stream_dtor = unsafe { core::mem::transmute::<_, fn(u32)>(0x081ada78 + 1) };
let file_stream_delete = unsafe { core::mem::transmute::<_, fn(u32)>(0x08191cdc + 1) };

    {
        let mut s = Str::new_from_bytes(b"test_file.txt\0");
        let file_stream = alloc_align2(0x54);
        file_stream_ctor(file_stream, &mut s as *mut _, 0, 0, 0x400, 1, 0);
        drop(s);

        let mut out = 0u32;
        let buf = [0u8; 0x800];
        file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

        file_stream_dtor(file_stream);
        file_stream_delete(file_stream);
    }

let mut s = Str::new_from_bytes(b"test_file.txt\0");

let file_stream = alloc_align2(0x54);
file_stream_ctor(file_stream, &mut s as *mut _, 0, 0, 0x400, 1, 0);
drop(s);

    for x in regs {
        let mut out = 0u32;
        let buf = x.to_le_bytes();
        file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);
    }

    let mut out = 0u32;
    let buf = ptr.to_le_bytes();
    file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

    let count = unsafe { (ptr as *mut u32).offset(0).read_volatile() };
    let buf_ptr = unsafe { (ptr as *mut u32).offset(1).read_volatile() };

    let mut out = 0u32;
    let buf = count.to_le_bytes();
    file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

    let mut out = 0u32;
    let buf = buf_ptr.to_le_bytes();
    file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

    let mut out = 0u32;
    let buf = [b'A' as u8; 8];
    file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

    let buf_ptr = buf_ptr - 0x20;

    for x in 0..0x300 {
        let buf_ptr = unsafe { (buf_ptr as *mut u8).offset(x).read_volatile() };

        let mut out = 0u32;
        let buf = [buf_ptr];
        file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);
    }

    /*let mut out = 0u32;
    let buf = [b'A' as u8; 8];
    file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);


    for off in 0u32..100 {
        // read stack
        let x: u32;
        unsafe {
            asm!("ldr {}, [sp, {}]", out(reg) x, in(reg) off);
        }

        let mut out = 0u32;
        let buf = x.to_le_bytes();
        file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);
    }*/

    // r4 and r6 seem to have ptrs in them, good, maybe can get buffer ptr and dump what comes after

   /* let buf = [b'B' as u8; 8];
    file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);*/

    /*let r4 = r4 as *mut u32;

    for x in 0..20 {
        let buf_ptr = unsafe { r4.offset(x).read_volatile() };

        let mut out = 0u32;
        let buf = buf_ptr.to_le_bytes();
        file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);
    }

    let buf = [b'C' as u8; 8];
    file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

    let buf_ptr = unsafe { r4.offset(9).read_volatile() } as *mut u32;

    for x in 0..20 {
        let buf_ptr = unsafe { buf_ptr.offset(x).read_volatile() };

        let mut out = 0u32;
        let buf = buf_ptr.to_le_bytes();
        file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);
    }*/

    /*
    let mut our_buffer = 0 as *mut u8;

    unsafe {
        // let mut base = 0x0BB00000 as *mut u8;
        // let end = 0x0C00000;

        let mut base = 0x09900000 as *mut u8;
        let end = 0x09A00000;
        let tgt = [0x00u8, 0x00 , 0x0B , 0xDC , 0x00 , 0x00 , 0x00 , 0x00 , 0x00 , 0x01 , 0x00 , 0x00 , 0x00 , 0x00 , 0x05 , 0x70 , 0x6D , 0x64 , 0x69 , 0x61];
        'outer: loop {
            base = base.offset(1);

            if base as u32 > end {
                let mut out = 0u32;
                let buf = [b'E' as u8; 4];
                file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

                let mut out = 0u32;
                let buf = [b'F' as u8; 4];
                file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

                let mut out = 0u32;
                let buf = [b'E' as u8; 4];
                file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);
                break;
            }

            for x in 0isize..tgt.len() as isize {
                if base.offset(x).read_volatile() != tgt[x as usize] {
                    continue 'outer;
                }
            }

            our_buffer = base;

            let mut out = 0u32;
            let buf = [b'E' as u8; 4];
            file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

            let mut out = 0u32;
            let buf = (base as u32).to_le_bytes();
            file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

            let mut out = 0u32;
            let buf = [b'E' as u8; 4];
            file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

            break;
        }
    }

    if our_buffer != 0 as *mut u8 {
        let mut out = 0u32;
        let buf = [b'A' as u8; 8];
        file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

        for x in 0..0x10 {
            let mut out = 0u32;
            let buf = unsafe { (our_buffer as *mut u32).offset(x).read_volatile() }.to_le_bytes();
            file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);
        }

        let buf = [b'B' as u8; 8];
        file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);
    }*/

    /*
    let alloc_count = unsafe { r4.read_volatile() };
    let buf_ptr = unsafe { r4.offset(1).read_volatile() };

    // *(r4) = alloc count
    // *(r4 + 4) = buf ptr

    let mut out = 0u32;
    let buf = alloc_count.to_le_bytes();
    file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);

    let mut out = 0u32;
    let buf = buf_ptr.to_le_bytes();
    file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);*/

  /*  let mut out = 0u32;
    let buf = [b'D' as u8; 8];
    file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);*/

    /*
    let buf_ptr = (buf_ptr - 0x10) as *mut u8;
    for x in 0..0x20 {
        let mut out = 0u32;
        let buf = unsafe { buf_ptr.offset(x).read_volatile() }.to_le_bytes();
        file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);
    }

    let mut out = 0u32;
    let buf = [b'D' as u8; 8];
    file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);*/

    // Pad buffer
  /*  for _ in 0..20 {
        let mut out = 0u32;
        let mut buf = [b'F' as u8; 10];
        file_stream_write(file_stream, buf.len() as _, buf.as_ptr(), &mut out as *mut _);
    }*/

file_stream_dtor(file_stream);
file_stream_delete(file_stream);
}

#[repr(C)]
struct Str {
    vtbl: u32,
    buf: u32,
}

impl Str {
    pub fn new_from_bytes(tmp: &'static [u8]) -> Self {
        let mut s = Str {
            buf: 0,
            vtbl: 0,
        };
        let string_ctor = unsafe { core::mem::transmute::<_, fn(*mut Str)>(0x0827f444 + 1) };
        string_ctor(&mut s as *mut _);
        let string_from_str = unsafe { core::mem::transmute::<_, fn(*mut Str, *const u8)>(0x08196c00 + 1) };
        string_from_str(&mut s as *mut _, tmp.as_ptr());
        s
    }
}

impl Drop for Str {
    fn drop(&mut self) {
        let string_dtor = unsafe { core::mem::transmute::<_, fn(*mut Str)>(0x0827f28c + 1) };
        string_dtor(self as *mut _);
    }
}


#[lang = "eh_personality"]
fn rust_eh_personality() {}


use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! { 
    loop {}
}
