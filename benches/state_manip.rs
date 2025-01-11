use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use hexstack::State;

fn criterion_benchmark(c: &mut Criterion) {
    let state0 = State::setup();
    c.bench_function("state clone", |b| b.iter(|| {
        black_box(state0.clone())
    }));

    
    c.bench_function("valid moves compute", |b| b.iter(|| {
        black_box(state0.valid_moves())
    }));

    
    let av_moves = state0.valid_moves();
    let ply = av_moves.first().unwrap();
    c.bench_function("move apply", |b| b.iter(|| {
        let mut state = state0.clone();
        state.apply_move(*ply);
        black_box(state)
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);