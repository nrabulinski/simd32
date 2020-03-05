#![feature(half_open_range_patterns, exclusive_range_pattern)]
use packed_simd::{Simd, u8x8};
use rayon::prelude::*;

macro_rules! decode_func {
    ($($name:ident: $n:literal),+) => {
        $(
        #[inline]
        fn $name(input: &[u8], res: &mut [u8]) {
            assert!(input.len() >= $n);
            let n = $n / 8;
            assert!(res.len() >= n * 5);
            type S = Simd<[u8; $n]>;
            let v = S::from_slice_unaligned(input);
            let m = v.ge(S::splat(0b1000000));
            let v = m.select((v & 0x5f) - 0b1000001, v ^ 0b101000);
            unsafe {
                for index in 0..n {
                    let i = index * 5;
                    let x = index * 8;
                    res[i + 0] = (v.extract_unchecked(x + 0) << 3) | (v.extract_unchecked(x + 1) >> 2);
                    res[i + 1] = (v.extract_unchecked(x + 1) << 6) | (v.extract_unchecked(x + 2) << 1) | (v.extract_unchecked(x + 3) >> 4);
                    res[i + 2] = (v.extract_unchecked(x + 3) << 4) | (v.extract_unchecked(x + 4) >> 1);
                    res[i + 3] = (v.extract_unchecked(x + 4) << 7) | (v.extract_unchecked(x + 5) << 2) | (v.extract_unchecked(x + 6) >> 3);
                    res[i + 4] = (v.extract_unchecked(x + 6) << 5) |  v.extract_unchecked(x + 7);
                }
            }
        }
        )+
    };
}

decode_func!(
    //decode_len_64: 64,
    //decode_len_32: 32,
    decode_len_16: 16
);

#[inline]
fn decode_len_8(input: &[u8], output: &mut [u8]) {
                let v = u8x8::from_slice_unaligned(input);
                let m = v.ge(u8x8::splat(0b1000000));
                let v = m.select((v & 0x5f) - 0b1000001, v ^ 0b101000);
                unsafe {
                    output[0] = (v.extract_unchecked(0) << 3) | (v.extract_unchecked(1) >> 2);
                    output[1] = (v.extract_unchecked(1) << 6) | (v.extract_unchecked(2) << 1) | (v.extract_unchecked(3) >> 4);
                    output[2] = (v.extract_unchecked(3) << 4) | (v.extract_unchecked(4) >> 1);
                    output[3] = (v.extract_unchecked(4) << 7) | (v.extract_unchecked(5) << 2) | (v.extract_unchecked(6) >> 3);
                    output[4] = (v.extract_unchecked(6) << 5) |  v.extract_unchecked(7);
                }
}

pub fn decode(input: &[u8]) -> Vec<u8> {
    assert!(input.len() >= 8);
    assert!(input.len() % 8 == 0);
    let pad_len = {
        let v = u8x8::from_slice_unaligned(input);
        let m = v.eq(u8x8::splat(0x3d));
        m.select(u8x8::splat(1), u8x8::splat(0)).wrapping_sum()
        //(v ^ 0x3d).trailing_zeros().wrapping_sum() / 8
    } as usize;
    let len = (input.len() - pad_len) * 5 / 8;
    let mut out = vec![0u8; input.len() / 8 * 5];
    const DATA_CHUNK: usize = 1024 * 3;
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
    let mut i_i = 0;
    let mut o_i = 0;
    loop {
        let input = &input[i_i..];
        let output = &mut output[o_i..];
        match input.len() {
            //65.. => {
            //    decode_len_64(input, output);
            //    i_i += 64; o_i += 40;
            //},
            //64 => return decode_len_64(input, output),
            //33.. => {
            //    decode_len_32(input, output);
            //    i_i += 32; o_i += 20;
            //},
            //32 => return decode_len_32(input, output),
            17.. => {
                decode_len_16(input, output);
                i_i += 16; o_i += 10;
            },
            16 => return decode_len_16(input, output),
            9..=15 => {
                panic!("invalid len");
            },
            8 => return decode_len_8(input, output),
            _ => panic!("Incorrect input length!")
        }
    }
}
