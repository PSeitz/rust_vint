extern crate criterion;
extern crate vint;

extern crate byteorder;
extern crate snap;
// extern crate mayda;

// use byteorder::{ByteOrder, LittleEndian};

use vint::vint::*;
use vint::vint_encode_most_common::*;
// use vint::vint_fixed::*;

use criterion::Criterion;
use criterion::*;

use std::io::BufReader;
// use std::io::BufWriter;

#[inline]
pub fn vec_with_size_uninitialized<T>(size: usize) -> Vec<T> {
    let mut buffer = Vec::with_capacity(size);
    unsafe {
        buffer.set_len(size);
    }
    buffer
}
// fn vec_to_bytes_u32(data: &[u32]) -> Vec<u8> {
//     let mut wtr: Vec<u8> = vec_with_size_uninitialized(data.len() * std::mem::size_of::<u32>());
//     LittleEndian::write_u32_into(data, &mut wtr);
//     wtr
// }

// fn bytes_to_vec_u32(data: &[u8]) -> Vec<u32> {
//     let mut out_dat: Vec<u32> = vec_with_size_uninitialized(data.len() / std::mem::size_of::<u32>());
//     unsafe {
//         //DANGER ZIOONNE
//         let ptr = std::mem::transmute::<*const u8, *const u32>(data.as_ptr());
//         ptr.copy_to_nonoverlapping(out_dat.as_mut_ptr(), data.len() / std::mem::size_of::<u32>());
//     }
//     out_dat
// }

// fn snappy_encode(data: &[u32]) -> Vec<u8> {
//     let mut encoder = snap::Encoder::new();
//     encoder.compress_vec(&vec_to_bytes_u32(data)).unwrap()
// }

fn pseudo_rand(i: u32) -> u32 {
    if i % 2 == 0 {
        ((i as u64 * i as u64) % 5_000_000) as u32
    } else {
        i % 3
    }
}
// use std::mem::transmute;
// fn pseudo_rand(i: u32) -> u32 {
//     let mut hash = 0xcbf29ce484222325;
//     let bytes: [u8; 4] = unsafe { transmute(i) };

//     for byte in bytes.iter() {
//         hash = hash ^ (*byte as u32);
//         hash = hash.wrapping_mul(0x100000001b3);
//     }
//     hash % 1_916_000
// }

/// https://developers.google.com/protocol-buffers/docs/encoding#varints
pub fn write_varu64(data: &mut [u8], mut n: u32) -> usize {
    let mut i = 0;
    while n >= 0b1000_0000 {
        data[i] = (n as u8) | 0b1000_0000;
        n >>= 7;
        i += 1;
    }
    data[i] = n as u8;
    i + 1
}

/// https://developers.google.com/protocol-buffers/docs/encoding#varints
pub fn read_varu64(data: &[u8]) -> (u32, usize) {
    let mut n: u32 = 0;
    let mut shift: u32 = 0;
    for (i, &b) in data.iter().enumerate() {
        if b < 0b1000_0000 {
            return match (b as u32).checked_shl(shift) {
                None => (0, 0),
                Some(b) => (n | b, i + 1),
            };
        }
        match ((b as u32) & 0b0111_1111).checked_shl(shift) {
            None => return (0, 0),
            Some(b) => n |= b,
        }
        shift += 7;
    }
    (0, 0)
}


