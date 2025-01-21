use std::{fmt::Display, iter::repeat_n, sync::{Arc, Mutex}};

use futures::executor::block_on;

use hexstack::{tokonoma::{Captured, Player, PlayerMap, Position, Score, TranspositionalTable}, Ply};
use itertools::Itertools;

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

#[allow(dead_code)]
fn simulate(starting_state : Position) -> SimResults{
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
                futures::executor::block_on(state.clone().moves_with_score(BOT_DEPTH,false, None));

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

#[allow(dead_code)]
fn expand_node(args : (Vec<Ply>, Position)) -> Vec<(Vec<Ply>,Position)>{
    let (moves_hist,pos) = args;
    pos.valid_moves().into_iter().map(|ply|{
        let mut copy = pos.clone();
        copy.apply_move(ply);

        let mut hist = moves_hist.clone();
        hist.push(ply);

        (hist,copy)
    }).collect()
}


const OPENING_DEPTH : usize = 2;
const SEARCH_DEPTH : usize = 9;

fn print_section_report(position : Position, current_depth : usize, transp : Arc<Mutex<TranspositionalTable>>) -> String{
    let res = block_on(position.clone().moves_with_score(SEARCH_DEPTH+OPENING_DEPTH-current_depth, false, Some(transp.clone())));

    //let mean_score = Score::mean(res.iter().map(|(_,ev)|ev.score).collect());
    let top_score = res.first().map_or(Score::EVEN,
        |(_,er)|er.score
    );
    let threshold = top_score.add( - 0.5);

    res.into_iter()
    .filter(|(_,er)|er.score >= threshold)
    .map(|(ply,er)|{
        let sub_depth = current_depth+1;
        let inner = if sub_depth < OPENING_DEPTH {
            let mut copy = position.clone();
            copy.apply_move(ply);
            Some(print_section_report(copy, sub_depth, transp.clone()))
        } else {
            None
        };
        let he = position.compute_history_entry(ply,PlayerMap::twin(Captured::empty()));

        format!("{}{}\t{}\t[{}]{}",
            repeat_n('\t',current_depth).join(""),
            he,
            er.score, er.nodes,
            inner.map_or("".to_string(), |i|format!("\n{}",i))
        )
    }).join("\n")

}


fn main(){
    
    // let expanded_tree : Vec<(Vec<Ply>, Position)> = expand_node((vec![], Position::setup()))
    //     .into_iter()
    //     .flat_map(expand_node)
    //     .collect();
    // let sequences_len = expanded_tree.len();

    // let mut expanded_positions = HashMap::new();

    // for (hist,pos) in expanded_tree{
    //     match expanded_positions.entry(pos){
    //         Entry::Vacant(vacancy) => {vacancy.insert(hist);},
    //         Entry::Occupied(..) => {}
    //     }
    // };

    // println!("{} distinct positions from 2 plies. ({} sequences)", expanded_positions.len(), sequences_len);

    // expanded_positions.iter().for_each(|(p,h)|{
    //     println!("{} {}", p.tabulation_hash(), h.iter().map(|v|format!("{}",v)).join(" "));
    // });


    let table = Arc::new(Mutex::new(TranspositionalTable::new()));
    let state0 = Position::setup();

    println!("{}", print_section_report(state0, 0, table))

    // let caps = HashMap::from_iter([Player::White,Player::Black].map(|p|(p,Captured::empty())));

    // for first_move in state0.valid_moves(){
    //     let mut copy = state0.clone();
    //     let hentry = copy.compute_history_entry(first_move,caps.clone());

    //     println!("1. {} ...", hentry);

    //     copy.apply_move(first_move);
        
    //     let los_evaluatos = block_on(
    //         copy.clone().moves_with_score(9, false, Some(table.clone())));

    //     for (response, eval) in los_evaluatos{
    //         let hentry2 = copy.compute_history_entry(response,caps.clone());
    //         println!("1. {} {} -- {}",hentry,hentry2,eval.score);
    //     }

        


    // }

}