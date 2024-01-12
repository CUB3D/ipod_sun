#![no_std]
#![no_main]
#![feature(naked_functions)]
#![allow(named_asm_labels)]

use core::arch::{asm};
use crate::display::Lcd;

pub mod font;
mod display;

pub fn malloc(a: usize) -> *mut u8 {
    let some_malloc = 0x0842d444 | 1;
    let some_malloc = unsafe { core::mem::transmute::<_, extern "C" fn(usize) -> *mut u8>(some_malloc)};
    some_malloc(a)
}

pub struct Heap {
    base: *mut u8,
    head: *mut u8,
    size: usize
}
impl Heap {
    pub fn alloc(&mut self, size: usize) -> *mut u8 {
        let h = self.head;
        assert!(self.size > size);
        self.head = unsafe { h.add(size) };
        h
    }
}

#[no_mangle]
pub extern "C" fn nanoboot_main() -> ! {

    {
        // let file_struct = heap.alloc(0x54);

        // let mut file_struct = [0u8; 0x54];

        // let file_struct = malloc(0x54);
        //
        // let file_open = 0x084137a8 | 1;
        // let file_open = unsafe { core::mem::transmute::<_, extern "C" fn(*mut u8, usize, usize, usize, usize, usize, usize)>(file_open)};
        // let mut str = *b"test.txt\0";
        // file_open(file_struct, str.as_mut_ptr() as usize, 1, 0, 0x400, 1, 0);
    }

    // let base = malloc((240*432*2 * 2) as usize);
    // let mut heap = Heap {
    //     base,
    //     head: base,
    //     size: (240*432*2 * 2)
    // };

    let mut lcd = Lcd::<240, 432>::new(/*(&mut heap*/);

    loop  {
        lcd.clear(0x00E0);
        lcd.draw_str("Hello World", 10, 10, 0xFFE0);
        // lcd.draw_char('A', 10, 10, 0xFFE0);
        // lcd.draw_char('B', 10, 40, 0xFFE0);

        lcd.refresh();
    }

    loop {}
}

#[no_mangle]
#[link_section = ".text.prologue"]
#[export_name = "_start"]
#[naked]
pub extern "C" fn custom_handler() {
    unsafe {
        asm!("\
            MSR CPSR_c, #0xD3          @ Supervisor mode, no IRQs, no FIQs

            MOV R0, #0x00050000
            ORR R0, #0x00000078
            MCR p15, 0, R0, c1, c0, 0  @ Get rid of some CPU \"features\" likely to cause trouble

            b nanoboot_main
        ", options(noreturn));
    };
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

