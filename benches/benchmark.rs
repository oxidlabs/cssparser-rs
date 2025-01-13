use logos::Logos;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use cssparser_rs::{parser::Parser, Token};

fn bench_lexer_css(c: &mut Criterion) {
    let input = include_str!("../bootstrap-4.css");
    let input_length = input.len() as u64;

    let mut group = c.benchmark_group("css_lexing");
    group.throughput(Throughput::Bytes(input_length));
    
    // Benchmark your parser
    group.bench_function("cssparser-rs", |b| {
        b.iter(|| {
            let mut lexer = Token::lexer(input);
            while let Some(_token) = lexer.next() {}
        });
    });

    // Benchmark the cssparser crate
    group.bench_function("cssparser", |b| {
        b.iter(|| {
            let mut input = cssparser::ParserInput::new(input);
            let mut parser = cssparser::Parser::new(&mut input);
            while let Ok(_token) = parser.next() {}
        });
    });
    
    group.finish();
}

fn bench_parser_css(c: &mut Criterion) {
    let input = include_str!("../bootstrap-4.css");
    let input_length = input.len() as u64;

    let mut group = c.benchmark_group("css_parsing");
    group.throughput(Throughput::Bytes(input_length));
    
    // Benchmark your parser
    group.bench_function("cssparser-rs", |b| {
        b.iter(|| {
            let mut parser = Parser::new(input);
            let style_sheet = parser.parse_stylesheet().unwrap();
        });
    });
    
    group.finish();
}

/* criterion_group! {
    name = benches;
    config = Criterion::default().without_plots();
    targets = bench_lexer_css
} */
criterion_group!(benches, bench_lexer_css, bench_parser_css);
criterion_main!(benches);