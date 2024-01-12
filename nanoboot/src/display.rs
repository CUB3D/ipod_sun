use crate::{font, Heap, malloc};

pub struct Lcd<const WIDTH: u32, const HEIGHT: u32> {
    framebuffer: *mut u16
}

impl<const WIDTH: u32, const HEIGHT: u32> Lcd<WIDTH, HEIGHT> {
    pub fn new(/*heap: &mut Heap*/) -> Lcd<WIDTH, HEIGHT> {
       // let framebuffer = heap.alloc((WIDTH*HEIGHT*2) as usize) as *mut u16;
        let mut lcd = Lcd::<WIDTH, HEIGHT> {
            framebuffer: 0x0850_0000 as *mut u16,
        };
        lcd.init();
        lcd
    }

    pub fn set_px(&mut self, x: u32, y: u32, col: u16) {
        unsafe { self.framebuffer.add((y*WIDTH + x) as usize).write_volatile(col) };
    }

    pub fn clear(&mut self, col: u16) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.set_px(x, y, col);
            }
        }
    }

    pub fn draw_char(&mut self, chr: char, x: u32, y: u32, col: u16) {
        let char_map: [u8; 8] = font::FONT_8X8_BASIC[chr as usize];
        for cx in 0..8u32 {
            for cy in 0..8u32 {
                if (char_map[cy as usize] >> cx) & 1 != 0 {
                    self.set_px(x+cx, y+cy, col);
                }
            }
        }
    }

    pub fn draw_str(&mut self, s: &str, mut x: u32, y: u32, col: u16) {
        for c in s.chars() {
            self.draw_char(c, x, y, col);
            x += 8;
        }
    }

    pub fn init(&mut self) {
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
    }

    pub fn refresh(&mut self) {
        let r1 = (0x3800_0000 + 0x00300000) as *mut u16;

        let mut r12: u32 = WIDTH * HEIGHT;

        let mut r2 = self.framebuffer;
        while r12 > 0 {
            let r0 = unsafe { r2.read_volatile() };
            if (r2 as usize) & 0x40000000 == 0 {
                r2 = unsafe {r2.add(1)} ;
            }
            sendlcdd(r0 as usize, r1);
            r12 -= 1;
        }
    }
}


pub fn waitlcd(r1: *mut u16) {
    while unsafe {r1.add(0x1c / 2).read_volatile() & 0x10 != 0} {}
}

pub fn sendlcdd(r0: usize, r1: *mut u16) {
    unsafe {r1.add(0x40 / 2).write_volatile(r0 as u16)}
    waitlcd(r1);
}

pub fn sendlcdc(r0: usize, r1: *mut u16) {
    unsafe {r1.add(0x04 / 2).write_volatile(r0 as u16)}
    waitlcd(r1);
}