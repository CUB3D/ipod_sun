#![feature(lazy_cell)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use unicorn_engine::unicorn_const::{Arch, HookType, Mode, Permission, SECOND_SCALE};
use unicorn_engine::{RegisterARM, Unicorn};


static MALLOC_ADDRS: RwLock<SlabAlloc> = RwLock::new(SlabAlloc {
    top_addr: 0x4000_0000,
    addrs: Vec::new(),
});

struct SlabAlloc {
    top_addr: u64,
    addrs: Vec<u64>
}

const MAX_ALLOC_SIZE: u64 = 0x8000;

impl SlabAlloc {
    pub fn alloc(&mut self, size: u64, f: &mut Unicorn<()>) -> u64 {
        if size > MAX_ALLOC_SIZE {
            panic!("ALloc {size:x} > MAX");
            return 0;
        }

        let x = self.top_addr;
        self.top_addr += MAX_ALLOC_SIZE + 0x1000;

        self.addrs.push(x);

        f.mem_map(x, MAX_ALLOC_SIZE as _, Permission::READ | Permission::WRITE).unwrap();
        x
    }

    pub fn free(&mut self, p: u64, f: &mut Unicorn<()>) {
        assert!(self.addrs.contains(&p));

        f.mem_unmap(p, MAX_ALLOC_SIZE as _).unwrap();

        let idx = self.addrs.iter().position(|p1| *p1 == p).unwrap();
        self.addrs.remove(idx);
    }

    pub fn reset(&mut self, f: &mut Unicorn<()>) {
        self.top_addr = 0x4000_0000;

        if !self.addrs.is_empty() {
            println!("mem leak");
        }

        for x in &self.addrs {
            f.mem_write(*x, &[0u8; MAX_ALLOC_SIZE as _]).unwrap();
            f.mem_unmap(*x, MAX_ALLOC_SIZE as _).unwrap();
        }

        self.addrs.clear();
    }
}

static LOG_MEM: AtomicBool = AtomicBool::new(false);
static LOG_EXEC: AtomicBool = AtomicBool::new(false);

