use core::convert::TryInto;

pub fn le_u32(i: &[u8]) -> (&[u8], u32) {
    (&i[4..], u32::from_le_bytes((&i[..4]).try_into().unwrap()))
}

pub fn be_u32(i: &[u8]) -> (&[u8], u32) {
    (&i[4..], u32::from_be_bytes((&i[..4]).try_into().unwrap()))
}

pub fn le_u16(i: &[u8]) -> (&[u8], u16) {
    (&i[2..], u16::from_le_bytes((&i[..2]).try_into().unwrap()))
}

pub fn be_u16(i: &[u8]) -> (&[u8], u16) {
    (&i[2..], u16::from_be_bytes((&i[..2]).try_into().unwrap()))
}

pub fn ne_u8(i: &[u8]) -> (&[u8], u8) {
    (&i[1..], i[0])
}

pub fn take_n(i: &[u8], count: usize) -> (&[u8], &[u8]) {
    (&i[count..], &i[..count])
}

pub fn take<const COUNT: usize>(b: &[u8]) -> (&[u8], [u8;COUNT]) {
    (&b[COUNT..], b[..COUNT].try_into().unwrap())
}

pub trait ParseBytes<'a> {
    fn parse(i: &'a [u8]) -> Result<(&'a [u8], Self), ()>   where Self:  Sized;
}

pub struct SliceWriter<'a>(&'a mut [u8], usize);

impl<'a> SliceWriter<'a> {
    pub fn new_from(data: &'a mut [u8]) -> SliceWriter<'a> {
        Self(data, 0)
    }

    pub fn len_written(&self) -> usize { self.1}

    pub fn as_slice_mut(&mut self) -> &mut[u8] {
        self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0[..self.1]
    }

    pub fn put(&mut self, data: &[u8]) {
        (&mut self.0[self.1..self.1+data.len()]).copy_from_slice(data);
        self.1 += data.len();
    }

    pub fn le_u16(&mut self, v: u16) {
        self.put(&v.to_le_bytes());
    }

    pub fn be_u16(&mut self, v: u16) {
        self.put(&v.to_be_bytes());
    }

    pub fn be_u32(&mut self, v: u32) {
        self.put(&v.to_be_bytes());
    }

    pub fn ne_u8(&mut self, v: u8) {
        self.put(&v.to_le_bytes());
    }
}

pub trait GenerateBytes {
    fn generate<'a, 'b>(&'b self, i: &'b mut SliceWriter<'a>);

    /// How much data will be produced?
    fn generated_size(&self) -> usize;
}
