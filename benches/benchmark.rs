use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use cssparser_rs::parse_css;

fn bench_parse_css(c: &mut Criterion) {
    let input = include_str!("../bootstrap-4.css"); // Replace with actual CSS input
    let input_length = input.len() as u64;

    let mut group = c.benchmark_group("parse_css_group");
    group.throughput(Throughput::Bytes(input_length));
    group.bench_function("parse_css", |b| {
        b.iter(|| black_box(parse_css(input)));
    });
    group.finish();
}

criterion_group!(benches, bench_parse_css);
criterion_main!(benches);