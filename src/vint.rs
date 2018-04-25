use serde::ser::{Serialize, Serializer, SerializeTuple};
use std::mem::transmute;
use util::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct VIntArray {
    // #[serde(serialize_with = "serialize_data")]
    pub data: Vec<u8>,
}


impl Serialize for VIntArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(self.data.len())?;
        seq.serialize_element(&(self.data.len() as u32))?;
        for element in &self.data[..] {
            seq.serialize_element(element)?;
        }
        seq.end()
    }
}

/// Only split for performance reasons
#[inline]
pub fn push_n_set_big(val: u32, data: &mut [u8]) {
    let mut el_u64: u64 = val as u64;
    el_u64 <<= 4;
    let bytes: [u8; 8] = unsafe { transmute(el_u64) };
    data[4] = bytes[4];
}

#[inline]
pub fn push_n_set(last_block: bool, el: &mut u32, pos: &mut u8, data: &mut [u8]) {
    if *pos > 0 {
        *el <<= 1;
    }
    if last_block {
        let bytes: [u8; 4] = unsafe { transmute(*el) };
        data[*pos as usize] = bytes[*pos as usize];
    } else {
        let bytes: [u8; 4] = unsafe { transmute(*el) };
        data[*pos as usize] = set_high_bit_u8(bytes[*pos as usize]);
    }
    *pos += 1;
}

#[inline]
pub fn encode_num(val: u32) -> ([u8; 8], u8) {
    let mut el = val;

    let mut data = [0, 0, 0, 0, 0, 0, 0, 0];
    let mut pos: u8 = 0;

    if val < 1 << 7 {
        //128
        push_n_set(true, &mut el, &mut pos, &mut data);
    } else if val < 1 << 14 {
        push_n_set(false, &mut el, &mut pos, &mut data);
        push_n_set(true, &mut el, &mut pos, &mut data);
    } else if val < 1 << 21 {
        push_n_set(false, &mut el, &mut pos, &mut data);
        push_n_set(false, &mut el, &mut pos, &mut data);
        push_n_set(true, &mut el, &mut pos, &mut data);
    } else if val < 1 << 28 {
        push_n_set(false, &mut el, &mut pos, &mut data);
        push_n_set(false, &mut el, &mut pos, &mut data);
        push_n_set(false, &mut el, &mut pos, &mut data);
        push_n_set(true, &mut el, &mut pos, &mut data);
    } else {
        push_n_set(false, &mut el, &mut pos, &mut data);
        push_n_set(false, &mut el, &mut pos, &mut data);
        push_n_set(false, &mut el, &mut pos, &mut data);
        push_n_set(false, &mut el, &mut pos, &mut data);
        push_n_set_big(val, &mut data);
        pos += 1;
    }
    (data, pos)
}

impl VIntArray {
    #[inline]
    pub fn encode_vals(&mut self, vals: &[u32]) {
        for val in vals {
            self.encode(*val);
        }
    }

    #[inline]
    /// decodes data from a slice and returns the total size of the data in the slice in bytes
    pub fn decode_from_slice(data: &[u8]) -> (Vec<u32>, u32) {
        let mut iter = VintArrayIterator::new(data); // the first two, are encoded as normal vint
        if let Some(size) = iter.next() {
            let mut iter_data = VintArrayIterator::new(&data[iter.pos .. iter.pos + size as usize]);
            let mut data = vec![];
            while let Some(el) = iter_data.next() {
                data.push(el);
            }
            (data, iter.pos as u32 + iter_data.pos as u32)
        }else{
            (vec![], iter.pos as u32)
        }
    }

    #[inline]
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::with_capacity(self.data.len() + 4);
        push_compact(self.data.len() as u32, &mut serialized);
        serialized.extend_from_slice(&self.data);
        serialized
    }

    #[inline]
    pub fn encode(&mut self, val: u32) {
        let (slice, len) = encode_num(val);
        self.data.extend_from_slice(&slice[..len as usize]);
    }

    pub fn iter(&self) -> VintArrayIterator {
        VintArrayIterator::new(&self.data)
    }
}

#[derive(Debug, Clone)]
pub struct VintArrayIterator<'a> {
    pub data: &'a [u8],
    /// the current offset in the slice
    pub pos: usize,
}

impl<'a> VintArrayIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        VintArrayIterator { data: data, pos: 0 }
    }

    #[inline]
    pub fn from_slice(data:&'a [u8]) -> Self {
        let mut iter = VintArrayIterator::new(data);
        if let Some(size) = iter.next() {
            VintArrayIterator::new(&data[iter.pos .. iter.pos + size as usize])
        }else{
            VintArrayIterator::new(&data[..0])
        }
    }

    #[inline]
    fn decode_u8(&self, pos: usize) -> (u8, bool) {
        unsafe {
            let el = *self.data.get_unchecked(pos);
            if is_high_bit_set(el) {
                (unset_high_bit_u8(el), true)
            } else {
                (el, false)
            }
        }
    }

    #[inline]
    fn get_apply_bits(&self, pos: usize, offset: usize, val: &mut u32) -> bool {
        let (val_u8, has_more) = self.decode_u8(pos);

        let mut bytes: [u8; 4] = [0, 0, 0, 0];
        bytes[offset] = val_u8;
        let mut add_val: u32 = unsafe { transmute(bytes) };
        add_val >>= offset;
        *val |= add_val;

        has_more
    }
}
impl<'a> Iterator for VintArrayIterator<'a> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        if self.pos == self.data.len() {
            None
        } else {
            let (val_u8, has_more) = self.decode_u8(self.pos);
            self.pos += 1;
            let mut val = val_u8 as u32;
            if has_more {
                let has_more = self.get_apply_bits(self.pos, 1, &mut val);
                self.pos += 1;
                if has_more {
                    let has_more = self.get_apply_bits(self.pos, 2, &mut val);
                    self.pos += 1;
                    if has_more {
                        let has_more = self.get_apply_bits(self.pos, 3, &mut val);
                        self.pos += 1;
                        if has_more {
                            let el = unsafe { *self.data.get_unchecked(self.pos) };
                            let bytes: [u8; 4] = [0, 0, 0, el];
                            let mut add_val: u32 = unsafe { transmute(bytes) };
                            add_val <<= 4;
                            val |= add_val as u32;
                            self.pos += 1;
                        }
                    }
                }
            }
            Some(val)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.data.len() - self.pos / 2,
            Some(self.data.len() - self.pos),
        )
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