fn criterion_benchmark(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    // let parameters = vec![1, 2, 25, 250, 2_500, 25_000, 250_000, 2_500_000];
    let parameters = vec![1, 2, 25, 250, 2_500, 25_000];
    let benchmark = ParameterizedBenchmark::new(
        "vint iter",
        |b, i| {
            let mut vint = VIntArray::default();
            for i in 0..*i {
                vint.encode(pseudo_rand(i));
            }
            b.iter(|| vint.iter().collect::<Vec<u32>>())
        },
        parameters,
    ).with_function("snappy varint", |b, i| {
        let sink: Vec<u8> = (0..*i).fold(vec![], |mut sink, val| {
            let mut buffer = [0,0,0,0] ;
            let bytes_written = write_varu64(&mut buffer as &mut [u8], pseudo_rand(val));
            sink.extend_from_slice(&buffer[..bytes_written]);
            sink
        });

        b.iter(|| {
            let mut vals: Vec<u32> = vec![];
            let mut pos = 0;
            loop {
                let (num, bytes_read) = read_varu64(&sink[pos..]);
                if bytes_read == 0 {
                    break;
                }else{
                    vals.push(num as u32);
                    pos += bytes_read;
                }
            }
            vals
        })
    }).with_function("vint reader", |b, i| {
        let sink = (0..*i).fold(vec![], |mut sink, val| {
            encode_varint_into(&mut sink, pseudo_rand(val));
            sink
        });

        b.iter(|| {
            let mut vals: Vec<u32> = vec![];
            let mut reader = BufReader::new(&sink[..]);
            while let Some(val) = decode_from_reader(reader.get_mut()) {
                vals.push(val);
            }
            vals
        })
    }).with_function("baseline", |b, i| {
        let mut data: Vec<u32> = vec![];
        for i in 0..*i {
            data.push(pseudo_rand(i));
        }
        b.iter(|| data.iter().cloned().collect::<Vec<u32>>())
    })
    // .with_function("mayda", |b, i| {
    //     use mayda::{Access, Encode, Monotone};
    //     let dat: Vec<u32> = (0..*i).map(|i| pseudo_rand(i)).collect();
    //     let mut bits = Monotone::new();
    //     bits.encode(&dat).unwrap();
    //     b.iter(|| bits.decode())
    // })
    // .with_function("snappy",
    // |b, i| {
    //     let mut data: Vec<u32> = vec![];
    //     for i in 0..*i {
    //         data.push(pseudo_rand(i));
    //     }
    //     let dat = snappy_encode(&data);
    //     b.iter(|| {
    //         let mut decoder = snap::Decoder::new();
    //         bytes_to_vec_u32(&decoder.decompress_vec(&dat).unwrap())
    //     });
    // })
    .with_function("vint most common", |b, i| {
        let mut vint = VIntArrayEncodeMostCommon::default();
        let values: Vec<u32> = (0..*i).map(|i| pseudo_rand(i)).collect();
        vint.encode_vals(&values);
        b.iter(|| vint.iter().collect::<Vec<u32>>())
    }).plot_config(plot_config)
    .throughput(|s| Throughput::Bytes((s * 4) as u64));
    c.bench("decode throughput, max_val 5_000_000", benchmark);

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let parameters = vec![1, 2, 5, 25, 250, 2_500, 25_000];
    let benchmark = ParameterizedBenchmark::new(
        "vint array",
        |b, i| {
            b.iter(|| {
                let mut vint = VIntArray::default();
                for i in 0..*i {
                    vint.encode(pseudo_rand(i));
                }
                vint
            })
        },
        parameters,
    ).with_function("vint into vec", |b, i| {
        b.iter(|| {
            let mut sink = vec![];
            for i in 0..*i {
                encode_varint_into(&mut sink, pseudo_rand(i))
            }
            // let values: Vec<u32> = (0..*i).map(|i| encode_num_into(&mut sink, i).unwrap());
            sink
            // let mut vint = VIntArrayEncodeMostCommon::default();
            // let values:Vec<u32> = (0..*i).map(|i| pseudo_rand(i)).collect();
            // vint.encode_vals(&values);
            // vint
        })
    }).with_function("snappy varint", |b, i| {
        b.iter(|| {

            let mut sink = vec![];
            for i in 0..*i {
                let mut buffer: [u8;4] = [0,0,0,0] ;
                let bytes_written = write_varu64(&mut buffer as &mut [u8], pseudo_rand(i));
                sink.extend_from_slice(&buffer[..bytes_written]);
            }
            sink
        })
    }).with_function("baseline", |b, i| {
        b.iter(|| {
            let mut data: Vec<u32> = vec![];
            for i in 0..*i {
                data.push(pseudo_rand(i));
            }
            data
        })
    })
    // .with_function("snappy", |b, i| {
    //     b.iter(|| {
    //         let mut data:Vec<u32> = vec![];
    //         for i in 0..*i {
    //             data.push(pseudo_rand(i));
    //         }
    //         snappy_encode(&data)
    //     });
    // })
    .with_function("vint most common", |b, i| {
        b.iter(|| {
            let mut vint = VIntArrayEncodeMostCommon::default();
            let values: Vec<u32> = (0..*i).map(|i| pseudo_rand(i)).collect();
            vint.encode_vals(&values);
            vint
        })
    }).plot_config(plot_config)
    .throughput(|s| Throughput::Bytes(*s as u64 * 4));

    c.bench("encode throughput, max_val 5_000_000", benchmark);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
