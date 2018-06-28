use average_delta_encoded::decode_average_encoded_delta;
use average_delta_encoded;
use std::ptr::copy_nonoverlapping;
use std::mem::transmute;

#[derive(Debug)]
pub struct EncodingInfo {
    avg_increase: u32,
    offset: i32,
    encoded: Vec<u8>,
    bytes_per_element: BytesRequired,
}

#[derive(Debug, Clone, Copy)]
pub enum BytesRequired {
    One = 1,
    Two,
    Three,
    Four,
}

fn get_bytes_required(val: u32) -> BytesRequired {
    if val < 1 << 8 {
        BytesRequired::One
    } else if val < 1 << 16 {
        BytesRequired::Two
    } else if val < 1 << 24 {
        BytesRequired::Three
    } else {
        BytesRequired::Four
    }
}

#[inline]
pub fn encode_vals_bitpacked_average_endcoded(vals: &[u32]) -> EncodingInfo {

    let delta_info = average_delta_encoded::delta_and_average_encode(vals);
    let max_val = *delta_info.data.iter().max().unwrap() as u32;
    let bytes_required = get_bytes_required(max_val);

    let mut encoded:Vec<u8> = vec![];
    let total_num_bytes = bytes_required as usize * delta_info.data.len();
    encoded.resize(1 + total_num_bytes, 0);
    for (i, val) in delta_info.data.iter().enumerate() {
        let first_block = i * bytes_required as usize;

        let data: [u8; 4] = unsafe { transmute(*val) };
        // encoded[first_block .. first_block + bytes_required as usize].copy_from_slice(&data[.. bytes_required as usize]);

        unsafe {
            copy_nonoverlapping(data.as_ptr(), encoded[first_block ..].as_mut_ptr(), bytes_required as usize);
        }
    }

    // println!("encoded {:?}", encoded);

    EncodingInfo{avg_increase: delta_info.avg_increase, offset: delta_info.offset, encoded, bytes_per_element:bytes_required}
}


#[inline]
pub fn encode_vals(vals: &[u32], bytes_required:BytesRequired) -> (BytesRequired, Vec<u8>) {

    // let max_val = *vals.iter().max().unwrap() as u32;
    // let bytes_required = get_bytes_required(max_val);

    let mut encoded:Vec<u8> = vec![];
    let total_num_bytes = bytes_required as usize * vals.len();
    encoded.resize(1 + total_num_bytes, 0);
    for (i, val) in vals.iter().enumerate() {
        let first_block = i * bytes_required as usize;
        let data: [u8; 4] = unsafe { transmute(*val) };
        unsafe {
            copy_nonoverlapping(data.as_ptr(), encoded[first_block ..].as_mut_ptr(), bytes_required as usize);
        }
    }

    (bytes_required, encoded)
}

// pub fn decode_bit_packed_val(val: &[u8], num_bits: u8, index: usize) -> u32 {
pub fn decode_bit_packed_val(info: &EncodingInfo, index: usize) -> u32 {
    // let num_blocks = num_bits / 8;
    // println!("num_blocks {:?}", num_blocks + 1);

    // let bit_pos_start = index * num_bits as usize;
    // let bit_pos_end = index * num_bits as usize + num_bits as usize;
    // let bytes_slice = &info.encoded[bit_pos_start .. bit_pos_end];

    let bit_pos_start = index * info.bytes_per_element as usize;
    let bytes_slice = &info.encoded[bit_pos_start ..];

    let mut out: u32 = 0;
    unsafe {
        copy_nonoverlapping(bytes_slice.as_ptr(), &mut out as *mut u32 as *mut u8, info.bytes_per_element as usize);
    }

    // decode_average_encoded_delta()
    decode_average_encoded_delta(out as i32, index, info.offset, info.avg_increase)

}

// #[test]
// fn test_ra_encode_bitpacked_monotone() {
//     let info:EncodingInfo = encode_vals_bitpacked_average_endcoded(&[150, 170, 175]);

//     // let bytes = vec_to_bytes_u32(&info.encoded);
//     // let bytes = info.encoded;
//     // decode_bit_packed_val(&bytes, info.bytes_per_element as u8, 3);
//     // assert_eq!(decode_bit_packed_val(&bytes, info.bytes_per_element as u8, 0), 150);
//     // assert_eq!(decode_bit_packed_val(&info, 0), 150);
//     assert_eq!(decode_bit_packed_val(&info, 2), 175);
// }





