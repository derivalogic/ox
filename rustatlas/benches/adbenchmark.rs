use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustatlas::prelude::*;

fn ad_benchmark(c: &mut Criterion) {
    c.bench_function("add operations in tape", |b| {
        b.iter(|| {
            Tape::start_recording();
            let a: ADNumber = ADNumber::new(1.0);
            let b: ADNumber = ADNumber::new(2.0);
            let mut c: ADNumber = (a * b).into();
            for _ in 0..100000 {
                c = (c * b).into();
            }
            black_box(c);
        })
    });
}
criterion_group!(benches, ad_benchmark);
criterion_main!(benches);
