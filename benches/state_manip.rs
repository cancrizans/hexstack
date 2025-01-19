use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use futures::executor::block_on;
use hexstack::{Piece, Species, Player, Position, Tile};

fn criterion_benchmark(c: &mut Criterion) {
    let state0 = Position::setup();
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


    c.bench_function("stage translate", |b|b.iter(||{
        let mut state = state0.clone();
        state.stage_translate(*ply);
        black_box(state)
    }));

    let mut translated = state0.clone();
    translated.stage_translate(*ply);

    c.bench_function("stage attack scan", |b|b.iter(||{
        let mut state = translated.clone();
        let attacking_player = state.to_play();
        let kills : Vec<(Tile,Species)> = state.stage_attack_scan(attacking_player).into_iter().collect();
        black_box(kills)
    }));

    c.bench_function("attack map", |b|b.iter(||{
        black_box(state0.double_attack_map(Player::White))
    }));

    // c.bench_function("attack kill", |b|b.iter(||{
    //     black_box(state0.double_attack_map(Player::White))
    // }));

    c.bench_function("eval", |b|b.iter(||{
        black_box(block_on(state0.clone().moves_with_score(6, false,None)))
    }));

    c.bench_function("eval heuristic", |b|b.iter(||{
        black_box(state0.clone().eval_heuristic())
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);