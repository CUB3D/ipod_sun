#![no_std]
#![no_main]
#![feature(naked_functions)]
#![allow(named_asm_labels)]

use core::arch::{asm, global_asm};

pub mod font;

#[no_mangle]
pub extern "C" fn waitlcd(_r0: usize, r1: *mut u16) {
    while  unsafe {r1.add(0x1c / 2).read_volatile() & 0x10 != 0} {

    }
}

#[no_mangle]
pub extern "C" fn sendlcdd(r0: usize, r1: *mut u16) {
    unsafe {r1.add(0x40 / 2).write_volatile(r0 as u16)}
    waitlcd(r0, r1);
}

#[no_mangle]
pub extern "C" fn sendlcdc(r0: usize, r1: *mut u16) {
    unsafe {r1.add(0x04 / 2).write_volatile(r0 as u16)}
    waitlcd(r0, r1);
}

#[no_mangle]
pub extern "C" fn displaylcd2() {
    const WIDTH: usize = 240;
    const HEIGHT: usize = 432;

    {
        let mut r2 = 0x08500000 as *mut u16;

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                unsafe { r2.add(y*WIDTH + x).write_volatile(0x00E0) };
            }
        }
    }

    {
        let chr = 'A';

        let mut x = 10;
        let y = 10;

        let mut r2 = 0x08500000 as *mut u16;

            let char_map: [u8; 8] = font::FONT_8X8_BASIC[chr as usize];
            for cx in 0..8 {
               for cy in 0..8 {
                    //    unsafe { r2.add((y + cy)*WIDTH + x + cx).write_volatile(0xFFE0) };
                    if (char_map[cy] >> cx) & 1 != 0 {
                        unsafe { r2.add((y + cy)*WIDTH + x + cx).write_volatile(0xFFE0) };
                    } else {
                        unsafe { r2.add((y + cy)*WIDTH + x + cx).write_volatile(0x00E0) };
                    }
               } 
            }

            x += 8;

            let chr = 'B';

            let char_map: [u8; 8] = font::FONT_8X8_BASIC[chr as usize];
            for cx in 0..8 {
               for cy in 0..8 {
                    //    unsafe { r2.add((y + cy)*WIDTH + x + cx).write_volatile(0xFFE0) };
                    if (char_map[cy] >> cx) & 1 != 0 {
                        unsafe { r2.add((y + cy)*WIDTH + x + cx).write_volatile(0xFFE0) };
                    } else {
                        unsafe { r2.add((y + cy)*WIDTH + x + cx).write_volatile(0x00E0) };
                    }
               } 
            }

            x += 8;
    }



    let r1 = (0x3800_0000 + 0x00300000) as *mut u16;

    sendlcdc(0x2a, r1);

    sendlcdd(0x00EF0000, r1);

    sendlcdd(0x00EF0000 >> 16, r1);

    sendlcdd(0x2b, r1);

    //WTF... But it's neccessary.
    let r9: usize = 0x01000000 + 0x003F0000;
    let mut r0 = r9;
    if ((r0 & 0x100) != 0) {
        r0 = r0 ^ 0x300;
    }
    sendlcdd(r0, r1);

    //WTF... But it's neccessary.
    let mut r0: usize = r9 >> 16;
    if ((r0 & 0x100) != 0) {
        r0 = r0 ^ 0x300;
    }
    sendlcdd(r0, r1);

    sendlcdd(0x2c, r1);

    let mut r12: usize = WIDTH * HEIGHT;

   let mut r2 = 0x08500000 as *mut u16;

  while r12 > 0 {
      let r0 = unsafe { r2.read_volatile() };
      if (r2 as usize) & 0x40000000 == 0 {
          r2 = unsafe {r2.add(1)} ;
      }
      sendlcdd(r0 as usize, r1);
      r12 -= 1;
  }
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

@MOV R0, #0x08500000
@LDR R1, val_color
@MVN R2, #1
@ADR R3, strings2
@BL rendertext
@
@MOV R0, #0x08500000
@ADD R0, R0, #0x1E00
@ADR R3, strings2
@MVN R2, #1
@BL rendertext

@MOV R2, #0x08500000
@MOV R5, #0x00EF0000
@MOV R9, #0x01000000
@ADD R9, R9, #0x003F0000
BL displaylcd2

hang:
b hang


