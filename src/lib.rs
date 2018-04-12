#![feature(test)]
extern crate test;
extern crate rand;

#[repr(C)]
union VIntData {
    bytes: [u8; 4],
    val: u32,
}

#[derive(Debug, Clone, Default)]
pub struct VIntArrayFixed{
    pub data: Vec<u8>,
}

impl VIntArrayFixed {
    pub fn encode(&mut self, val:u32) {
        unsafe{

            let mut el = VIntData{val};
            el.val <<= 2; // shift two to the left, to make space for the signal flags

            if val < 1 << 6 { //64
                self.data.push(el.bytes[0]);
            }else if val < 1 << 14 {
                el.val |= 1;
                self.data.push(el.bytes[0]);
                self.data.push(el.bytes[1]);
            }else if val < 1 << 22 {
                el.val |= 2;
                self.data.push(el.bytes[0]);
                self.data.push(el.bytes[1]);
                self.data.push(el.bytes[2]);
            }else{
                el.val |= 3;
                self.data.push(el.bytes[0]);
                self.data.push(el.bytes[1]);
                self.data.push(el.bytes[2]);
                self.data.push(el.bytes[3]);
            };

        }

    }

    pub fn iter(& self) -> VintArrayFixedIterator {
        VintArrayFixedIterator {
            list: &self,
            pos: 0,
            len: self.data.len(),
        }
    }

}


#[derive(Debug, Clone)]
pub struct VintArrayFixedIterator<'a>  {
    list: & 'a VIntArrayFixed,
    pos:usize,
    len:usize
}

impl<'a> Iterator for VintArrayFixedIterator<'a> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        unsafe{
            if self.pos == self.len {
                None
            }else {

                let mut val = *self.list.data.get_unchecked(self.pos);
                let flags:u8 = val & 0b0000_0011;

                if flags == 0 {
                    val >>= 2;
                    self.pos += 1;
                    Some(val as u32)
                } else if flags == 1 {
                    let mut el = VIntData{val: 0};
                    el.bytes[..=1].copy_from_slice(&self.list.data[self.pos..=self.pos+1]);
                    el.val >>= 2;
                    self.pos += 2;
                    Some(el.val)
                } else if flags == 2 {
                    let mut el = VIntData{val: 0};
                    el.bytes[..=2].copy_from_slice(&self.list.data[self.pos..=self.pos+2]);
                    el.val >>= 2;
                    self.pos += 3;
                    Some(el.val)
                }else {
                    let mut el = VIntData{val: 0};
                    el.bytes[..=3].copy_from_slice(&self.list.data[self.pos..=self.pos+3]);
                    el.val >>= 2;
                    self.pos += 4;
                    Some(el.val)
                }
            }
        }

    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len-self.pos / 2, Some(self.len-self.pos))
    }

    #[inline]
    fn count(self) -> usize {
        self.list.data.len()
    }

}


#[test]
fn test_encode_decode_vint_array() {
    let mut vint = VIntArrayFixed::default();
    vint.encode(50);
    vint.encode(120);

    vint.encode(200);
    vint.encode(2000);
    vint.encode(70000);
    vint.encode(3_000_000);

    let mut iter = vint.iter();
    assert_eq!(iter.next().unwrap(), 50);
    assert_eq!(iter.next().unwrap(), 120);
    assert_eq!(iter.next().unwrap(), 200);
    assert_eq!(iter.next().unwrap(), 2000);
    assert_eq!(iter.next().unwrap(), 70000);
    assert_eq!(iter.next().unwrap(), 3_000_000);
}


#[derive(Debug, Clone, Default)]
pub struct VIntArray{
    pub data: Vec<u8>,
}

