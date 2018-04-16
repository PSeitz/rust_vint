use fnv::FnvHashMap;
use std::mem::transmute;
use util::*;

#[derive(Debug, Clone, Default)]
pub struct VIntArrayEncodeMostCommon{
    pub data: Vec<u8>,
    pub most_common_val: Option<u32>
}

#[derive(Debug)]
enum BytesRequired {
    One = 1,
    Two,
    Three,
    Four,
}

fn get_bytes_required(val:u32) -> BytesRequired {
    if val < 1 << 6 { //64  1 byte for most_common, 1 bit to signal more
        BytesRequired::One
    }else if val < 1 << 13 {
        BytesRequired::Two
    }else if val < 1 << 20 {
        BytesRequired::Three
    }else{
        BytesRequired::Four
    }
}

impl VIntArrayEncodeMostCommon {

    pub fn get_space_used_by_most_common_val(&mut self, vals: &[u32]) -> (u32, u32) {
        // calculate needed size of value
        let val_by_size = vals.iter().fold(FnvHashMap::default(), |mut m, val| {
            *m.entry(*val).or_insert(0) += get_bytes_required(*val) as u32;
            m
        });

        let el_with_most_space = val_by_size.iter().max_by_key(|(_, val)| *val).map(|(key, val)| (*key, *val)).unwrap();
        el_with_most_space
    }

    pub fn encode_vals(&mut self, vals: &[u32]) {
        if vals.is_empty() {return;}
        if self.most_common_val.is_none() {
            let (most_common_val, _space_used) = self.get_space_used_by_most_common_val(vals);
            self.most_common_val = Some(most_common_val);
        }
        let mut iter = vals.iter().peekable();
        while let Some(val) = iter.next() {
            let mut move_iter = false;
            if let Some(next_val) = iter.peek() {
                if **next_val == self.most_common_val.unwrap() {
                    self.encode(*val, true);
                    move_iter = true;
                }else{
                    self.encode(*val, false);
                }
            }else{
                self.encode(*val, false);
            }
            if move_iter{iter.next();};  // move_iter, because next val is already encoded
        }

    }

    fn encode(&mut self, val:u32, next_is_most_common_val: bool) {
        let mut pos = 0;
        let mut el = val;
        let mut push_n_set = |last_block: bool|{
            let is_first_block = pos == 0;
            if pos > 0 {
                el <<= 1;
            }
            let mut byte = unsafe { transmute::<u32, [u8; 4]>(el)[pos] };
            if is_first_block {
                if next_is_most_common_val {
                    byte = set_second_high_bit_u8(byte);
                }else{
                    byte = unset_second_high_bit_u8(byte);
                }
            }
            if last_block {
                self.data.push(byte);
            }else{
                self.data.push(set_high_bit_u8(byte));
            }
            if pos == 0{
                el <<= 1;
            }
            pos +=1;
        };

        match get_bytes_required(val) {
            BytesRequired::One => {
                push_n_set(true);
            },
            BytesRequired::Two => {
                push_n_set(false);
                push_n_set(true);
            },
            BytesRequired::Three => {
                push_n_set(false);
                push_n_set(false);
                push_n_set(true);
            },
            BytesRequired::Four => {
                push_n_set(false);
                push_n_set(false);
                push_n_set(false);
                push_n_set(true);
            },
        }

    }

    #[inline]
    pub fn decode_u8(&self, pos:usize) -> (u8, bool) {
        unsafe{
            let el = *self.data.get_unchecked(pos);
            if is_high_bit_set(el){
                (unset_high_bit_u8(el), true)
            }else{
                (el, false)
            }
        }
    }

    #[inline]
    fn get_apply_bits(&self, pos:usize, offset:usize, val: &mut u32) -> bool {
        let (val_u8, has_more) = self.decode_u8(pos);

        let mut bytes: [u8; 4] = [0, 0, 0, 0];
        bytes[offset] = val_u8;
        let mut add_val: u32 = unsafe { transmute(bytes) };
        add_val >>= offset + 1;
        *val |= add_val;

        has_more
    }

    pub fn iter(& self) -> VintArrayMostCommonIterator {
        VintArrayMostCommonIterator {
            list: &self,
            pos: 0,
            len: self.data.len(),
            next_val: None,
            most_common_val: self.most_common_val.unwrap_or(0)
        }
    }

}

#[derive(Debug, Clone)]
pub struct VintArrayMostCommonIterator<'a>  {
    list: & 'a VIntArrayEncodeMostCommon,
    pos:usize,
    len:usize,
    next_val: Option<u32>,
    most_common_val: u32
}

impl<'a> Iterator for VintArrayMostCommonIterator<'a> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        if let Some(next_val) = self.next_val {
            self.next_val = None;
            return Some(next_val);
        }

        if self.pos == self.len {
            None
        }else {
            let (mut val_u8, has_more) = self.list.decode_u8(self.pos);
            if is_second_high_bit_set(val_u8){
                val_u8 = unset_second_high_bit_u8(val_u8);
                self.next_val = Some(self.most_common_val);
            }
            self.pos += 1;
            let mut val = val_u8 as u32;
            if has_more{
                let has_more = self.list.get_apply_bits(self.pos, 1, &mut val);
                self.pos += 1;
                if has_more{
                    let has_more = self.list.get_apply_bits(self.pos, 2, &mut val);
                    self.pos += 1;
                    if has_more{
                        self.list.get_apply_bits(self.pos, 3, &mut val);
                        self.pos += 1;
                    }
                }
            }
            Some(val)
        }

    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len-self.pos / 2, Some(self.len-self.pos))
    }

}

#[test]
fn test_encode_decode_vint_most_common_1000() {
    let mut vint = VIntArrayEncodeMostCommon::default();
    let dat = vec![1000, 1000, 1000];
    vint.encode_vals(&dat);
    let decoded_data:Vec<u32> = vint.iter().collect();
    assert_eq!(&dat, &decoded_data);
}

#[test]
fn test_encode_decode_vint_most_common_single() {
    let mut vint = VIntArrayEncodeMostCommon::default();
    let dat = vec![10];
    vint.encode_vals(&dat);
    let decoded_data:Vec<u32> = vint.iter().collect();
    assert_eq!(&dat, &decoded_data);
}

