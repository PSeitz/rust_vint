#[macro_use]
extern crate criterion;
extern crate vint;

extern crate byteorder;
extern crate snap;

use byteorder::{ByteOrder, LittleEndian};

use vint::vint_encode_most_common::*;
use vint::vint::*;
// use vint::vint_fixed::*;

use criterion::Criterion;
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
    let mut out_dat: Vec<u32> =
        vec_with_size_uninitialized(data.len() / std::mem::size_of::<u32>());
    unsafe {
        //DANGER ZIOONNE
        let ptr = std::mem::transmute::<*const u8, *const u32>(data.as_ptr());
        ptr.copy_to_nonoverlapping(
            out_dat.as_mut_ptr(),
            data.len() / std::mem::size_of::<u32>(),
        );
    }
    out_dat
}

fn snappy_encode(data: &[u32]) -> Vec<u8> {
    let mut encoder = snap::Encoder::new();
    encoder.compress_vec(&vec_to_bytes_u32(data)).unwrap()
}

fn pseudo_rand(i: u32) -> u32 {
    if i % 2 == 0 {
        ((i as u64 * i as u64) % 16_000) as u32
    } else {
        i % 3
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let parameters = vec![1, 2, 25, 250, 2_500, 25_000, 250_000, 2_500_000];
    let benchmark = ParameterizedBenchmark::new(
        "baseline", |b, i| {
            let mut data: Vec<u32> = vec![];
            for i in 1..*i {
                data.push(pseudo_rand(i));
            }
            b.iter(|| data.iter().cloned().collect::<Vec<u32>>())
        }
        , parameters)
        .with_function("vint", |b, i| {
            let mut vint = VIntArray::default();
            for i in 1..*i {
                vint.encode(pseudo_rand(i));
            }
            b.iter(|| vint.iter().collect::<Vec<u32>>())
        })
        .with_function("snappy",
        |b, i| {
            let mut data: Vec<u32> = vec![];
            for i in 1..*i {
                data.push(pseudo_rand(i));
            }
            let dat = snappy_encode(&data);
            b.iter(|| {
                let mut decoder = snap::Decoder::new();
                bytes_to_vec_u32(&decoder.decompress_vec(&dat).unwrap())
            });
        })
        .with_function("vint most common", |b, i| {
            let mut vint = VIntArrayEncodeMostCommon::default();
            let values: Vec<u32> = (1..*i).map(|i| pseudo_rand(i)).collect();
            vint.encode_vals(&values);
            b.iter(|| vint.iter().collect::<Vec<u32>>())
        })
        .plot_config(plot_config)
        .throughput(|s| Throughput::Bytes(s * 4 as u32));
    c.bench("decode throughput, max_val 16_000", benchmark);

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let parameters = vec![1, 2, 25, 250, 2_500, 25_000, 250_000];
    let benchmark = ParameterizedBenchmark::new("vint", |b, i| {
        b.iter(|| {
            let mut vint = VIntArray::default();
            for i in 1..*i {
                vint.encode(pseudo_rand(i));
            }
            vint
        })
    }, parameters)
    .with_function("baseline", |b, i| {
        b.iter(|| {
            let mut data:Vec<u32> = vec![];
            for i in 1..*i {
                data.push(pseudo_rand(i));
            }
            data
        })
    })
    .with_function("snappy", |b, i| {
        b.iter(|| {
            let mut data:Vec<u32> = vec![];
            for i in 1..*i {
                data.push(pseudo_rand(i));
            }
            snappy_encode(&data)
        });
    })
    .with_function("vint most common", |b, i| {
        b.iter(|| {
            let mut vint = VIntArrayEncodeMostCommon::default();
            let values:Vec<u32> = (1..*i).map(|i| pseudo_rand(i)).collect();
            vint.encode_vals(&values);
            vint
        })
    })
    .plot_config(plot_config)
    .throughput(|s| Throughput::Bytes(s * 4 as u32));

    c.bench("encode throughput, max_val 16_000", benchmark);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
