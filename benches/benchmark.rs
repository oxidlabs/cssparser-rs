use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};

use cssparser_rs::parse_css;

fn bench_parse_css(c: &mut Criterion) {
    c.bench_function("parse_css", |b| {
        b.iter(|| black_box(parse_css()));
    });
}

criterion_group!(benches, bench_parse_css);
criterion_main!(benches);