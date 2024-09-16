#![feature(lazy_cell)]
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, RwLock};
use basic_mutator::InputDatabase;
use unicorn_engine::unicorn_const::{Arch, HookType, Mode, Permission, SECOND_SCALE};
use unicorn_engine::{RegisterARM, Unicorn};

const EXAMPLE_CERT: [u8; 3043] = *include_bytes!("../cert_bundle.bin");

static COV_PC: LazyLock<RwLock<HashMap<u64, ()>>> = LazyLock::new(|| RwLock::new(HashMap::new()));
static COV_MEM: LazyLock<RwLock<HashMap<u64, ()>>> = LazyLock::new(|| RwLock::new(HashMap::new()));
static GBL_COV: AtomicBool = AtomicBool::new(false);

static GBL_LOG: RwLock<String> = RwLock::new(String::new());

static MALLOC_ADDRS: RwLock<SlabAlloc> = RwLock::new(SlabAlloc {
    top_addr: 0x4000_0000,
    addrs: Vec::new(),
});

struct SlabAlloc {
    top_addr: u64,
    addrs: Vec<u64>
}

const MAX_ALLOC_SIZE: u64 = 0x1000;

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

pub fn take(i: &[u8], count: usize) -> (&[u8], &[u8]) {
    (&i[count..], &i[..count])
}

pub fn le_u32(i: &[u8]) -> (&[u8], u32) {
    (&i[4..], u32::from_le_bytes((&i[..4]).try_into().unwrap()))
}

pub fn le_u16(i: &[u8]) -> (&[u8], u16) {
    (&i[2..], u16::from_le_bytes((&i[..2]).try_into().unwrap()))
}