@      rendertext:
@        ldrb r12, [r3], #1
@        cmp r12, #0
@        moveq pc, lr
@        cmn r2, #1
@        beq rendernobg
@          mov r6, r0
@          mov r4, #8
@          renderbgrow:
@            mov r5, #6
@            renderbgcol:
@              cmn r2, #2
@              ldrheq r7, [r6]
@             moveq r7, r7,lsr#1
@              biceq r7, #0x410
@              strheq r7, [r6], #2
@              strhne r2, [r6], #2
@              subs r5, r5, #1
@            bne renderbgcol
@           add r6, r6, #468
@           subs r4, r4, #1
@          bne renderbgrow
@        rendernobg:
@        adr r5, font
@        sub r12, r12, #0x20
@        cmp r12, #0x5f
@        addcc r5, r12,lsl#2
@        addcc r5, r12
@        mov r12, #5
@        rendercol:
@            mov r6, r0
@            ldrb r4, [r5], #1
@          renderrow:
@           tst r4, #1
@           strhne r1, [r6]
@           add r6, r6, #480
@           movs r4, r4,lsr#1
@          bne renderrow
@          add r0, r0, #2
@          subs r12, r12, #1
@        bne rendercol
@        add r0, r0, #2
@      b rendertext
@
@
@      val_color:
@      .word 0xFFE0
@
@      strings2:
@      .ascii \"Test\\0\"
@
@      .align 2
@      font:
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 95
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 7
@      .byte 0
@      .byte 7
@      .byte 0
@      .byte 20
@      .byte 127
@      .byte 20
@      .byte 127
@      .byte 20
@      .byte 36
@      .byte 42
@      .byte 127
@      .byte 42
@      .byte 18
@      .byte 35
@      .byte 19
@      .byte 8
@      .byte 100
@      .byte 98
@      .byte 54
@      .byte 73
@      .byte 85
@      .byte 34
@      .byte 80
@      .byte 5
@      .byte 3
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 28
@      .byte 34
@      .byte 65
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 65
@      .byte 34
@      .byte 28
@      .byte 20
@      .byte 8
@      .byte 62
@      .byte 8
@      .byte 20
@      .byte 8
@      .byte 8
@      .byte 62
@      .byte 8
@      .byte 8
@      .byte 0
@      .byte -96
@      .byte 96
@      .byte 0
@      .byte 0
@      .byte 8
@      .byte 8
@      .byte 8
@      .byte 8
@      .byte 8
@      .byte 0
@      .byte 96
@      .byte 96
@      .byte 0
@      .byte 0
@      .byte 32
@      .byte 16
@      .byte 8
@      .byte 4
@      .byte 2
@      .byte 62
@      .byte 81
@      .byte 73
@      .byte 69
@      .byte 62
@      .byte 0
@      .byte 66
@      .byte 127
@      .byte 64
@      .byte 0
@      .byte 66
@      .byte 97
@      .byte 81
@      .byte 73
@      .byte 70
@      .byte 33
@      .byte 65
@      .byte 69
@      .byte 75
@      .byte 49
@      .byte 24
@      .byte 20
@      .byte 18
@      .byte 127
@      .byte 16
@      .byte 39
@      .byte 69
@      .byte 69
@      .byte 69
@      .byte 57
@      .byte 60
@      .byte 74
@      .byte 73
@      .byte 73
@      .byte 48
@      .byte 1
@      .byte 113
@      .byte 9
@      .byte 5
@      .byte 3
@      .byte 54
@      .byte 73
@      .byte 73
@      .byte 73
@      .byte 54
@      .byte 6
@      .byte 73
@      .byte 73
@      .byte 41
@      .byte 30
@      .byte 0
@      .byte 54
@      .byte 54
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 86
@      .byte 54
@      .byte 0
@      .byte 0
@      .byte 8
@      .byte 20
@      .byte 34
@      .byte 65
@      .byte 0
@      .byte 20
@      .byte 20
@      .byte 20
@      .byte 20
@      .byte 20
@      .byte 0
@      .byte 65
@      .byte 34
@      .byte 20
@      .byte 8
@      .byte 2
@      .byte 1
@      .byte 81
@      .byte 9
@      .byte 6
@      .byte 50
@      .byte 73
@      .byte 121
@      .byte 65
@      .byte 62
@      .byte 124
@      .byte 18
@      .byte 17
@      .byte 18
@      .byte 124
@      .byte 127
@      .byte 73
@      .byte 73
@      .byte 73
@      .byte 62
@      .byte 62
@      .byte 65
@      .byte 65
@      .byte 65
@      .byte 34
@      .byte 127
@      .byte 65
@      .byte 65
@      .byte 34
@      .byte 28
@      .byte 127
@      .byte 73
@      .byte 73
@      .byte 73
@      .byte 65
@      .byte 127
@      .byte 9
@      .byte 9
@      .byte 9
@      .byte 1
@      .byte 62
@      .byte 65
@      .byte 73
@      .byte 73
@      .byte 58
@      .byte 127
@      .byte 8
@      .byte 8
@      .byte 8
@      .byte 127
@      .byte 0
@      .byte 65
@      .byte 127
@      .byte 65
@      .byte 0
@      .byte 32
@      .byte 64
@      .byte 65
@      .byte 63
@      .byte 1
@      .byte 127
@      .byte 8
@      .byte 20
@      .byte 34
@      .byte 65
@      .byte 127
@      .byte 64
@      .byte 64
@      .byte 64
@      .byte 64
@      .byte 127
@      .byte 2
@      .byte 12
@      .byte 2
@      .byte 127
@      .byte 127
@      .byte 4
@      .byte 8
@      .byte 16
@      .byte 127
@      .byte 62
@      .byte 65
@      .byte 65
@      .byte 65
@      .byte 62
@      .byte 127
@      .byte 9
@      .byte 9
@      .byte 9
@      .byte 6
@      .byte 62
@      .byte 65
@      .byte 81
@      .byte 33
@      .byte 94
@      .byte 127
@      .byte 9
@      .byte 25
@      .byte 41
@      .byte 70
@      .byte 38
@      .byte 73
@      .byte 73
@      .byte 73
@      .byte 50
@      .byte 1
@      .byte 1
@      .byte 127
@      .byte 1
@      .byte 1
@      .byte 63
@      .byte 64
@      .byte 64
@      .byte 64
@      .byte 63
@      .byte 31
@      .byte 32
@      .byte 64
@      .byte 32
@      .byte 31
@      .byte 127
@      .byte 32
@      .byte 24
@      .byte 32
@      .byte 127
@      .byte 99
@      .byte 20
@      .byte 8
@      .byte 20
@      .byte 99
@      .byte 3
@      .byte 4
@      .byte 120
@      .byte 4
@      .byte 3
@      .byte 97
@      .byte 81
@      .byte 73
@      .byte 69
@      .byte 67
@      .byte 0
@      .byte 127
@      .byte 65
@      .byte 65
@      .byte 0
@      .byte 2
@      .byte 4
@      .byte 8
@      .byte 16
@      .byte 32
@      .byte 0
@      .byte 65
@      .byte 65
@      .byte 127
@      .byte 0
@      .byte 4
@      .byte 2
@      .byte 1
@      .byte 2
@      .byte 4
@      .byte 64
@      .byte 64
@      .byte 64
@      .byte 64
@      .byte 64
@      .byte 1
@      .byte 2
@      .byte 4
@      .byte 0
@      .byte 0
@      .byte 32
@      .byte 84
@      .byte 84
@      .byte 84
@      .byte 120
@      .byte 127
@      .byte 68
@      .byte 68
@      .byte 68
@      .byte 56
@      .byte 56
@      .byte 68
@      .byte 68
@      .byte 68
@      .byte 40
@      .byte 56
@      .byte 68
@      .byte 68
@      .byte 68
@      .byte 127
@      .byte 56
@      .byte 84
@      .byte 84
@      .byte 84
@      .byte 24
@      .byte 8
@      .byte 126
@      .byte 9
@      .byte 1
@      .byte 2
@      .byte 8
@      .byte 84
@      .byte 84
@      .byte 84
@      .byte 60
@      .byte 127
@      .byte 4
@      .byte 4
@      .byte 4
@      .byte 120
@      .byte 0
@      .byte 68
@      .byte 125
@      .byte 64
@      .byte 0
@      .byte 32
@      .byte 64
@      .byte 64
@      .byte 61
@      .byte 0
@      .byte 127
@      .byte 16
@      .byte 40
@      .byte 68
@      .byte 0
@      .byte 0
@      .byte 65
@      .byte 127
@      .byte 64
@      .byte 0
@      .byte 124
@      .byte 4
@      .byte 24
@      .byte 4
@      .byte 120
@      .byte 124
@      .byte 8
@      .byte 4
@      .byte 4
@      .byte 120
@      .byte 56
@      .byte 68
@      .byte 68
@      .byte 68
@      .byte 56
@      .byte 124
@      .byte 20
@      .byte 20
@      .byte 20
@      .byte 24
@      .byte 8
@      .byte 20
@      .byte 20
@      .byte 20
@      .byte 124
@      .byte 124
@      .byte 8
@      .byte 4
@      .byte 4
@      .byte 8
@      .byte 72
@      .byte 84
@      .byte 84
@      .byte 84
@      .byte 32
@      .byte 4
@      .byte 63
@      .byte 68
@      .byte 64
@      .byte 32
@      .byte 60
@      .byte 64
@      .byte 64
@      .byte 32
@      .byte 124
@      .byte 28
@      .byte 32
@      .byte 64
@      .byte 32
@      .byte 28
@      .byte 60
@      .byte 64
@      .byte 56
@      .byte 64
@      .byte 60
@      .byte 68
@      .byte 40
@      .byte 16
@      .byte 40
@      .byte 68
@      .byte 12
@      .byte 80
@      .byte 80
@      .byte 80
@      .byte 60
@      .byte 68
@      .byte 100
@      .byte 84
@      .byte 76
@      .byte 68
@      .byte 0
@      .byte 8
@      .byte 54
@      .byte 65
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 119
@      .byte 0
@      .byte 0
@      .byte 0
@      .byte 65
@      .byte 54
@      .byte 8
@      .byte 0
@      .byte 2
@      .byte 1
@      .byte 2
@      .byte 4
@      .byte 2
    ", options(noreturn));
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

