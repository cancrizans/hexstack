use std::fmt::Display;

use hexstack::{Player, State};

// const OPENING_DEPTH : usize = 2;
const SAMPLES : usize = 100;
const DRAW_THRESHOLD_PLIES : usize = 100;

const BOT_DEPTH : usize = 4;

struct SimResults{
    white_victories : usize,
    black_victories : usize,
    draws : usize
}

impl Display for SimResults{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}/{}/{}",self.white_victories,self.draws,self.black_victories)
    }
}

fn simulate(starting_state : State) -> SimResults{
    let mut results = SimResults{
        white_victories : 0,
        black_victories : 0,
        draws : 0
    };

    let bar = indicatif::ProgressBar::new(SAMPLES as u64);
    (0..SAMPLES).for_each(|_sample_idx|{
        bar.inc(1);

        let mut state = starting_state.clone();

        let mut winner = None;
        for _ply_num in 0..DRAW_THRESHOLD_PLIES{
            
            // println!("{}/{}",_sample_idx,_ply_num);
            if let Some(this_winner) = state.is_won(){
                winner = Some(this_winner);
                break;
            }

            let scored_moves = 
                futures::executor::block_on(state.clone().moves_with_score(BOT_DEPTH,false));

            let (ply,_) = scored_moves.first().unwrap();
            state.apply_move(*ply);

            
        };

        
        
        match winner{
            None => results.draws += 1,
            Some(Player::White) => results.white_victories+=1,
            Some(Player::Black) => results.black_victories+=1
        };
    });
    bar.finish();

    

    results
}

fn main(){
    let state0 = State::setup();

    for first_move in state0.valid_moves(){
        let mut copy = state0.clone();
        let hentry = copy.compute_history_entry(first_move);

        copy.apply_move(first_move);

        let res = simulate(copy);

        println!("{} | {}",hentry, res);


    }

}