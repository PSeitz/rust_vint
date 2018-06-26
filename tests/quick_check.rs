#![feature(plugin)]
#![plugin(quickcheck_macros)]

extern crate vint;
extern crate quickcheck;

mod quick_tests {
    use vint::vint::*;
    use vint::vint_encode_most_common::*;

    #[quickcheck]
    fn encode_and_decoded_is_same(xs: Vec<u32>) -> bool {
        let xs: Vec<u32> = xs.iter().map(|el| el / 8).collect();
        let mut vint = VIntArray::default();
        vint.encode_vals(&xs);
        let decoded_data: Vec<u32> = vint.iter().collect();
        xs == decoded_data
    }

    #[quickcheck]
    fn encode_and_decoded_is_same_most_common(xs: Vec<u32>) -> bool {
        let xs: Vec<u32> = xs.iter().map(|el| el / 8).collect();
        let mut vint = VIntArrayEncodeMostCommon::default();
        vint.encode_vals(&xs);
        let decoded_data: Vec<u32> = vint.iter().collect();
        xs == decoded_data
    }
}

