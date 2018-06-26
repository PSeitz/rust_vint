/*!
Crate `vint` is a library for compressing u32 integer arrays, composed mainly by small values.
by using only the necessary number of bytes required to encode the number.
This is done in `VIntArray` for example by using one bit in every byte, to signal if more bytes are needed to encode a number.

So for number 0-127 only one byte is required.   e.g. 8  -> `0b00001000`

Above we would set the high bit as signal bit to read the next byte. e.g 136 -> `0b10001000` `0b00000001`

*/


#![feature(test)]

#[cfg(test)]
extern crate rand;

#[cfg(test)]
extern crate bincode;
#[cfg(test)]
extern crate byteorder;
#[cfg(test)]
extern crate bitpacking;

#[macro_use] extern crate itertools;

pub mod vint;
pub mod vint_encode_most_common;
pub mod average_delta_encoded;
pub mod bitpacked_ra;
pub mod bitpacked_ra_step;

mod util;

#[cfg(test)]
mod tests {
    use vint::*;
    use vint_encode_most_common::*;

    fn get_test_array() -> Vec<u32> {
        vec![64, 128, 110, 120, 200, 2000, 70000, 3_000_000, 10_000_000]
    }

    #[test]
    fn test_encode_decode_vint() {
        let mut vint = VIntArray::default();
        for el in get_test_array().iter() {
            vint.encode(*el);
        }
        let decoded_data: Vec<u32> = vint.iter().collect();
        assert_eq!(get_test_array(), decoded_data);
    }

    #[test]
    fn test_encode_decode_vint_encoded() {
        let mut vint = VIntArrayEncodeMostCommon::default();
        vint.encode_vals(&get_test_array());
        let decoded_data: Vec<u32> = vint.iter().collect();
        assert_eq!(get_test_array(), decoded_data);
    }

    #[test]
    fn test_size() {
        let mut vint = VIntArray::default();

        // let dat:Vec<u32> = vec![10, 23, 788, 1, 1, 300, 1,  1, 1, 1, 1,];
        let dat:Vec<u32> = (0..128).map(|x| (x * 13)  + 4_000_307).collect();
        vint.encode_vals(&dat);

        use bincode::serialize;
        use std::fs::File;
        use std::io::prelude::*;

        let encoded: Vec<u8> = serialize(&dat).unwrap();
        File::create("check_size_bincode")
            .unwrap()
            .write_all(&encoded)
            .unwrap();

        File::create("check_size_vint")
            .unwrap()
            .write_all(&vint.serialize())
            .unwrap();

        let mut vint_common = VIntArrayEncodeMostCommon::default();
        vint_common.encode_vals(&dat);
        File::create("check_size_vint_common")
            .unwrap()
            .write_all(&vint_common.serialize())
            .unwrap();


        use bitpacking::{BitPacker4x, BitPacker};
        // Detects if `SSE3` is available on the current computed
        // and uses the best available implementation accordingly.
        let bitpacker = BitPacker4x::new();

        // Computes the number of bits used for each integers in the blocks.
        // my_data is assumed to have a len of 128 for `BitPacker4x`.
        let num_bits: u8 = bitpacker.num_bits(&dat);

        // The compressed array will take exactly `num_bits * BitPacker4x::BLOCK_LEN / 8`.
        // But it is ok to have an output with a different len as long as it is larger
        // than this.
        let mut compressed:Vec<u8> = vec![0u8; num_bits as usize * BitPacker4x::BLOCK_LEN / 8];

        // Compress returns the len.
        let _compressed_len = bitpacker.compress(&dat, &mut compressed[..], num_bits);

        File::create("check_size_bitpack")
            .unwrap()
            .write_all(&compressed)
            .unwrap();

    }


}
