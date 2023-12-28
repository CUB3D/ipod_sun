use crate::cff::Type1CharStringOp;

pub struct CffPayloadBuilder<'a> {
    pub ops: &'a mut Vec<Type1CharStringOp>,
}

impl<'a> CffPayloadBuilder<'a> {
    pub fn index_write(&mut self, idx: u16, value: u32) {
        self.ops.push(Type1CharStringOp::Push(value));
        self.ops.push(Type1CharStringOp::Push((idx as u32) << 16));
        self.ops.push(Type1CharStringOp::Put);
    }

    pub fn index_write_array(&mut self, idx: u16, value: [u8; 4]) {
        self.ops
            .push(Type1CharStringOp::Push(u32::from_le_bytes(value)));
        self.ops.push(Type1CharStringOp::Push((idx as u32) << 16));
        self.ops.push(Type1CharStringOp::Put);
    }
}