fn main() {
    // List of caveats / differences about this emu vs on-device:
    // - we don't use the correct libc functions
    // - - we are not 100% sure we have the *correct* libc functions for each (in 22xxxxx addr space, we just used the FT code rather than reversing)
    // - our heap is *stricter*, this is a good thing
    // - We map low ram

    // What this means:
    // if it doesn't work on n5g it could be:
    // - Wrong libc functions causing different behaviour
    // - If low ram is protected / used somewhere else we might be causing corruption
    // - Stack differences due to above


    let ram_data = std::fs::read("./ram.bin").unwrap();


    let mut unicorn = unicorn_engine::Unicorn::new(Arch::ARM, Mode::THUMB | Mode::LITTLE_ENDIAN)
        .expect("failed to initialize Unicorn instance");

    let emu = &mut unicorn;

    // Dodgy page, after corruption code might try to read/write from random ptrs, hopefully that is fine on device...
    emu.mem_map(0, 0x1000, Permission::ALL).unwrap();

    emu.add_mem_hook(HookType::MEM_ALL, 0, u64::MAX, |f, a, b, c, d| {
        let mut dat = vec![0u8; c];
        let _ = f.mem_read(b, &mut dat);

        let s = format!("   {a:?} @ {b:x?}, size = {c:x}, dat = {dat:x?}");
        if LOG_MEM.load(Ordering::SeqCst) {
            println!("{s}");
        }
        true
    })
        .unwrap();


    emu.add_code_hook(0, u64::MAX, |f, a, _b| {
        let pc = f.pc_read().unwrap();
        let mut tmp = [0u8; 4];
        let _ = f.mem_read(pc, &mut tmp);

        let s = format!("EXEC {:x} [{:08x}, {:08x}, {:08x}, {:08x}, {:08x}, {:08x}, {:08x}, {:08x}], {:02x?}", a ,
                        f.reg_read(RegisterARM::R0).unwrap(),
                        f.reg_read(RegisterARM::R1).unwrap(),
                        f.reg_read(RegisterARM::R2).unwrap(),
                        f.reg_read(RegisterARM::R3).unwrap(),
                        f.reg_read(RegisterARM::R4).unwrap(),
                        f.reg_read(RegisterARM::R5).unwrap(),
                        f.reg_read(RegisterARM::R6).unwrap(),
                        f.reg_read(RegisterARM::R7).unwrap(),
                        tmp,
        );
        if LOG_EXEC.load(Ordering::SeqCst) {
            println!("{s}");
        }
    })
        .unwrap();



    emu.mem_map(0x0800_0000, 0x00f0_0000, Permission::ALL).expect("failed to map code page");
    emu.mem_write(0x0800_0000, &ram_data).unwrap();
    //                       8e5_0228

    emu.mem_map(0x2200_0000, 0x070_0000, Permission::ALL).expect("failed to map code page");

    // realloc
    emu.add_code_hook(
        0x082e6180 + 1,
        0x082e6180 as u64 + 8,
        |f, _a, _b| {
            let count = f.reg_read(RegisterARM::R0).unwrap();

            let addr = MALLOC_ADDRS.write().unwrap().alloc(count, f);
            f.reg_write(RegisterARM::R0, addr).unwrap();

           //println!("malloc({count:x}) = {addr:x}");


            let lr = f.reg_read(RegisterARM::LR).unwrap();
            f.set_pc(lr).unwrap();
        },
    ).unwrap();

    emu.add_code_hook(
        0x082e6194 + 1,
        0x082e6194 as u64 + 8,
        |f, _a, _b| {
            // Apple, why, are, the, args, the, wrong, way, around
            let ptr = f.reg_read(RegisterARM::R1).unwrap();
            let count = f.reg_read(RegisterARM::R0).unwrap();

            if ptr == 0 {
                panic!("Null ptr");
            }

            if count > MAX_ALLOC_SIZE {
                panic!("Realloc fail, req {count:x} bigger than max");
            }

            //println!("realloc({ptr:x}, {count:x})");

            f.reg_write(RegisterARM::R0, ptr).unwrap();



            let lr = f.reg_read(RegisterARM::LR).unwrap();
            f.set_pc(lr).unwrap();
        },
    ).unwrap();

    // Memset
    emu.add_code_hook(
        0x22000074,
        0x22000070 as u64 + 8,
        |f, _a, _b| {
            let ptr = f.reg_read(RegisterARM::R0).unwrap();
            // let val = f.reg_read(RegisterARM::R1).unwrap();
            let count = f.reg_read(RegisterARM::R1).unwrap();

         //   println!("bzero({ptr:x}, {count:x})");

            for i in 0..count {
                f.mem_write(ptr + i, &[0u8]).unwrap()
            }

            let lr = f.reg_read(RegisterARM::LR).unwrap();
            f.set_pc(lr).unwrap();
        },
    ).unwrap();

    // Bzero
    emu.add_code_hook(
        0x082002cc,
        0x082002cc as u64 + 8,
        |f, _a, _b| {
            let ptr = f.reg_read(RegisterARM::R0).unwrap();
            let count = f.reg_read(RegisterARM::R1).unwrap();

          //  eprintln!("bzero2({ptr:x}, {count:x})");

            for i in 0..count {
                f.mem_write(ptr + i, &[0u8]).unwrap()
            }

            let lr = f.reg_read(RegisterARM::LR).unwrap();
            f.set_pc(lr).unwrap();
        },
    ).unwrap();

    // free
    emu.add_code_hook(
        0x082e618a,
        0x082e618a as u64 + 8,
        |f, _a, _b| {
            let ptr = f.reg_read(RegisterARM::R0).unwrap();
            let lr = f.reg_read(RegisterARM::LR).unwrap();

            eprintln!("free({ptr:x}), called from {lr:x}");

            f.set_pc(lr).unwrap();
        },
    ).unwrap();

    // memcpy
    emu.add_code_hook(
        0x220000b4,
        0x220000b4 as u64 + 8,
        |f, _a, _b| {
            let dst = f.reg_read(RegisterARM::R0).unwrap();
            let src = f.reg_read(RegisterARM::R1).unwrap();
            let len = f.reg_read(RegisterARM::R2).unwrap();

            for i in 0..len {
                let mut tmp = [0u8];
                f.mem_read(src + i, &mut tmp).unwrap();
                f.mem_write(dst + i, &tmp).unwrap();
            }

       //     eprintln!("memcpy({dst:x}, {src:x}, {len:x})");

            let lr = f.reg_read(RegisterARM::LR).unwrap();
            f.set_pc(lr).unwrap();
        },
    ).unwrap();

    // memcpy
    emu.add_code_hook(
        0x220001ec,
        0x220001ec as u64 + 8,
        |f, _a, _b| {
            let dst = f.reg_read(RegisterARM::R0).unwrap();
            let src = f.reg_read(RegisterARM::R1).unwrap();
            let len = f.reg_read(RegisterARM::R2).unwrap();

            for i in 0..len {
                let mut tmp = [0u8];
                f.mem_read(src + i, &mut tmp).unwrap();
                f.mem_write(dst + i, &tmp).unwrap();
            }

         //   eprintln!("memcpy2({dst:x}, {src:x}, {len:x})");

            let lr = f.reg_read(RegisterARM::LR).unwrap();
            f.set_pc(lr).unwrap();
        },
    ).unwrap();

    emu.add_code_hook(
        0x08272474,
        0x08272474 as u64 + 8,
        |f, _a, _b| {
            eprintln!("If you are here, something has gone very wrong");
            panic!();
        },
    ).unwrap();

    let tmp = [
        (0x082e88f4, "cff_face_init"),
        (0x082e9e44, "cff_driver_init"),
        (0x082efe14, "ft_new_glyph_slot"),
        (0x0821c228, "ft_list_add"),
        (0x083173c0, "find_unicode_charmap"),
        // (0x08244070, "ft_mem_alloc"),
        (0x08306c90, "destroy_charmap"),
        // (0x081f5f98, "ft_get_module_interface"),
        (0x082e7e7c, "tt_face_init"),
        (0x082e7edc, "tt_face_init[ret]"),

        (0x082f1fc0, "sfnt_init_face"),
        // (0x08235b90, "ft_stream_seek"),
        (0x082ef360, "ps_hinter_init"),
        // (0x08232928, "ft_module_get_service"),
        (0x08231334, "FT_ASSERT"),
        (0x082ef28c, "cff_get_interface"),
        // (0x082312fc, "ft_get_module"),
        // (0x082f8d60, "sfnt_get_interface"),
        (0x082f5202, "open_face"),
        // (0x08205a40, "ft_service_list_lookup"),
        // (0x0818edaa, "ft_strcmp"),
        (0x082f5298, "open_face[ret]"),
        (0x08237ea6, "ft_stream_new"),
        (0x08228290, "ft_stream_open_memory"),
        (0x08324bb2, "ft_stream_open"),
        (0x08237f1a, "ft_stream_new[free]"),
        (0x08237ed8, "ft_stream_new[flag chk]"),
        (0x082e7afc, "t1_face_init"),


        (0x08231732, "print_check"),

        (0x082e7624, "cff_load_glyph"),
        (0x082f1764, "cff_slot_load"),
        (0x0820e400, "cff_decoder_parse_charstrings"),
        (0x0820f13e, "case[endchar]"),
        // (0x0820f406, "case[put]"),
        // (0x0820f41e, "put[idx] =r0"),
        // (0x0820f42e, "put[len_buildchar] =r1"),
    ];

    for (addr, name) in tmp.into_iter() {
        emu.add_code_hook(
            addr,
            addr as u64 + 0,
            |f, _a, _b| {
                let name = name.to_string();
                println!("{}, [{:x}, {:x}, {:x}]", name,
                         f.reg_read(RegisterARM::R0).unwrap(),
                         f.reg_read(RegisterARM::R1).unwrap(),
                         f.reg_read(RegisterARM::R2).unwrap(),
                );
            },
        ).unwrap();
    }


    emu.add_code_hook(
        0x100ffc80,
        0x100ffc80 as u64 + 0,
        |f, _a, _b| {
            let pc = f.pc_read().unwrap();

            let mut code_bytes = [0u8; 0x10];
            f.mem_read(pc, &mut code_bytes).unwrap();

            println!("pc = {:x}, Code at overwritten pc = {:x?}", pc, code_bytes);


            LOG_EXEC.store(true, Ordering::SeqCst);
            LOG_MEM.store(true, Ordering::SeqCst);
        },
    ).unwrap();

    // put
    emu.add_code_hook(
        0x0820f406,
        0x0820f406 as u64 + 0,
        |f, _a, _b| {
            let sp = f.reg_read(RegisterARM::SP).unwrap();
            let mut tmp = [0u8; 4];
            f.mem_read(sp + 0x48, &mut tmp).unwrap();
            let decoder = u32::from_le_bytes(tmp);
            println!("decoder @ {:x}", decoder);

            let mut tmp = [0u8; 4];
            f.mem_read(decoder as u64 + 0x30c, &mut tmp).unwrap();
            let len_buildhcar = u32::from_le_bytes(tmp);
            println!("len_buildchar @ {:x}", len_buildhcar);

            let mut tmp = [0u8; 4];
            f.mem_read(decoder as u64 + 0x308, &mut tmp).unwrap();
            let len_buildhcar = u32::from_le_bytes(tmp);
            println!("buildchar @ {:x}", len_buildhcar);

        },
    ).unwrap();

    // This turn on verbose logging, it is off on device
    // But useful for testing, however it requires patching around some floating point stuff
    //  because we don't turn it on
    const FORCE_FT_LOGGING: bool = true;

    if FORCE_FT_LOGGING {
        // apple left verbose logging on, it will try to print some values as floats, we haven't set the cpu flags to turn on floats
        emu.add_code_hook(
            0x0827f410,
            0x0827f410 as u64 + 0,
            |f, _a, _b| {
                let lr = f.reg_read(RegisterARM::LR).unwrap();
                f.set_pc(lr).unwrap();
            },
        ).unwrap();


        // Force trace level to print more
        emu.mem_write(0x08e501c0, 0x10_u32.to_le_bytes().as_slice()).unwrap();

        // There are 2 flags for this apprently, this one is for parse_charstrings
        emu.mem_write(0x8e50244, 0x10_u32.to_le_bytes().as_slice()).unwrap();

        // TRACE
        emu.add_code_hook(
            0x0824679c,
            0x0824679c as u64 + 0,
            |f, _a, _b| {
                let mut chars = Vec::new();

                let mut str_addr = f.reg_read(RegisterARM::R0).unwrap();

                loop {
                    let mut tmp = [0u8];
                    f.mem_read(str_addr, &mut tmp).unwrap();
                    if tmp[0] == 0 {
                        break;
                    }
                    chars.push(tmp[0]);
                    str_addr += 1;
                }

                println!("TRACE: {:?}, [0x{:x}, 0x{:x}]", String::from_utf8(chars),
                         f.reg_read(RegisterARM::R1).unwrap(),
                         f.reg_read(RegisterARM::R2).unwrap(),
                );
            },
        ).unwrap();

        // FT_TRace2 (print requires setup)
        emu.add_code_hook(
            0x0824679c,
            0x0824679c as u64 + 8,
            |f, _a, _b| {
                let lr = f.reg_read(RegisterARM::LR).unwrap();
                f.set_pc(lr).unwrap();
            },
        ).unwrap();
    }


    let sp_base: u64 = 0x1000_0000;
    let sp_size: u64 = 0x0080_0000;
    emu.mem_map(sp_base, sp_size as _, Permission::ALL)
        .expect("failed to map code page");

    emu.reg_write(RegisterARM::SP as i32, sp_base + sp_size - 0x100)
        .expect("failed write R5");

    const LIB: u32 = 0x1100_0000;
    const LIB_PTR: u32 = 0x1150_0000;
    const LIB_PTR2: u32 = 0x1160_0000;
    const ARGS: u32 = 0x1200_0000;
    const FACE: u32 = 0x1300_0000;

    emu.mem_map(LIB as _, 0x0010_0000, Permission::ALL).unwrap();
    emu.mem_map(LIB_PTR as _, 0x0010_0000, Permission::ALL).unwrap();
    emu.mem_map(LIB_PTR2 as _, 0x0010_0000, Permission::ALL).unwrap();
    emu.mem_map(ARGS as _, 0x0010_0000, Permission::ALL).unwrap();
    emu.mem_map(FACE as _, 0x0010_0000, Permission::ALL).unwrap();

    emu.mem_write(LIB_PTR as _, LIB.to_le_bytes().as_slice()).unwrap();
    emu.mem_write(LIB_PTR2 as _, LIB_PTR.to_le_bytes().as_slice()).unwrap();

    //FT_Init_FreeType
    {
        emu.reg_write(RegisterARM::LR as i32, 0xDEADBEE0)
            .expect("failed write R5");

        emu.reg_write(RegisterARM::R0 as i32, LIB_PTR as _)
            .expect("failed write R0");

        let r = emu.emu_start(0x082c088c + 1, 0, 10 * SECOND_SCALE, 100000);
        if let Err(_x) = r {
            let pc = emu.pc_read().unwrap();
            // Magic for returned with error
            if pc != 0xDEADBEE0 {
                panic!("pc != {:x?}", pc);
            }
        }
        let r0 = emu.reg_read(RegisterARM::R0).unwrap() as i32;
        println!("ret = {:x?}", r0);
        assert_eq!(r0, 0);
    }

    println!("--------- INIT FT OK ---------");

    let mut tmp = [0u8; 4];

    emu.mem_read(LIB_PTR as _, &mut tmp).unwrap();
    let ptr = u32::from_le_bytes(tmp);
    println!("TMP = {ptr:x}");

    //FT_Open_Face
    {
        emu.reg_write(RegisterARM::LR as i32, 0xDEADBEE0)
            .expect("failed write R5");

        emu.reg_write(RegisterARM::R0 as i32, ptr as u64).expect("failed write R0");
        emu.reg_write(RegisterARM::R1 as i32, ARGS as _).unwrap();
        emu.reg_write(RegisterARM::R2 as i32, 0).unwrap();
        emu.reg_write(RegisterARM::R3 as i32, FACE as _).unwrap();

        let mem = std::fs::read("/home/cub3d/projects/ipod/freetype_codexec/cff_generate/hmm.cff").unwrap();
        // let mem = std::fs::read("/home/cub3d/projects/ipod/freetype_codexec/5g_rsrc_patching/ipodhax/comic-otf-exploit.otf").unwrap();

        let mem_base = 0x3000_0000_u32;
        let mem_size: u32 = mem.len().try_into().unwrap();//0x010_u32;
        let alloc_size = 0x80000_u32;

        assert!(mem_size < alloc_size, "{} < {}", mem_size, alloc_size);

        emu.mem_map(mem_base as _, alloc_size as _, Permission::ALL).unwrap();

        emu.mem_write(mem_base as _, &mem).unwrap();


        // Setup ARGS
        emu.mem_write(ARGS as u64 + 0, 1u32.to_le_bytes().as_slice()).unwrap(); // flags = mem stream
        emu.mem_write(ARGS as u64 + 4, mem_base.to_le_bytes().as_slice()).unwrap();
        emu.mem_write(ARGS as u64 + 8, mem_size.to_le_bytes().as_slice()).unwrap();

        let r = emu.emu_start(0x082315e0 + 1, 0, 30 * SECOND_SCALE, 10_000_000);
        let pc = emu.pc_read().unwrap();
        println!("Exit setup pc = {pc}");
        if let Err(_x) = r {
            // Magic for returned with error
            if pc != 0xDEADBEE0 {
                println!("pc = {:x?}", pc);
                return;
            }
        } else {
            // panic!("Failed");
        }
        let r0 = emu.reg_read(RegisterARM::R0).unwrap() as i32;
        println!("ret = {:x?}", r0);
    }

    let mut tmp = [0u8; 4];

    emu.mem_read(FACE as _, &mut tmp).unwrap();
    let ptr = u32::from_le_bytes(tmp);
    println!("face = {ptr:x}");

    // FT_Load_Glyph
    {
        println!("---------------");
        println!("---------------");
        println!("---------------");
        println!("---------------");

        emu.reg_write(RegisterARM::LR as i32, 0xDEADBEE0)
            .expect("failed write R5");

        emu.reg_write(RegisterARM::R0 as i32, ptr as u64).expect("failed write R0");
        emu.reg_write(RegisterARM::R1 as i32, 1).unwrap(); // glyph idx
        emu.reg_write(RegisterARM::R2 as i32, 0).unwrap();

        let r = emu.emu_start(0x08232374 + 1, 0, 20 * SECOND_SCALE, 1000_000);
        let pc = emu.pc_read().unwrap();
        println!("Exit pc = {pc:x}");
        if let Err(_x) = r {
            // Magic for returned with error
            if pc != 0xDEADBEE0 {
                println!("pc = {:x?}, err = {:?}", pc, _x);
            }
            println!("Returned no error");
        } else {
             panic!("Too many instructions");
        }
        let r0 = emu.reg_read(RegisterARM::R0).unwrap() as i32;
        println!("ret = {:x?}", r0);
    }

    LOG_EXEC.store(true, Ordering::SeqCst);

    // emu.mem_write(0x080016a4, &[0u8]).unwrap();

    for _ in 0..10 {
        emu.reg_write(RegisterARM::LR as i32, 0xDEADBEE0)
            .expect("failed write R5");
        let r = emu.emu_start(0x0800_00c5, 0, 20 * SECOND_SCALE, 0x3500);
        let pc = emu.pc_read().unwrap();
        println!("Exit pc = {pc:x}, r = {r:?}");
    }
}