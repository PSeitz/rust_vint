/*!
Crate `vint` is a library for compressing u32 integer arrays, composed mainly by small values. 
by using only the necessary number of bytes required to encode the number.
This is done in `VIntArray` for example by using one bit in every byte, to signal if more bytes are needed to encode a number.

So for number 0-127 only one byte is required.   e.g. 8  -> `0b00001000`

Above we would set the high bit as signal bit to read the next byte. e.g 136 -> `0b10001000` `0b00000001`

*/

#![feature(test)]
#![feature(plugin)]
#![plugin(quickcheck_macros)]
extern crate test;
extern crate rand;
extern crate fnv;

#[cfg(test)]
extern crate quickcheck;

pub mod vint;
// pub mod vint_fixed;
pub mod vint_encode_most_common;
mod util;

#[cfg(test)]
mod quick_tests {
    use vint::*;
    // use vint_fixed::*;
    use vint_encode_most_common::*;

    // #[quickcheck]
    // fn encode_and_decoded_is_same_fixed(xs: Vec<u32>) -> bool {
    //     let xs:Vec<u32> = xs.iter().map(|el| el / 8).collect();
    //     let mut vint = VIntArrayFixed::default();
    //     for el in xs.iter() {
    //         vint.encode(*el);
    //     }
    //     let decoded_data:Vec<u32> = vint.iter().collect();
    //     xs == decoded_data
    // }

    #[quickcheck]
    fn encode_and_decoded_is_same(xs: Vec<u32>) -> bool {
        let xs:Vec<u32> = xs.iter().map(|el| el / 8).collect();
        let mut vint = VIntArray::default();
        vint.encode_vals(&xs);
        let decoded_data:Vec<u32> = vint.iter().collect();
        xs == decoded_data
    }

    #[quickcheck]
    fn encode_and_decoded_is_same_most_common(xs: Vec<u32>) -> bool {
        let xs:Vec<u32> = xs.iter().map(|el| el / 8).collect();
        let mut vint = VIntArrayEncodeMostCommon::default();
        vint.encode_vals(&xs);
        let decoded_data:Vec<u32> = vint.iter().collect();
        xs == decoded_data
    }
}

#[cfg(test)]
mod tests {
    use vint::*;
    // use vint_fixed::*;
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
        let decoded_data:Vec<u32> = vint.iter().collect();
        assert_eq!(get_test_array(), decoded_data);
    }

    // #[test]
    // fn test_encode_decode_vint_fixed() {
    //     let mut vint = VIntArrayFixed::default();
    //     for el in get_test_array().iter() {
    //         vint.encode(*el);
    //     }
    //     let decoded_data:Vec<u32> = vint.iter().collect();
    //     assert_eq!(get_test_array(), decoded_data);
    // }

    #[test]
    fn test_encode_decode_vint_encoded() {
        let mut vint = VIntArrayEncodeMostCommon::default();
        vint.encode_vals(&get_test_array());
        let decoded_data:Vec<u32> = vint.iter().collect();
        assert_eq!(get_test_array(), decoded_data);
    }

}


