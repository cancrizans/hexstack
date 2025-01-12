use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use hexstack::{Piece, PieceType, Player, State, Tile};

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


    c.bench_function("stage_translate", |b|b.iter(||{
        let mut state = state0.clone();
        state.stage_translate(*ply);
        black_box(state)
    }));

    let mut translated = state0.clone();
    translated.stage_translate(*ply);

    c.bench_function("stage_attack_scan", |b|b.iter(||{
        let mut state = translated.clone();
        let attacking_player = state.to_play();
        let kills : Vec<(Tile,PieceType)> = state.stage_attack_scan(attacking_player).collect();
        black_box(kills)
    }));

    c.bench_function("attack map", |b|b.iter(||{
        black_box(state0.double_attack_map(Player::White))
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);