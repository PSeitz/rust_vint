use std::mem::transmute;
use util::*;

#[derive(Debug, Clone, Default)]
pub struct VIntArray{
    pub data: Vec<u8>,
}

impl VIntArray {
    pub fn encode(&mut self, val:u32) {
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
        }else{
            push_n_set(false);
            push_n_set(false);
            push_n_set(false);
            push_n_set(true);
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
        add_val >>= offset;
        *val |= add_val;

        has_more
    }

    pub fn iter(& self) -> VintArrayIterator {
        VintArrayIterator {
            list: &self,
            pos: 0,
            len: self.data.len(),
        }
    }

}

#[derive(Debug, Clone)]
pub struct VintArrayIterator<'a>  {
    list: & 'a VIntArray,
    pos:usize,
    len:usize
}

impl<'a> Iterator for VintArrayIterator<'a> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        if self.pos == self.len {
            None
        }else {
            let (val_u8, has_more) = self.list.decode_u8(self.pos);
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