impl VIntArray {
    pub fn encode(&mut self, val:u32) {
        unsafe{
            let mut pos = 0;
            let mut el = VIntData{val};
            let mut push_n_set = |last_block: bool|{
                if pos > 0 {
                    el.val <<= 1;
                }
                if last_block {
                    self.data.push(el.bytes[pos]);
                }else{
                    self.data.push(set_high_bit_u8(el.bytes[pos]));
                }
                pos +=1;
            };

            // let bytes: [u8; 4] = unsafe { transmute(el as u32) };
            if val < 1<< 7 { //128
                // self.data.push(el.bytes[0]);
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

    }

    #[inline]
    pub fn decode_u8(&self, pos:usize) -> (u8, bool) {
        unsafe{
            let el = *self.data.get_unchecked(pos);
            // if is_high_bit_set(*self.data.get_unchecked(pos)){
            if el >= 1 << 7{
                (unset_high_bit_u8(el), true)
            }else{
                (el, false)
            }
        }
    }
    #[inline]
    fn get_apply_bits(&self, pos:usize, offset:usize, val: &mut u32) -> bool {
        let (val_u8, has_more) = self.decode_u8(pos);
        let mut el = VIntData{bytes: [0, 0, 0, 0]};
        unsafe{
            el.bytes[offset] = val_u8;
            el.val >>= offset;
            *val |= el.val;
        }
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

#[test]
fn test_encode_decode_vint() {
    let mut vint = VIntArray::default();
    vint.encode(110);
    vint.encode(120);

    vint.encode(200);
    vint.encode(2000);
    vint.encode(70000);
    vint.encode(3_000_000);

    let mut iter = vint.iter();
    assert_eq!(iter.next().unwrap(), 110);
    assert_eq!(iter.next().unwrap(), 120);
    assert_eq!(iter.next().unwrap(), 200);
    assert_eq!(iter.next().unwrap(), 2000);
    assert_eq!(iter.next().unwrap(), 70000);
    assert_eq!(iter.next().unwrap(), 3_000_000);
}


#[derive(Debug, Clone)]
pub struct VintArrayIterator<'a>  {
    list: & 'a VIntArray,
    pos:usize,
    len:usize
    // ptr: *const u8,
    // end: *const u8,
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

    #[inline]
    fn count(self) -> usize {
        self.list.data.len()
    }

}



#[bench]
fn encode_6_values(b: &mut test::Bencher) {
    let mut vint = VIntArray::default();
    b.iter(||{
        vint.data.clear();
        vint.encode(110);
        vint.encode(120);

        vint.encode(200);
        vint.encode(2000);
        vint.encode(70000);
        vint.encode(3_000_000);

    })
}

#[bench]
fn encode_300_values(b: &mut test::Bencher) {
    let mut vint = VIntArray::default();
    b.iter(||{
        vint.data.clear();
        for i in 1..300 {
            vint.encode(i*i*i);
        }
    })
}

#[bench]
fn encode_300_values_fixed_vint(b: &mut test::Bencher) {
    let mut vint = VIntArrayFixed::default();
    b.iter(||{
        vint.data.clear();
        for i in 1..300 {
            vint.encode(i*i*i);
        }
    })
}

#[bench]
fn decode_sum_6_values_iter(b: &mut test::Bencher) {
    let mut vint = VIntArray::default();
    vint.encode(110);
    vint.encode(120);

    vint.encode(200);
    vint.encode(2000);
    vint.encode(70000);
    vint.encode(3_000_000);
    b.iter(||{
        vint.iter().sum::<u32>()
    })

}

#[bench]
fn decode_sum_20_000_values_iter(b: &mut test::Bencher) {
    use rand::distributions::{IndependentSample, Range};
    let mut rng = rand::thread_rng();
    let between = Range::new(0, 16_000_000);

    let mut vint = VIntArray::default();
    for _ in 1..20_000 {
        vint.encode(between.ind_sample(&mut rng));
    }
    println!("VIntArray Bytes {:?}", vint.data.len() * 8);
    b.iter(||{
        let mut data:Vec<u32> = vec![];
        for el in vint.iter(){
            data.push(el);
        }
        data
    })
}

#[bench]
fn decode_sum_20_000_values_fixed_iter(b: &mut test::Bencher) {
    use rand::distributions::{IndependentSample, Range};
    let mut rng = rand::thread_rng();
    let between = Range::new(0, 16_000_000);

    let mut vint = VIntArrayFixed::default();
    for _ in 1..20_000 {
        vint.encode(between.ind_sample(&mut rng));
    }
    println!("VIntArrayFixed Bytes {:?}", vint.data.len() * 8);
    b.iter(||{
        let mut data:Vec<u32> = vec![];
        for el in vint.iter(){
            data.push(el);
        }
        data
    })
}

#[bench]
fn decode_sum_20_000_baseline(b: &mut test::Bencher) {
    use rand::distributions::{IndependentSample, Range};
    let mut rng = rand::thread_rng();
    let between = Range::new(0, 16_000_000);

    let mut data:Vec<u32> = vec![];
    for _ in 1..20_000 {
        data.push(between.ind_sample(&mut rng));
    }

    b.iter(||{
        let mut data_out:Vec<u32> = vec![];
        for el in data.iter(){
            data_out.push(*el);
        }
        data_out
    })
}


#[bench]
fn unset_high_bit_u8_bench(b: &mut test::Bencher) {
    let data:Vec<u8> = vec![150, 150, 150, 150, 150, 150];
    b.iter(||{
        unset_high_bit_u8(data[0]) + unset_high_bit_u8(data[1])+ unset_high_bit_u8(data[2])+ unset_high_bit_u8(data[3])+ unset_high_bit_u8(data[4])+ unset_high_bit_u8(data[5])
    })
}


#[bench]
fn unset_high_bit_u8_shift(b: &mut test::Bencher) {
    let data:Vec<u8> = vec![150, 150, 150, 150, 150, 150];
    b.iter(||{
        (data[0] << 1 >> 1) + (data[1] << 1 >> 1) + (data[2] << 1 >> 1) + (data[3] << 1 >> 1) + (data[4] << 1 >> 1) + (data[5] << 1 >> 1)
    })
}

#[inline]
pub fn set_bit_at(input: &mut u8, n: u8) {
    *input |= 1 << n
}

const ONLY_HIGH_BIT_SET:u8 = (1 << 7);
const ALL_BITS_BUT_HIGHEST_SET:u8 = (1 << 7) - 1;

#[inline]
pub fn set_high_bit_u8(input: u8) -> u8{
    input | ONLY_HIGH_BIT_SET
}

#[inline]
pub fn unset_high_bit(input: &mut u8) {
    *input &= ALL_BITS_BUT_HIGHEST_SET
}

#[inline]
pub fn unset_high_bit_u8(input: u8) -> u8 {
    input << 1 >> 1
}

#[inline]
pub fn is_high_bit_set(input: u8) -> bool {
    input & ONLY_HIGH_BIT_SET != 0
}

