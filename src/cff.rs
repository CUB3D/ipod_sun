use parse::{be_u16, be_u32, ne_u8, take_n, ParseBytes};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CffHead {
    major: u8,
    minor: u8,
    offset_size: u8,
}

impl CffHead {
    fn write(&self, out: &mut Vec<u8>) {
        out.push(self.major); // major
        out.push(self.minor); // minor
        out.push(4); // head size
        out.push(self.offset_size); // offsize
    }
}

impl<'a> ParseBytes<'a> for CffHead {
    fn parse(i: &'a [u8]) -> Result<(&'a [u8], Self), ()> {
        let (i, major) = ne_u8(i);
        let (i, minor) = ne_u8(i);
        let (i, head_size) = ne_u8(i);
        let (i, offset_size) = ne_u8(i);
        assert_eq!(head_size, 4);

        Ok((
            i,
            Self {
                minor,
                major,
                offset_size,
            },
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CffIndex {
    off_size: u8,
    pub data: Vec<Vec<u8>>,
}

impl CffIndex {
    fn write(&self, i: &mut Vec<u8>) {
        i.extend_from_slice((self.data.len() as u16).to_be_bytes().as_slice());

        if self.data.is_empty() {
            return;
        }

        i.push(self.off_size);

        let mut c: u32 = 1;

        for ii in 0..self.data.len() {
            match self.off_size {
                4 => {
                    i.extend_from_slice(c.to_be_bytes().as_slice());
                }
                1 => {
                    i.push(c as u8);
                }
                _ => panic!(),
            }

            c += self.data[ii].len() as u32;
        }
        // Last one is to be able to compute length of last one on read
        match self.off_size {
            4 => {
                i.extend_from_slice(c.to_be_bytes().as_slice());
            }
            1 => {
                i.push(c as u8);
            }
            _ => panic!(),
        }

        for dat in &self.data {
            i.extend_from_slice(dat);
        }
    }
}

impl<'a> ParseBytes<'a> for CffIndex {
    fn parse(i: &'a [u8]) -> Result<(&'a [u8], Self), ()> {
        let (i, count) = be_u16(i);
        if count == 0 {
            return Ok((
                i,
                Self {
                    data: Vec::new(),
                    off_size: 0,
                },
            ));
        }
        let (i, off_size) = ne_u8(i);
        //println!("off sz = {off_size}");

        let mut offsets = Vec::new();
        let mut i = i;
        for _ in 0..count + 1 {
            let (j, v) = match off_size {
                1 => {
                    let (j, v) = ne_u8(i);
                    (j, v as u64)
                }
                2 => {
                    let (j, v) = be_u16(i);
                    (j, v as u64)
                }
                4 => {
                    let (j, v) = be_u32(i);
                    (j, v as u64)
                }
                _ => panic!(),
            };
            offsets.push(v);
            i = j;
        }

        let mut data = Vec::new();
        for idx in 0..count as usize {
            let len = offsets[idx + 1] - offsets[idx];
            let _start = offsets[idx] - 1;
            let (j, dat) = take_n(i, len as usize);
            data.push(dat.to_vec());
            i = j;
        }

        Ok((i, Self { data, off_size }))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CffFile {
    pub hed: CffHead,
    pub name_idx: CffIndex,
    pub font_dict_idx: CffIndex,
    pub string_idx: CffIndex,
    pub global_subrs_idx: CffIndex,
    pub charstrings_idx: CffIndex,
}
impl CffFile {
    pub fn write(&self, i: &mut Vec<u8>) {
        self.hed.write(i);
        self.name_idx.write(i);
        self.font_dict_idx.write(i);
        self.string_idx.write(i);
        self.global_subrs_idx.write(i);
        self.charstrings_idx.write(i);
    }
}
impl<'a> ParseBytes<'a> for CffFile {
    fn parse(i: &'a [u8]) -> Result<(&'a [u8], Self), ()> {
        let (i, hed) = CffHead::parse(i)?;
        let (i, name_idx) = CffIndex::parse(i)?;
        let (i, font_dict_idx) = CffIndex::parse(i)?;
        let (i, string_idx) = CffIndex::parse(i)?;
        let (i, global_subrs_idx) = CffIndex::parse(i)?;
        let (i, charstrings_idx) = CffIndex::parse(i)?;

        Ok((
            i,
            Self {
                hed,
                name_idx,
                font_dict_idx,
                string_idx,
                global_subrs_idx,
                charstrings_idx,
            },
        ))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Type1CharStringOp {
    EndChar,
    Random,
    Or,
    Index,
    Drop,
    Push(u32),
    Dup,
    Get,
    Exch,
    Eq,
    Sub,
    Put,
}

impl Type1CharStringOp {
    fn write(&self, i: &mut Vec<u8>) {
        match self {
            Type1CharStringOp::EndChar => {
                i.push(14u8);
            }
            Type1CharStringOp::Dup => {
                i.push(12u8);
                i.push(27u8);
            }
            Type1CharStringOp::Random => {
                i.push(12u8);
                i.push(23u8);
            }
            Type1CharStringOp::Push(x) => {
                let x = *x;

                i.push(0xFFu8);
                i.extend_from_slice(x.to_be_bytes().as_slice());
            }
            Type1CharStringOp::Or => {
                i.push(12u8);
                i.push(4u8);
            }
            Type1CharStringOp::Index => {
                i.push(12u8);
                i.push(29u8);
            }
            Type1CharStringOp::Get => {
                i.push(12u8);
                i.push(21u8);
            }
            Type1CharStringOp::Exch => {
                i.push(12u8);
                i.push(28u8);
            }
            Type1CharStringOp::Drop => {
                i.push(12u8);
                i.push(18u8);
            }
            Type1CharStringOp::Eq => {
                i.push(12u8);
                i.push(15u8);
            }
            Type1CharStringOp::Sub => {
                i.push(12u8);
                i.push(11u8);
            }
            Type1CharStringOp::Put => {
                i.push(12u8);
                i.push(20u8);
            } // _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct Type1CharString {
    pub ops: Vec<Type1CharStringOp>,
}

impl Type1CharString {
    pub fn write(&self, i: &mut Vec<u8>) {
        for op in &self.ops {
            op.write(i);
        }
    }
}

impl<'a> ParseBytes<'a> for Type1CharString {
    fn parse(i: &'a [u8]) -> Result<(&'a [u8], Self), ()>
    where
        Self: Sized,
    {
        let mut ops = Vec::new();

        let mut i = i;
        loop {
            let (j, op) = ne_u8(i);
            i = j;
            //println!("op = {op}");

            let val;

            if op >= 32 || op == 28 {
                if op == 28 {
                    panic!()
                } else if op < 247 {
                    panic!()
                } else if op < 251 {
                    panic!()
                } else if op < 255 {
                    let (j, v) = ne_u8(i);
                    i = j;

                    val = 0 - (op as u32 - 251) * 256 - v as u32 - 108;
                } else {
                    let (j, v) = be_u32(i);
                    i = j;

                    val = v;
                }

                ops.push(Type1CharStringOp::Push(val));

                continue;
            }

            let op = match op {
                14 => Type1CharStringOp::EndChar,
                12 => {
                    let (j, op2) = ne_u8(i);
                    i = j;
                    //println!("op2 = {op2}");

                    match op2 {
                        4 => Type1CharStringOp::Or,
                        18 => Type1CharStringOp::Drop,
                        29 => Type1CharStringOp::Index,
                        // Inaccurate TODO?
                        23 => Type1CharStringOp::Random,
                        _ => unimplemented!(),
                    }
                }
                _ => unimplemented!(),
            };
            //println!("op = {op:?}");
            ops.push(op);

            if let Type1CharStringOp::EndChar = op {
                break;
            }
        }

        Ok((i, Self { ops }))
    }
}
