#[macro_use]
extern crate criterion;
extern crate vint;

extern crate snap;
extern crate byteorder;

use byteorder::{ByteOrder, LittleEndian};

use vint::*;

use criterion::Criterion;
use criterion::Fun;
use criterion::*;

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

    // let decode_copy_vint = Fun::new("vint", |b, i| {
    //     let mut vint = VIntArray::default();
    //     for i in 1..*i {
    //         vint.encode(((i as u64 * i as u64) % 16_000) as u32);
    //     }
    //     b.iter(|| vint.iter().collect::<Vec<u32>>())
    // });

    // let decode_copy_vint_fixed = Fun::new("vint_fixed", |b, i| {
    //     let mut vint = VIntArrayFixed::default();
    //     for i in 1..*i {
    //         vint.encode(((i as u64 * i as u64) % 16_000) as u32);
    //     }
    //     b.iter(|| vint.iter().collect::<Vec<u32>>())
    // });

    // let decode_copy_baseline = Fun::new("baseline", |b, i| {
    //     let mut data:Vec<u32> = vec![];
    //     for i in 1..*i {
    //         data.push(((i as u64 * i as u64) % 16_000) as u32);
    //     }
    //     b.iter(|| data.iter().cloned().collect::<Vec<u32>>())
    // });

    // let decode_copy_snappy = Fun::new("snappy", |b, i| {
    //     let mut data:Vec<u32> = vec![];
    //     for i in 1..*i {
    //         data.push(((i as u64 * i as u64) % 16_000) as u32);
    //     }
    //     let dat = snappy_encode(&data);
    //     b.iter(|| {
    //         let mut decoder = snap::Decoder::new();
    //         bytes_to_vec_u32(&decoder.decompress_vec(&dat).unwrap())
    //     });
    // });

    // let functions = vec!(decode_copy_vint, decode_copy_vint_fixed, decode_copy_baseline, decode_copy_snappy);
    
    // c.bench_functions("Encode Decode", functions, 500_000);

    // c.bench_function_over_inputs("decode_copy_vint", |b, &&size| {
    //     let mut vint = VIntArray::default();
    //     for i in 1..size {
    //         vint.encode(((i as u64 * i as u64) % 16_000) as u32);
    //     }
    //     b.iter(|| vint.iter().collect::<Vec<u32>>());
    // }, &[300, 100_000]);

    // c.bench_function_over_inputs("decode_copy_vint_fixed", |b, &&size| {
    //     let mut vint = VIntArrayFixed::default();
    //     for i in 1..size {
    //         vint.encode(((i as u64 * i as u64) % 16_000) as u32);
    //     }
    //     b.iter(|| vint.iter().collect::<Vec<u32>>());
    // }, &[300, 100_000]);

    // c.bench_function_over_inputs("decode_copy_baseline", |b, &&size| {
    //     let mut data:Vec<u32> = vec![];
    //     for i in 1..size {
    //         data.push(((i as u64 * i as u64) % 16_000) as u32);
    //     }
    //     b.iter(|| data.iter().cloned().collect::<Vec<u32>>());
    // }, &[300, 100_000]);

    // c.bench_function_over_inputs("decode_copy_snappy", |b, &&size| {
    //     let mut data:Vec<u32> = vec![];
    //     for i in 1..size {
    //         data.push(((i as u64 * i as u64) % 16_000) as u32);
    //     }
    //     let dat = snappy_encode(&data);
    //     b.iter(|| {
    //         let mut decoder = snap::Decoder::new();
    //         bytes_to_vec_u32(&decoder.decompress_vec(&dat).unwrap())
    //     });
    // }, &[100, 100_000, 1_000_000]);

    let parameters = vec![100, 1_000, 10_000, 50_000, 100_000, 200_000, 300_000, 400_000, 500_000];
    let benchmark = ParameterizedBenchmark::new("snappy", |b, i| {
        let mut data:Vec<u32> = vec![];
        for i in 1..*i {
            data.push(((i as u64 * i as u64) % 16_000) as u32);
        }
        let dat = snappy_encode(&data);
        b.iter(|| {
            let mut decoder = snap::Decoder::new();
            bytes_to_vec_u32(&decoder.decompress_vec(&dat).unwrap())
        });
    }, parameters)
    .with_function("baseline", |b, i| {
        let mut data:Vec<u32> = vec![];
        for i in 1..*i {
            data.push(((i as u64 * i as u64) % 16_000) as u32);
        }
        b.iter(|| data.iter().cloned().collect::<Vec<u32>>())
    })
    .with_function("vint", |b, i| {
        let mut vint = VIntArray::default();
        for i in 1..*i {
            vint.encode(((i as u64 * i as u64) % 16_000) as u32);
        }
        b.iter(|| vint.iter().collect::<Vec<u32>>())
    })
    .with_function("vint_fixed", |b, i| {
        let mut vint = VIntArrayFixed::default();
        for i in 1..*i {
            vint.encode(((i as u64 * i as u64) % 16_000) as u32);
        }
        b.iter(|| vint.iter().collect::<Vec<u32>>())
    });

    c.bench("decode encode", benchmark);


    let parameters = vec![1, 10, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1_000];
    let benchmark = ParameterizedBenchmark::new("snappy", |b, i| {
        let mut data:Vec<u32> = vec![];
        for i in 1..*i {
            data.push(((i as u64 * i as u64) % 16_000) as u32);
        }
        let dat = snappy_encode(&data);
        b.iter(|| {
            let mut decoder = snap::Decoder::new();
            bytes_to_vec_u32(&decoder.decompress_vec(&dat).unwrap())
        });
    }, parameters)
    .with_function("baseline", |b, i| {
        let mut data:Vec<u32> = vec![];
        for i in 1..*i {
            data.push(((i as u64 * i as u64) % 16_000) as u32);
        }
        b.iter(|| data.iter().cloned().collect::<Vec<u32>>())
    })
    .with_function("vint", |b, i| {
        let mut vint = VIntArray::default();
        for i in 1..*i {
            vint.encode(((i as u64 * i as u64) % 16_000) as u32);
        }
        b.iter(|| vint.iter().collect::<Vec<u32>>())
    })
    .with_function("vint_fixed", |b, i| {
        let mut vint = VIntArrayFixed::default();
        for i in 1..*i {
            vint.encode(((i as u64 * i as u64) % 16_000) as u32);
        }
        b.iter(|| vint.iter().collect::<Vec<u32>>())
    });

    c.bench("decode encode small", benchmark);


    let parameters = vec![1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
    let benchmark = ParameterizedBenchmark::new("snappy", |b, i| {
        let mut data:Vec<u32> = vec![];
        for i in 1..*i {
            data.push(((i as u64 * i as u64) % 16_000) as u32);
        }
        let dat = snappy_encode(&data);
        b.iter(|| {
            let mut decoder = snap::Decoder::new();
            bytes_to_vec_u32(&decoder.decompress_vec(&dat).unwrap())
        });
    }, parameters)
    .with_function("baseline", |b, i| {
        let mut data:Vec<u32> = vec![];
        for i in 1..*i {
            data.push(((i as u64 * i as u64) % 16_000) as u32);
        }
        b.iter(|| data.iter().cloned().collect::<Vec<u32>>())
    })
    .with_function("vint", |b, i| {
        let mut vint = VIntArray::default();
        for i in 1..*i {
            vint.encode(((i as u64 * i as u64) % 16_000) as u32);
        }
        b.iter(|| vint.iter().collect::<Vec<u32>>())
    })
    .with_function("vint_fixed", |b, i| {
        let mut vint = VIntArrayFixed::default();
        for i in 1..*i {
            vint.encode(((i as u64 * i as u64) % 16_000) as u32);
        }
        b.iter(|| vint.iter().collect::<Vec<u32>>())
    });

    c.bench("decode encode very small", benchmark);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
