use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use internment::{LocalIntern};
use shared::SpwnSource;
use std::fs;
use spwn::parse_spwn;

fn test_all(input: &str, src: SpwnSource) {
    parse_spwn(input.to_string(), src, &[]).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let input = fs::read_to_string("test/test.spwn").unwrap();
    let src = LocalIntern::new("".to_string());

    let mut bmg = c.benchmark_group("main");
    bmg.throughput(Throughput::Bytes(input.len() as u64));
    bmg.bench_function("test_all", |b| b.iter(|| test_all(&input, SpwnSource::String(src))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
