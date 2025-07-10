use criterion::{black_box, criterion_group, criterion_main, Criterion};
use routefinder::rng::SggPcg;

fn bench_rng_advance_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng_advance_small");
    
    group.bench_function("advance_1", |b| {
        b.iter(|| {
            let mut rng = SggPcg::new(12345);
            rng.advance(black_box(1));
        });
    });
    
    group.bench_function("advance_10", |b| {
        b.iter(|| {
            let mut rng = SggPcg::new(12345);
            rng.advance(black_box(10));
        });
    });
    
    group.bench_function("advance_100", |b| {
        b.iter(|| {
            let mut rng = SggPcg::new(12345);
            rng.advance(black_box(100));
        });
    });
    
    group.bench_function("advance_1000", |b| {
        b.iter(|| {
            let mut rng = SggPcg::new(12345);
            rng.advance(black_box(1000));
        });
    });
    
    group.finish();
}

fn bench_rng_advance_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng_advance_large");
    
    group.bench_function("advance_10000", |b| {
        b.iter(|| {
            let mut rng = SggPcg::new(12345);
            rng.advance(black_box(10000));
        });
    });
    
    group.bench_function("advance_100000", |b| {
        b.iter(|| {
            let mut rng = SggPcg::new(12345);
            rng.advance(black_box(100000));
        });
    });
    
    group.bench_function("advance_1000000", |b| {
        b.iter(|| {
            let mut rng = SggPcg::new(12345);
            rng.advance(black_box(1000000));
        });
    });
    
    group.finish();
}

fn bench_rng_advance_massive(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng_advance_massive");
    
    // Test with very large deltas that might be used in practice
    group.bench_function("advance_10000000", |b| {
        b.iter(|| {
            let mut rng = SggPcg::new(12345);
            rng.advance(black_box(10000000));
        });
    });
    
    group.bench_function("advance_100000000", |b| {
        b.iter(|| {
            let mut rng = SggPcg::new(12345);
            rng.advance(black_box(100000000));
        });
    });
    
    group.bench_function("advance_max_u32", |b| {
        b.iter(|| {
            let mut rng = SggPcg::new(12345);
            rng.advance(black_box(u32::MAX as u64));
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_rng_advance_small, bench_rng_advance_large, bench_rng_advance_massive);
criterion_main!(benches);