#![feature(test)]
#![feature(plugin)]
#![plugin(quickcheck_macros)]
extern crate test;
extern crate rand;
extern crate fnv;

#[cfg(test)]
extern crate quickcheck;

pub mod vint;
pub mod vint_fixed;
pub mod vint_encode_most_common;
mod util;

#[cfg(test)]
mod quick_tests {
    use vint::*;
    use vint_fixed::*;
    use vint_encode_most_common::*;

    #[quickcheck]
    fn encode_and_decoded_is_same_fixed(xs: Vec<u32>) -> bool {
        let xs:Vec<u32> = xs.iter().map(|el| el / 8).collect();
        let mut vint = VIntArrayFixed::default();
        for el in xs.iter() {
            vint.encode(*el);
        }
        let decoded_data:Vec<u32> = vint.iter().collect();
        xs == decoded_data
    }

    #[quickcheck]
    fn encode_and_decoded_is_same(xs: Vec<u32>) -> bool {
        let xs:Vec<u32> = xs.iter().map(|el| el / 8).collect();
        let mut vint = VIntArray::default();
        for el in xs.iter() {
            vint.encode(*el);
        }
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
    use vint_fixed::*;
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

    #[test]
    fn test_encode_decode_vint_fixed() {
        let mut vint = VIntArrayFixed::default();
        for el in get_test_array().iter() {
            vint.encode(*el);
        }
        let decoded_data:Vec<u32> = vint.iter().collect();
        assert_eq!(get_test_array(), decoded_data);
    }

    #[test]
    fn test_encode_decode_vint_encoded() {
        let mut vint = VIntArrayEncodeMostCommon::default();
        vint.encode_vals(&get_test_array());
        let decoded_data:Vec<u32> = vint.iter().collect();
        assert_eq!(get_test_array(), decoded_data);
    }

}


