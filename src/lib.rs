#![feature(half_open_range_patterns, exclusive_range_pattern)]
use packed_simd::{Simd, u8x8};
use rayon::prelude::*;

macro_rules! decode_func {
    ($($name:ident: $n:literal),+) => {
        $(
        #[inline]
        fn $name(input: &[u8], res: &mut [u8], mut buf: [u8; 64]) {
            assert!(input.len() >= $n);
            let n = $n / 8;
            assert!(res.len() >= n * 5);
            type S = Simd<[u8; $n]>;
            let v = S::from_slice_unaligned(input);
            let m = v.ge(S::splat(0b1000000));
            unsafe {
                m.select((v & 0x5f) - 0b1000001, v ^ 0b101000)
                    .write_to_slice_aligned_unchecked(&mut buf);
            }
            for index in 0..n {
                let i = index * 5;
                let x = index * 8;
                res[i + 0] = (buf[x + 0] << 3) | (buf[x + 1] >> 2);
                res[i + 1] = (buf[x + 1] << 6) | (buf[x + 2] << 1) | (buf[x + 3] >> 4);
                res[i + 2] = (buf[x + 3] << 4) | (buf[x + 4] >> 1);
                res[i + 3] = (buf[x + 4] << 7) | (buf[x + 5] << 2) | (buf[x + 6] >> 3);
                res[i + 4] = (buf[x + 6] << 5) |  buf[x + 7];
            }
        }
        )+
    };
}

decode_func!(
    decode_len_64: 64,
    decode_len_32: 32,
    decode_len_16: 16
);

pub fn decode(input: &[u8]) -> Vec<u8> {
    assert!(input.len() >= 8);
    assert!(input.len() % 8 == 0);
    let data_len = if let Some(n) = input.iter().position(|&a| a == b'=') { n } else { input.len() };
    let len = data_len * 5 / 8;
    let mut out = vec![0u8; input.len() / 8 * 5];
    const DATA_CHUNK: usize = 1024;
    const OUT_CHUNK: usize = DATA_CHUNK / 8 * 5;
    if input.len() >= DATA_CHUNK {
        input.par_chunks(DATA_CHUNK).zip(out.par_chunks_mut(OUT_CHUNK)).for_each(|(input, output)| {
            decode_internal(input, output);
        });
    } else {
        decode_internal(&input, &mut out);
    }
    unsafe { out.set_len(len); }
    out
}

#[inline]
fn decode_internal(input: &[u8], output: &mut [u8]) {
    let mut buf = [0u8; 64];
    let mut i_i = 0;
    let mut o_i = 0;
    loop {
        let input = &input[i_i..];
        let output = &mut output[o_i..];
        match input.len() {
            65.. => {
                decode_len_64(input, output, buf);
                i_i += 64; o_i += 40;
            },
            64 => return decode_len_64(input, output, buf),
            33..=63 => {
                decode_len_32(input, output, buf);
                i_i += 32; o_i += 20;
            },
            32 => return decode_len_32(input, output, buf),
            17..=31 => {
                decode_len_16(input, output, buf);
                i_i += 16; o_i += 10;
            },
            16 => return decode_len_16(input, output, buf),
            9..=15 => {
                panic!("Not valid base32");
            }
            8 => {
                let v = u8x8::from_slice_unaligned(input);
                let m = v.ge(u8x8::splat(0b1000000));
                unsafe {
                    m.select((v & 0x5f) - 0b1000001, v ^ 0b101000)
                        .write_to_slice_aligned_unchecked(&mut buf);
                }
                output[0] = (buf[0] << 3) | (buf[1] >> 2);
                output[1] = (buf[1] << 6) | (buf[2] << 1) | (buf[3] >> 4);
                output[2] = (buf[3] << 4) | (buf[4] >> 1);
                output[3] = (buf[4] << 7) | (buf[5] << 2) | (buf[6] >> 3);
                output[4] = (buf[6] << 5) |  buf[7];
                return;
            },
            _ => panic!("Incorrect input length!")
        }
    }
}