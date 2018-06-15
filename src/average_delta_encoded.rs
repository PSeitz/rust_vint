
#[derive(Debug)]
pub struct AverageEncodedDelta {
    pub avg_increase: u32,
    pub offset: i32,
    pub data: Vec<i32>,
}

#[inline]
pub fn delta_and_average_encode(vals: &[u32]) -> AverageEncodedDelta {
    // let sum:usize = vals.iter().map(|val|*val as usize).sum::<usize>();
    let avg_increase = (*vals.last().unwrap() / vals.len() as u32) as u32;
    let mut vals:Vec<i32> = vals.iter().enumerate().map(|(pos, val)| (*val as i64 - (avg_increase as usize * (pos + 1)) as i64) as i32).collect();
    let offset:i32 = vals.iter().min().cloned().unwrap();
    if offset < 0 {
        vals = vals.iter().map(|val| val - offset).collect();
    }

    AverageEncodedDelta{offset: offset as i32, data: vals, avg_increase}
}


// #[inline]
// pub fn deocde_average_encoded_delta(vals: &[i32], pos: usize, offset: i32, avg_increase: u32) -> u32 {
//     (vals[pos] as i32 + offset + (avg_increase as i32 * (pos as i32 + 1)) as i32) as u32
// }


// #[inline]
// pub fn decode_average_encoded_delta(vals: &[i32], pos: usize, offset: i32, avg_increase: u32) -> Vec[] {
//     (val + offset + (avg_increase as i32 * (pos as i32 + 1)) as i32) as u32
// }


#[inline]
pub fn decode_average_encoded_delta(val: i32, pos: usize, offset: i32, avg_increase: u32) -> u32 {
    (val + offset + (avg_increase as i32 * (pos as i32 + 1)) as i32) as u32
}

#[test]
fn test_ra_average_encoded_delta() {
    let delta_data = delta_and_average_encode(&[20, 80, 122, 520, 1100, 1234]);
    assert_eq!(decode_average_encoded_delta(delta_data.data[3], 3, delta_data.offset, delta_data.avg_increase), 520);
}
