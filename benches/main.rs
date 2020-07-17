// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
    c.bench_function("fib 25", |b| b.iter(|| fibonacci(black_box(25))));
    c.bench_function("fib 30", |b| b.iter(|| fibonacci(black_box(30))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
