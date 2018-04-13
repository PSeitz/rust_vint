#[macro_use]
extern crate criterion;
extern crate vint;

extern crate snap;
extern crate byteorder;

use byteorder::{ByteOrder, LittleEndian};

use vint::*;

use criterion::Criterion;


#[inline]
pub fn vec_with_size_uninitialized<T>(size: usize) -> Vec<T> {
    let mut buffer = Vec::with_capacity(size);
    unsafe {
        buffer.set_len(size);
    }
    buffer
}
fn vec_to_bytes_u32(data: &[u32]) -> Vec<u8> {
    let mut wtr: Vec<u8> = vec_with_size_uninitialized(data.len() * std::mem::size_of::<u32>());
    LittleEndian::write_u32_into(data, &mut wtr);
    wtr
}

fn bytes_to_vec_u32(data: &[u8]) -> Vec<u32> {
    let mut out_dat: Vec<u32> = vec_with_size_uninitialized(data.len() / std::mem::size_of::<u32>());
    unsafe {
        //DANGER ZIOONNE
        let ptr = std::mem::transmute::<*const u8, *const u32>(data.as_ptr());
        ptr.copy_to_nonoverlapping(out_dat.as_mut_ptr(), data.len() / std::mem::size_of::<u32>());
    }
    out_dat
}

fn snappy_encode(data: &[u32]) -> Vec<u8> {
    let mut encoder = snap::Encoder::new();
    encoder.compress_vec(&vec_to_bytes_u32(data)).unwrap()
}



fn criterion_benchmark(c: &mut Criterion) {

    let mut vint = VIntArray::default();
    c.bench_function("encode_6_values", move |b| b.iter(||{
        vint.data.clear();
        vint.encode(110);
        vint.encode(120);

        vint.encode(200);
        vint.encode(2000);
        vint.encode(70000);
        vint.encode(3_000_000);
    }));

    c.bench_function("encode_300_values", |b| b.iter(||{
        let mut vint = VIntArray::default();
        vint.data.clear();
        for i in 1..300 {
            vint.encode(i*i*i);
        }
    }));

    c.bench_function("encode_300_values_fixed_vint", |b| b.iter(||{
        let mut vint = VIntArrayFixed::default();
        vint.data.clear();
        for i in 1..300 {
            vint.encode(i*i*i);
        }
    }));

    c.bench_function("encode_300_baseline", move |b| b.iter(||{
        let mut data = vec![];
        for i in 1..300 {
            data.push(i*i*i);
        }
    }));

    c.bench_function("encode_300_values_snappy", |b| b.iter(||{
        let mut dat:Vec<u32> = vec![];
        for i in 1..300 {
            dat.push(i*i*i);
        }
        snappy_encode(&dat)
    }));

    let mut vint = VIntArrayFixed::default();
    for i in 1..300 {
        vint.encode(i*i*i);
    }
    c.bench_function("decode_sum_300_values_iter", move |b| b.iter(||{
        vint.iter().sum::<u32>()
    }));


    let mut vint = VIntArray::default();
    for i in 1..1_000_000 {
        vint.encode(((i as u64 * i as u64) % 16_000) as u32);
    }
    println!("VIntArray Bytes {:?}", vint.data.len());
    c.bench_function("decode_copy_1_000_000_values_iter", move |b| b.iter(||{
        let mut data_out:Vec<u32> = vec![];
        for el in vint.iter(){
            data_out.push(el);
        }
        data_out
    }));

    let mut vint = VIntArrayFixed::default();
        for i in 1..1_000_000 {
        vint.encode(((i as u64 * i as u64) % 16_000) as u32);
    }
    println!("VIntArrayFixed Bytes {:?}", vint.data.len());
    c.bench_function("decode_copy_1_000_000_values_fixed_iter", move |b| b.iter(||{
        let mut data_out:Vec<u32> = vec![];
        for el in vint.iter(){
            data_out.push(el);
        }
        data_out
    }));


    let mut data:Vec<u32> = vec![];
    for i in 1..1_000_000 {
        data.push(((i as u64 * i as u64) % 16_000) as u32);
    }
    c.bench_function("decode_copy_1_000_000_baseline", move |b| b.iter(||{
        let mut data_out:Vec<u32> = vec![];
        for el in data.iter(){
            data_out.push(*el);
        }
        data_out
    }));


    let mut data:Vec<u32> = vec![];
    for i in 1..1_000_000 {
        data.push(((i as u64 * i as u64) % 16_000) as u32);
    }
    c.bench_function("decode_copy_1_000_000_super_baseline", move |b| b.iter(||{
        data.to_vec()
    }));


    let mut data:Vec<u32> = vec![];
    for i in 1..1_000_000 {
        data.push(((i as u64 * i as u64) % 16_000) as u32);
    }
    let dat = snappy_encode(&data);

    println!("Size in Bytes {:?}", dat.len());
    c.bench_function("decode_copy_1_000_000_snappy", move |b| b.iter(||{
        let mut decoder = snap::Decoder::new();
        bytes_to_vec_u32(&decoder.decompress_vec(&dat).unwrap())
    }));

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
