use std::mem::transmute;
use util::*;

#[derive(Debug, Clone, Default)]
pub struct VIntArray{
    pub data: Vec<u8>,
}

impl VIntArray {
    pub fn encode_vals(&mut self, vals: &[u32]) {
        for val in vals{
            self.encode(*val);
        }
    }

    pub fn encode_large(&mut self, val:u32) {
        let mut pos = 0;
        let mut el = val;
        let mut push_n_set = |last_block: bool|{
            if pos == 4 {
                let mut el_u64: u64 = val as u64;
                el_u64 <<= 4;
                let bytes: [u8; 8] = unsafe { transmute(el_u64) };
                self.data.push(bytes[pos]);
                return;
            }
            if pos > 0 {
                el <<= 1;
            }
            if last_block {
                let bytes: [u8; 4] = unsafe { transmute(el) };
                self.data.push(bytes[pos]);
            }else{
                let bytes: [u8; 4] = unsafe { transmute(el) };
                self.data.push(set_high_bit_u8(bytes[pos]));
            }
            pos +=1;
        };

        push_n_set(false);
        push_n_set(false);
        push_n_set(false);
        push_n_set(false);
        push_n_set(true);
    }

    pub fn encode(&mut self, val:u32) {
        if  val >= 1 << 28{
            self.encode_large(val);
            return;
        }
        let mut pos = 0;
        let mut el = val;
        let mut push_n_set = |last_block: bool|{
            if pos > 0 {
                el <<= 1;
            }
            if last_block {
                let bytes: [u8; 4] = unsafe { transmute(el) };
                self.data.push(bytes[pos]);
            }else{
                let bytes: [u8; 4] = unsafe { transmute(el) };
                self.data.push(set_high_bit_u8(bytes[pos]));
            }
            pos +=1;
        };

        if val < 1 << 7 { //128
            push_n_set(true);
        }else if val < 1 << 14 {
            push_n_set(false);
            push_n_set(true);
        }else if val < 1 << 21 {
            push_n_set(false);
            push_n_set(false);
            push_n_set(true);
        }else {
            push_n_set(false);
            push_n_set(false);
            push_n_set(false);
            push_n_set(true);
        }

    }

    pub fn iter(& self) -> VintArrayIterator {
        VintArrayIterator {
            data: &self.data,
            pos: 0,
        }
    }

}

#[derive(Debug, Clone)]
pub struct VintArrayIterator<'a>  {
    data: & 'a [u8],
    pos:usize
}

impl<'a> VintArrayIterator<'a> {
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
        }else {
            let (val_u8, has_more) = self.decode_u8(self.pos);
            self.pos += 1;
            let mut val = val_u8 as u32;
            if has_more{
                let has_more = self.get_apply_bits(self.pos, 1, &mut val);
                self.pos += 1;
                if has_more{
                    let has_more = self.get_apply_bits(self.pos, 2, &mut val);
                    self.pos += 1;
                    if has_more{
                        let has_more = self.get_apply_bits(self.pos, 3, &mut val);
                        self.pos += 1;
                        if has_more{
                            let el = unsafe{*self.data.get_unchecked(self.pos) };
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
        (self.data.len()-self.pos / 2, Some(self.data.len()-self.pos))
    }


}


#[test]
fn test_encode_decode_vint_very_large_number() {
    let mut vint = VIntArray::default();
    let dat = vec![4_000_000_000];
    vint.encode_vals(&dat);
    let decoded_data:Vec<u32> = vint.iter().collect();
    assert_eq!(&dat, &decoded_data);
}

