use std::io;
use std::io::Read;
use std::io::Write;
use std::iter::FusedIterator;

#[inline(always)]
pub fn encode_varint_into(output: &mut Vec<u8>, mut value: u32) {
    let do_one = |output: &mut Vec<u8>, value: &mut u32| {
        output.push(((*value & 127) | 128) as u8);
        *value >>= 7;
    };
    let do_last = |output: &mut Vec<u8>, value: u32| {
        output.push((value & 127) as u8);
    };

    if value < 1 << 7 {
        //128
        do_last(output, value);
    } else if value < 1 << 14 {
        do_one(output, &mut value);
        do_last(output, value);
    } else if value < 1 << 21 {
        do_one(output, &mut value);
        do_one(output, &mut value);
        do_last(output, value);
    } else if value < 1 << 28 {
        do_one(output, &mut value);
        do_one(output, &mut value);
        do_one(output, &mut value);
        do_last(output, value);
    } else {
        do_one(output, &mut value);
        do_one(output, &mut value);
        do_one(output, &mut value);
        do_one(output, &mut value);
        do_last(output, value);
    }
}

#[inline(always)]
pub fn encode_varint_into_writer<W: Write>(mut output: W, mut value: u32) -> Result<(), io::Error> {
    let do_one = |output: &mut W, value: &mut u32| -> Result<(), io::Error> {
        output.write_all(&[((*value & 127) | 128) as u8])?;
        *value >>= 7;
        Ok(())
    };
    let do_last = |output: &mut W, value: u32| -> Result<(), io::Error> {
        output.write_all(&[(value & 127) as u8])?;
        Ok(())
    };

    if value < 1 << 7 {
        //128
        do_last(&mut output, value)?;
    } else if value < 1 << 14 {
        do_one(&mut output, &mut value)?;
        do_last(&mut output, value)?;
    } else if value < 1 << 21 {
        do_one(&mut output, &mut value)?;
        do_one(&mut output, &mut value)?;
        do_last(&mut output, value)?;
    } else if value < 1 << 28 {
        do_one(&mut output, &mut value)?;
        do_one(&mut output, &mut value)?;
        do_one(&mut output, &mut value)?;
        do_last(&mut output, value)?;
    } else {
        do_one(&mut output, &mut value)?;
        do_one(&mut output, &mut value)?;
        do_one(&mut output, &mut value)?;
        do_one(&mut output, &mut value)?;
        do_last(&mut output, value)?;
    }
    Ok(())
}

#[test]
fn test_varint() {
    let mut sink = vec![];
    encode_varint_into(&mut sink, 5);
    encode_varint_into(&mut sink, 127);
    encode_varint_into(&mut sink, 128);
    encode_varint_into(&mut sink, 50);
    encode_varint_into(&mut sink, 500);
    encode_varint_into(&mut sink, 5000);
    encode_varint_into(&mut sink, 4_000_000_000);
    assert_eq!(sink.len(), 14);

    let mut iter = sink.iter().cloned();
    assert_eq!(decode_varint(&mut iter), Some(5));
    assert_eq!(decode_varint(&mut iter), Some(127));
    assert_eq!(decode_varint(&mut iter), Some(128));
    assert_eq!(decode_varint(&mut iter), Some(50));
    assert_eq!(decode_varint(&mut iter), Some(500));
    assert_eq!(decode_varint(&mut iter), Some(5000));
    assert_eq!(decode_varint(&mut iter), Some(4_000_000_000));

    let iter = VintArrayIterator { data: &sink, pos: 0 };
    let dat: Vec<_> = iter.collect();
    assert_eq!(dat, vec![5, 127, 128, 50, 500, 5000, 4_000_000_000]);

    use std::io::BufReader;
    let mut reader = BufReader::new(&sink[..]);
    assert_eq!(decode_from_reader(reader.get_mut()), Some(5));
    assert_eq!(decode_from_reader(reader.get_mut()), Some(127));
    assert_eq!(decode_from_reader(reader.get_mut()), Some(128));
    assert_eq!(decode_from_reader(reader.get_mut()), Some(50));
    assert_eq!(decode_from_reader(reader.get_mut()), Some(500));
    assert_eq!(decode_from_reader(reader.get_mut()), Some(5000));
    assert_eq!(decode_from_reader(reader.get_mut()), Some(4_000_000_000));
    assert_eq!(decode_from_reader(reader.get_mut()), None);
}

