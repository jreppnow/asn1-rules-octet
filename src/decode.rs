use bytes::{Buf, BufMut};

pub fn decode_boolean(buffer: &mut impl Buf) -> bool {
     0 < buffer.get_u8()
}