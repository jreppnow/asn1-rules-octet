use bytes::BufMut;
use std::collections::HashSet;
use std::io::Write;
use std::ops::Bound::{Excluded, Included};
use std::ops::{Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeTo};

pub fn encode_boolean(buffer: &mut impl BufMut, value: bool) {
    if value {
        buffer.put_u8(1);
    } else {
        buffer.put_u8(0);
    }
}

pub fn write_length_encoding(buffer: &mut impl BufMut, length: usize) {
    if length < 0b1000_0000 {
        buffer.put_u8(length as u8);
    } else {
        let mut bytes_used: u8 = 0;
        {
            let mut length = length;
            while length > 0 {
                bytes_used += 1;
                length >>= 8;
            }
        }
        buffer.put_u8(0b1000_0000 | bytes_used);
        while bytes_used > 0 {
            buffer.put_u8(((length >> ((bytes_used - 1) as usize * 8)) & 0xFF) as u8);
            bytes_used -= 1;
        }
    }
}

pub enum Tag {
    Universal(usize),
    Application(usize),
    ContextSpecific(usize),
    Private(usize),
}

pub fn write_tag(buffer: &mut impl BufMut, tag: Tag) {
    let (first_bits, value) = match tag {
        Tag::Universal(value) => (0b0000_0000, value),
        Tag::Application(value) => (0b0100_0000, value),
        Tag::ContextSpecific(value) => (0b1000_0000, value),
        Tag::Private(value) => (0b1100_0000, value),
    };

    if value < 0b0100_0000 {
        buffer.put_u8(first_bits | value as u8);
    } else {
        buffer.put_u8(first_bits | 0b0011_1111);
        let mut byte_count = 0;
        {
            let mut value = value;
            while value > 0 {
                value >>= 7;
                byte_count += 1;
            }
        }
        while byte_count > 0 {
            byte_count -= 1;
            let bit: u8 = if byte_count > 0 { 0b1000_0000 } else { 0 };
            buffer.put_u8(bit | ((value >> (byte_count * 7)) & 0x0111_1111) as u8);
        }
    }
}

fn included(bounds: Bound<&i128>) -> Option<i128> {
    match bounds {
        Included(value) => Some(*value ),
        Excluded(value) => Some(value - 1),
        Bound::Unbounded => None,
    }
}

pub fn encode_int(buffer: &mut impl BufMut, value: isize, bounds: impl RangeBounds<i128>) {
    match included(bounds.start_bound()) {
        Some(start) if start >= 0 => {
            match included(bounds.end_bound()) {
                Some(end) if end <= u8::MAX as i128 => {
                    buffer.put_u8(value as u8);
                }
                Some(end) if end <= u16::MAX as i128 => {
                    buffer.put_u16(value as u16);
                }
                Some(end) if end <= u32::MAX as i128 => {
                    buffer.put_u32(value as u32);
                }
                Some(end) if end <= u64::MAX as i128 => {
                    buffer.put_u64(value as u64);
                }
                None => {}
            }
        },
        None =>
        Included(&lower) if lower >= 0 => match bounds.end_bound() {
            Included(&upper) if upper <= 0xFF => {
                buffer.put_u8(value as u8);
            }
            Excluded(&upper) if upper <= 0xFF + 1 => {}
            Unbounded => {}
        },
        Bound::Excluded(&lower) if lower >= -1 => {}
        Bound::Unbounded => {}
    }
}
