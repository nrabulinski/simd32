use criterion::{
    criterion_group, criterion_main, Criterion
};
use base32::decode;

pub fn base32_decode(c: &mut Criterion) {
    let input = std::fs::read("/home/niko/projects/base32/testcase16mb").unwrap();
    c.bench_function("base32 decode (26M in, 16M out)", |b| {
        b.iter(|| decode(&input))
    });
    let input = std::fs::read("/home/niko/projects/base32/testcase").unwrap();
    c.bench_function("base32 decode (320K in, 200K out)", |b| {
        b.iter(|| decode(&input))
    });
}

criterion_group!(benches, base32_decode);
criterion_main!(benches);
