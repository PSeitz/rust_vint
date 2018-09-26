use average_delta_encoded;

#[derive(Debug)]
pub struct EncodingInfo {
    avg_increase: u32,
    offset: i32,
    encoded: Vec<u32>,
    num_bits: u8,
}

fn get_required_num_bits(v: u32) -> u8 {
    if v == 0 {
        0
    } else {
        32u8 - (v.leading_zeros() as u8)
    }
}

#[inline]
pub fn encode_vals_bitpacked_average_endcoded(vals: &[u32]) -> EncodingInfo {
    let delta = average_delta_encoded::delta_and_average_encode(vals);
    println!("{:?}", delta);
    let max_val = *delta.data.iter().max().unwrap() as u32;
    let num_bits = get_required_num_bits(max_val);
    if num_bits == 0 {}
    println!("max_val {:?}", max_val);
    println!("num_bits {:?}", num_bits);
    let mut encoded: Vec<u32> = vec![];
    let total_num_bits = num_bits as usize * delta.data.len();
    encoded.resize(1 + total_num_bits / 32, 0);
    for (i, val) in delta.data.iter().enumerate() {
        let bit_pos = i * num_bits as usize;
        println!("bit_pos {:?}", bit_pos);
        if bit_pos / 32 != (bit_pos + num_bits as usize) / 32 {
            // hits number border

        } else {
            let closest_block = bit_pos - bit_pos % 32;
            // println!("closest_block {:?}", closest_block);
            let shifted_val = (*val as u32) << (bit_pos % 32);
            // let block = bit_pos / 32;
            encoded[closest_block as usize] |= shifted_val;
        }
    }

    // println!("encoded {:?}", encoded);

    EncodingInfo {
        avg_increase: delta.avg_increase,
        offset: delta.offset,
        encoded,
        num_bits,
    }
}

use std;
use std::mem::transmute;

#[inline]
pub fn vec_with_size_uninitialized<T>(size: usize) -> Vec<T> {
    let mut buffer = Vec::with_capacity(size);
    unsafe {
        buffer.set_len(size);
    }
    buffer
}

pub fn vec_to_bytes_u32(data: &[u32]) -> Vec<u8> {
    // let mut wtr: Vec<u8> = vec_with_size_uninitialized(data.len() * std::mem::size_of::<u32>());
    // for el in data {
    // }
    // let data: [u8; 4] = unsafe { transmute(val[block_with_bits]) };
    // LittleEndian::write_u32_into(data, &mut wtr);
    // wtr

    let mut out: Vec<u8> = Vec::with_capacity(data.len() * std::mem::size_of::<u32>());
    for el in data {
        let block: [u8; 4] = unsafe { transmute(*el) };
        out.extend(block.iter());
    }
    out
}

// #[inline]
// pub fn decode_bit_packed_val(val: &[u8], num_bits: u8, index: usize) -> u32 {
//     let num_blocks = (num_bits / 8);
//     println!("num_blocks {:?}", num_blocks + 1);

//     let bit_pos_start = index * num_bits as usize;
//     let bit_pos_end = index * num_bits as usize + num_bits;
//     if bit_pos / 8 != (bit_pos + num_bits as usize) / 8 { // hits number border

//     }

//     let closest_block = bit_pos - bit_pos % 32;
//     let block_with_bits = bit_pos / 32;
//     let offset_in_block = bit_pos % 32;

//     // //Copy data in u64
//     // let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
//     // let data: [u8; 4] = unsafe { transmute(val[block_with_bits]) };
//     // bytes[..=3].copy_from_slice(&data[..]);

//     // let data: [u8; 4] = unsafe { transmute(val[block_with_bits + 1]) };
//     // bytes[4..].copy_from_slice(&data[..]);

//     // val[block_with_bits]

//     // let mut block = vals[closest_block as usize];
//     // println!("block {:?}", block);
//     println!("offset_in_block {:?}", offset_in_block);
//     println!("bit_pos {:?}", bit_pos);
//     // block >>= offset_in_block;
//     // println!("block {:?}", block);

//     // println!("encoded {:?}", encoded);
//     0
// }

#[test]
fn test_ra_encode_bitpacked_monotone() {
    let info: EncodingInfo = encode_vals_bitpacked_average_endcoded(&[150, 170, 175]);

    let _bytes = vec_to_bytes_u32(&info.encoded);
    // decode_bit_packed_val(&bytes, info.num_bits, 3);
}