fn main() {
    let pe_byte = std::fs::read("./section1.pe").unwrap();
    let pe = goblin::pe::PE::parse(&pe_byte).unwrap();

    let reloc = pe.sections.iter().find(|s| s.name().unwrap().contains(".reloc")).unwrap();
    let reloc_sec = &pe_byte[reloc.virtual_address as usize..][..reloc.size_of_raw_data as usize];
    println!("Prefered load addr = {:x}", pe.image_base);
    assert_eq!(pe.image_base, 0);

    let pe_base: u64 = 0x1000;

    // let realloc_delta = pe.image_base as u64 + pe_base;

    let mut unicorn = unicorn_engine::Unicorn::new(Arch::ARM, Mode::THUMB)
        .expect("failed to initialize Unicorn instance");
    let emu = &mut unicorn;

    emu.add_mem_hook(HookType::MEM_ALL, 0, u64::MAX, |f, a, b, c, d| {
        let mut dat = vec![0u8; c];
        let _ = f.mem_read(b, &mut dat);

        let s = format!("   {a:?} @ {b:x?}, size = {c:x}, dat = {dat:x?}\n");
        GBL_LOG.write().unwrap().push_str(&s);
        COV_MEM.write().unwrap().entry(b).or_insert_with(|| {
            //println!("Cov @ {}", a-pe_base);
            GBL_COV.store(true, Ordering::SeqCst);
        });
        true
    })
    .unwrap();
    emu.add_code_hook(0, u64::MAX, |f, a, _b| {
        let s = format!("EXEC pe_base+{:x}, [{:x}, {:x}, {:x}]\n", a - pe_base,
            f.reg_read(RegisterARM::R0).unwrap(),
            f.reg_read(RegisterARM::R1).unwrap(),
            f.reg_read(RegisterARM::R2).unwrap(),
        );
        //eprint!("{s}");
        GBL_LOG.write().unwrap().push_str(&s);
        COV_PC
            .write()
            .unwrap()
            .entry(a - pe_base)
            .or_insert_with(|| {
                //println!("Cov @ {}", a-pe_base);
                GBL_COV.store(true, Ordering::SeqCst);
            });
    })
    .unwrap();

    let mut corpus = InputDb::default();
    corpus.corpus.push(EXAMPLE_CERT.to_vec());

    emu.mem_map(pe_base, 0x4000, Permission::ALL)
        .expect("failed to map code page");

    let ctx_base: u32 = 0x1100_0000;
    emu.mem_map(
        ctx_base as _,
        0x4000 as _,
        Permission::READ | Permission::WRITE,
    )
    .expect("failed to map code page");

    let input_data: u32 = 0x2000_0000;
    emu.mem_map(input_data as _, 0x4000 as _, Permission::READ)
        .expect("failed to map code page");

    // Code cave for hooks
    let hook_code_cave_stdlib: u32 = 0x2222_0000;
    emu.mem_map(hook_code_cave_stdlib as _, 0x4000 as _, Permission::ALL)
        .expect("failed to map code page");

    let hook_code_cave_memory_alloc: u32 = 0x2223_0000;
    emu.mem_map(hook_code_cave_memory_alloc as _, 0x4000 as _, Permission::ALL)
        .expect("failed to map code page");

    let hook_code_cave_sha1: u32 = 0x2224_0000;
    emu.mem_map(hook_code_cave_sha1 as _, 0x4000 as _, Permission::ALL)
        .expect("failed to map code page");

    let sp_base: u64 = 0x1000_0000;
    let sp_size: u64 =    0x100000;
    emu.mem_map(sp_base, sp_size as _, Permission::READ | Permission::WRITE)
        .expect("failed to map code page");

    // Used so that not impl'ing a hook results in a unmapped err
    // Set memset import to a code cave
    for i in 0u32..50 {
        emu.mem_write(hook_code_cave_stdlib as u64 + (i * 4) as u64, (hook_code_cave_stdlib + i * 4).to_le_bytes().as_slice()).unwrap();
        emu.mem_write(hook_code_cave_memory_alloc as u64 + (i * 4) as u64, (hook_code_cave_memory_alloc + i * 4).to_le_bytes().as_slice()).unwrap();
        emu.mem_write(hook_code_cave_sha1 as u64 + (i * 4) as u64, (hook_code_cave_sha1 + i * 4).to_le_bytes().as_slice()).unwrap();
    }

    // Hook that code cave and impl memset
    emu.add_code_hook(
        hook_code_cave_stdlib as u64,
        hook_code_cave_stdlib as u64 + 0x4000,
        |f, _a, _b| {
            let op = f.pc_read().unwrap() & 0xFFFF;

            match op {
                0xc0 => {
                    let out = f.reg_read(RegisterARM::R0).unwrap();
                    let count = f.reg_read(RegisterARM::R1).unwrap();
                    let val = f.reg_read(RegisterARM::R2).unwrap() as u32;

                    let s = format!("memset({out:x}, {count}, {val})\n");
                   // eprintln!("{s}");
                    GBL_LOG.write().unwrap().push_str(&s);

                    for i in 0..count {
                        f.mem_write(out + i, val.to_le_bytes().as_slice()).unwrap()
                    }
                }
                0xbc => {
                    let dst = f.reg_read(RegisterARM::R0).unwrap();
                    let src = f.reg_read(RegisterARM::R1).unwrap();
                    let count = f.reg_read(RegisterARM::R2).unwrap();

                    let s = format!("memcpy({dst:x}, {src:x}, {count})\n");
                    GBL_LOG.write().unwrap().push_str(&s);

                    for i in 0u64..count {
                        let mut tmp = [0u8];
                        f.mem_read(src + i, &mut tmp).unwrap();
                        f.mem_write(dst + i, &tmp).unwrap()
                    }
                }
                _ => unimplemented!("Unhooked stdlib op {:x}", op),
            }

            let lr = f.reg_read(RegisterARM::LR).unwrap();
            f.set_pc(lr).unwrap();
        },
    )
    .unwrap();

    emu.add_code_hook(
        hook_code_cave_memory_alloc as u64,
        hook_code_cave_memory_alloc as u64 + 0x4000,
        |f, _a, _b| {
            let op = f.pc_read().unwrap() & 0xFFFF;

            match op {
                0xc => {
                    let _self = f.reg_read(RegisterARM::R0).unwrap();
                    let count = f.reg_read(RegisterARM::R1).unwrap();
                    let _align = f.reg_read(RegisterARM::R2).unwrap() as u32;

                    let addr = MALLOC_ADDRS.write().unwrap().alloc(count, f);

                    let s = format!("malloc({count:x}, {_align})\n");
                    GBL_LOG.write().unwrap().push_str(&s);

                    f.reg_write(RegisterARM::R0, addr).unwrap();
                }
                0x10 => {
                    let _self = f.reg_read(RegisterARM::R0).unwrap();
                    let p = f.reg_read(RegisterARM::R1).unwrap();

                    MALLOC_ADDRS.write().unwrap().free(p, f);


                    //eprintln!("free({p:x})");
                }
                _ => unimplemented!("Unhooked mem op {:x}", op),
            }

            let lr = f.reg_read(RegisterARM::LR).unwrap();
            f.set_pc(lr).unwrap();
        },
    ).unwrap();

    emu.add_code_hook(
        hook_code_cave_sha1 as u64,
        hook_code_cave_sha1 as u64 + 0x4000,
        |f, _a, _b| {
            let op = f.pc_read().unwrap() & 0xFFFF;

            match op  {
                0 => {
                    //TODO: do something here
                    //eprintln!("sha1()");

                    f.reg_write(RegisterARM::R0, 0).unwrap();
                }
                _ => unimplemented!("Unhooked sha1 op {:x}", op),
            }

            let lr = f.reg_read(RegisterARM::LR).unwrap();
            f.set_pc(lr).unwrap();
        },
    ).unwrap();

    let mut r = thread_rng();

    let mut mutator = basic_mutator::Mutator::new()
        .seed(r.gen::<u64>() + 1024)
        .max_input_size(EXAMPLE_CERT.len() * 2)
        .printable(false);

    mutator.input = vec![0u8; EXAMPLE_CERT.len() * 2];


    loop {
        emu.mem_write(pe_base, &pe_byte)
            .expect("failed to write instructions");

        emu.mem_write(pe_base + 0x1c04, hook_code_cave_stdlib.to_le_bytes().as_slice())
            .unwrap();
        emu.mem_write(pe_base + 0x1b48, hook_code_cave_memory_alloc.to_le_bytes().as_slice())
            .unwrap();
        emu.mem_write(pe_base + 0x1b50, hook_code_cave_sha1.to_le_bytes().as_slice())
            .unwrap();

        {
            let mut r = reloc_sec;
            loop {
                let (j, block_base) = le_u32(r);
                let (j, len) = le_u32(j);
                let (j, body) = take(j, len as usize - 8);

                for ent in body.chunks_exact(2) {
                    let ent = u16::from_le_bytes(ent.try_into().unwrap());
                    if ent == 0 {
                        continue;
                    }

                    let typ = ent >> 12;
                    assert_eq!(typ, 3);
                    let off = ent & 0xFFF;

                    let virt_addr = pe_base + block_base as u64 + off as u64;

                    let mut tmp = [0u8; 4];
                    emu.mem_read(virt_addr, &mut tmp).unwrap();
                    let old_val = u32::from_le_bytes(tmp);
                    let new_val = (old_val as u64 + pe_base) as u32;
                    //println!("Ent ({:x}) = @ pe_base+{:x} ({virt_addr:x}), old = {:x}, new = {:x}", ent, block_base + off as u32, old_val, new_val);
                    emu.mem_write(virt_addr, &new_val.to_le_bytes()).unwrap();
                }

               // println!("Reloc {:x}", block_base);
              //  println!("Reloc {:x}", len);

                r = j;

                if r.len() < 8 {
                    break;
                }
            }
        }


        emu.reg_write(RegisterARM::SP as i32, sp_base + sp_size - 0x100)
            .expect("failed write R5");
        emu.reg_write(RegisterARM::LR as i32, 0xDEADBEE0)
            .expect("failed write R5");

        MALLOC_ADDRS.write().unwrap().reset(emu);

        mutator.input.clear();

        let rng = r.gen_range(0..corpus.num_inputs());
        let rng = corpus.input(rng).unwrap();
        mutator.input.extend_from_slice(rng);

        mutator.mutate(r.gen::<usize>() % 10, &corpus);

        emu.mem_write(input_data as _, &mutator.input).unwrap();


        // Hitting cert chain
        emu.reg_write(RegisterARM::R0 as i32, input_data as _)
            .unwrap();
        emu.reg_write(RegisterARM::R1 as i32, mutator.input.len() as _)
            .unwrap();
        emu.reg_write(RegisterARM::R2 as i32, ctx_base as u64)
            .unwrap();

        let r = emu.emu_start(pe_base + 0x6c8 + 1, 0, 10 * SECOND_SCALE, 1000);
        if let Err(_x) = r {
            let pc = emu.pc_read().unwrap();
            // Magic for returned with error
            if pc != 0xDEADBEE0 {
                // println!("Found err = {:?}", x);
                println!("pc = {:x?}", pc);
                println!("LOG:");
                println!("{}", GBL_LOG.read().unwrap());
                std::fs::write("bug_cert.bin", &mutator.input).unwrap();
                return;
            }
        }

        /*println!("LOG:");
        println!("{}", GBL_LOG.read().unwrap());
        // println!("r = {r:?}");
        let r0 = emu.reg_read(RegisterARM::R0).unwrap() as i32;
        println!("{:x?}", r0);*/

        GBL_LOG.write().unwrap().clear();
        if GBL_COV.swap(false, Ordering::SeqCst) {
            corpus.corpus.push(mutator.input.to_vec());
            println!("corpus");
            //return;

            std::fs::write(&format!("./corpus/{:x}.bin", md5::compute(&mutator.input)), &mutator.input).unwrap();

            std::fs::write("./corpus/cov-pc.json", serde_json::to_string(&*COV_PC.read().unwrap()).unwrap()).unwrap();
            std::fs::write("./corpus/cov-mem.json", serde_json::to_string(&*COV_MEM.read().unwrap()).unwrap()).unwrap();
        }
    }
}

#[derive(Default)]
struct InputDb {
    corpus: Vec<Vec<u8>>,
}

impl basic_mutator::InputDatabase for InputDb {
    fn num_inputs(&self) -> usize {
        self.corpus.len()
    }

    fn input(&self, idx: usize) -> Option<&[u8]> {
        self.corpus.get(idx).map(|f| f.as_slice())
    }
}