#[inline(always)]
pub fn decode_varint<I: Iterator<Item = u8>>(input: &mut I) -> Option<u32> {
    let mut ret: u32 = 0;
    if let Some(next) = input.next() {
        ret |= u32::from(next) & 127;
        if next & 128 == 0 {
            return Some(ret);
        }
        let next = input.next().unwrap();
        ret |= (u32::from(next) & 127) << 7;
        if next & 128 == 0 {
            return Some(ret);
        }
        let next = input.next().unwrap();
        ret |= (u32::from(next) & 127) << 14;
        if next & 128 == 0 {
            return Some(ret);
        }
        let next = input.next().unwrap();
        ret |= (u32::from(next) & 127) << 21;
        if next & 128 == 0 {
            return Some(ret);
        }
        let next = input.next().unwrap();
        ret |= (u32::from(next) & 127) << 28;
        return Some(ret);
    } else {
        return None;
    }
}

#[inline(always)]
pub fn decode_from_reader<R: Read>(r: &mut R) -> Option<u32> {
    let mut iter = r.bytes().map(|el| el.unwrap());
    decode_varint(&mut iter)
}

#[derive(Debug, Clone)]
pub struct VintArrayIterator<'a> {
    pub data: &'a [u8],
    /// the current offset in the slice
    pub pos: usize,
}

impl<'a> VintArrayIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        VintArrayIterator { data, pos: 0 }
    }

    #[inline]
    pub fn from_slice(data: &'a [u8]) -> Self {
        let mut iter = VintArrayIterator::new(data);
        if let Some(size) = iter.next() {
            VintArrayIterator::new(&data[iter.pos..iter.pos + size as usize])
        } else {
            VintArrayIterator::new(&data[..0])
        }
    }
}

impl<'a> Iterator for VintArrayIterator<'a> {
    type Item = u32;

    #[inline(always)]
    fn next(&mut self) -> Option<u32> {
        if self.pos == self.data.len() {
            None
        } else {
            let next = self.data[self.pos];
            self.pos += 1;
            let mut ret: u32 = u32::from(next) & 127;
            if next & 128 == 0 {
                return Some(ret);
            }
            let next = self.data[self.pos];
            self.pos += 1;
            let mut shift_by = 7;
            ret |= (u32::from(next) & 127) << shift_by;
            if next & 128 == 0 {
                return Some(ret);
            }
            let next = self.data[self.pos];
            self.pos += 1;
            shift_by += 7;
            ret |= (u32::from(next) & 127) << shift_by;
            if next & 128 == 0 {
                return Some(ret);
            }
            let next = self.data[self.pos];
            self.pos += 1;
            shift_by += 7;
            ret |= (u32::from(next) & 127) << shift_by;
            if next & 128 == 0 {
                return Some(ret);
            }
            let next = self.data[self.pos];
            self.pos += 1;
            shift_by += 7;
            ret |= (u32::from(next) & 127) << shift_by;
            Some(ret)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.data.len() - self.pos / 2, Some(self.data.len() - self.pos))
    }
}

impl<'a> FusedIterator for VintArrayIterator<'a> {}

#[derive(Debug, Clone, Default)]
pub struct VIntArray {
    pub data: Vec<u8>,
}

impl VIntArray {
    #[inline]
    pub fn encode_vals(&mut self, vals: &[u32]) {
        for val in vals {
            encode_varint_into(&mut self.data, *val);
        }
    }

    #[inline]
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::with_capacity(self.data.len() + 4);
        encode_varint_into(&mut serialized, self.data.len() as u32);
        serialized.extend_from_slice(&self.data);
        serialized
    }

    #[inline(always)]
    pub fn encode(&mut self, val: u32) {
        encode_varint_into(&mut self.data, val);
    }

    pub fn iter(&self) -> VintArrayIterator {
        VintArrayIterator::new(&self.data)
    }
}

#[test]
fn test_encode_decode_vint_very_large_number() {
    let mut vint = VIntArray::default();
    let dat = vec![4_000_000_000];
    vint.encode_vals(&dat);
    let decoded_data: Vec<u32> = vint.iter().collect();
    assert_eq!(&dat, &decoded_data);
}

#[test]
fn test_serialize() {
    let mut vint = VIntArray::default();
    let dat = vec![4_000_000_000];
    vint.encode_vals(&dat);

    let data = vint.serialize();

    let iter = VintArrayIterator::from_slice(&data);
    let decoded_data: Vec<u32> = iter.collect();
    assert_eq!(&dat, &decoded_data);
}
