 #![feature(test)]
#[macro_use]
extern crate bitfield;
extern crate test;

// use bitfield::Bit;
// use bitfield::BitRange;



pub struct VInt{
    pub id: [u8; 4],
}

impl VInt {
    pub fn get_width(&self) -> u8 {
        if is_hight_bit_set(self.id[0]){
            if is_hight_bit_set(self.id[1]){
                if is_hight_bit_set(self.id[2]){
                    4
                }else {
                    3
                }
            }else {
                2
            }
        }else{
            1
        }

    }
    pub fn get_value(&self) -> u32 {
        let mut val = unset_high_bit_u8(self.id[0]) as u32;
        if is_hight_bit_set(self.id[0]){
            val += unset_high_bit_u8(self.id[1]) as u32 >> 7;
            if is_hight_bit_set(self.id[1]){
                val += unset_high_bit_u8(self.id[2]) as u32 >> 14;
                if is_hight_bit_set(self.id[2]){
                    val += unset_high_bit_u8(self.id[3]) as u32 >> 21;
                }
            }
        }
        val
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // assert_eq!(2 + 2, 4);
    }
}



#[bench]
fn unset_high_bit_u8_bench(b: &mut test::Bencher) {
    let data:Vec<u8> = vec![150, 50, 150, 150, 150, 150];
    b.iter(||{
        unset_high_bit_u8(data[0]) + unset_high_bit_u8(data[1])+ unset_high_bit_u8(data[2])+ unset_high_bit_u8(data[3])+ unset_high_bit_u8(data[4])+ unset_high_bit_u8(data[5])
    })
}


#[bench]
fn unset_high_bit_u8_shift(b: &mut test::Bencher) {
    let data:Vec<u8> = vec![150, 50, 150, 150, 150, 150];
    b.iter(||{
        (data[0] << 1 >> 1) + (data[1] << 1 >> 1) + (data[2] << 1 >> 1) + (data[3] << 1 >> 1) + (data[4] << 1 >> 1) + (data[5] << 1 >> 1)
    })
}

// fn get_u32_from_bytes(data: &[u8], pos: usize) -> u32 {
//     let mut bytes: [u8; 4] = [0, 0, 0, 0];
//     bytes.copy_from_slice(&data[pos..pos+4]);
//     unsafe {
//         transmute(bytes)
//     }
// }


// pub fn get_bit_at(input: u32, n: u8) -> bool {
//     if n < 32 {
//         input & (1 << n) != 0
//     } else {
//         false
//     }
// }

#[inline]
pub fn set_bit_at(input: &mut u8, n: u8) {
    *input = *input | (1 << n)
}

const ONLY_HIGH_BIT_SET:u8 = (1 << 7);
const ALL_BITS_BUT_HIGHEST_SET:u8 = (1 << 7) - 1;

#[inline]
pub fn set_high_bit(input: &mut u8) {
    *input = *input | ONLY_HIGH_BIT_SET
}

#[inline]
pub fn unset_high_bit(input: &mut u8) {
    *input = *input & ALL_BITS_BUT_HIGHEST_SET
}
// #[inline]
// pub fn unset_high_bit_u8(input: u8) -> u8 {
//     input & ALL_BITS_BUT_HIGHEST_SET
// }

#[inline]
pub fn unset_high_bit_u8(input: u8) -> u8 {
    input << 1 >> 1
}

#[inline]
pub fn is_hight_bit_set(input: u8) -> bool {
    input & ONLY_HIGH_BIT_SET != 0
}

// bitfield!{
//     struct IpV4Header(MSB0 [u8]);
//     impl Debug;
//     u32;
//     get_first, _: 7, 0;
//     has_more, _: 8, 0;
//     get_ihl, _: 7, 4;
//     get_dscp, _: 13, 8;
//     get_ecn, _: 15, 14;
//     get_total_length, _: 31, 16;
//     get_identification, _: 47, 31;
//     get_df, _: 49;
//     get_mf, _: 50;
//     get_fragment_offset, _: 63, 51;
//     get_time_to_live, _: 71, 64;
//     get_protocol, _: 79, 72;
//     get_header_checksum, _: 95, 79;
//     u8, get_source_address, _: 103, 96, 4;
//     u32, into Ipv4Addr, get_destination_address, _: 159, 128;
// }

// bitfield!{
//     struct IpV4Header(MSB0 [u8]);
//     impl Debug;
//     u8;
//     get_first, _: 7, 0;
//     has_more, _: 8, 0;
//     get_ihl, _: 7, 4;
//     get_dscp, _: 13, 8;
//     get_ecn, _: 15, 14;
//     get_total_length, _: 31, 16;
//     get_identification, _: 47, 31;
//     get_df, _: 49;
//     get_mf, _: 50;
//     get_fragment_offset, _: 63, 51;
//     get_time_to_live, _: 71, 64;
//     get_protocol, _: 79, 72;
//     get_header_checksum, _: 95, 79;
//     u8, get_source_address, _: 103, 96, 4;
// }

// bitfield_fields!{
//     // The default type will be `u64
//     u64;
//     // filed1 is read-write, public, the methods are inline
//     #[inline]
//     pub field1, set_field1: 10, 0;
//     // `field2` is  read-only, private, and of type bool.
//     field2, _ : 0;
//     // `field3` will be read as an `u32` and then converted to `FooBar`.
//     // The setter is not affected, it still need an `u32` value.
//     u32, into FooBar, field3, set_field3: 10, 0;
// }