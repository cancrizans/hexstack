use std::collections::hash_map::Entry;
use std::collections::HashMap;

use hexstack::{PieceType, Player, Ply, Score, State};
use hexstack::board::ZobristHash;

#[allow(dead_code)]
fn search_shortest(state : State, depth : usize) -> Option<Vec<Ply>>{
    if depth == 0{
        return if state.is_won().is_some() {Some(vec![])} else {None}
    }

    let plies = state.valid_moves();

    for ply in plies{
        let mut copy = state.clone();
        copy.apply_move(ply);

        match search_shortest(copy, depth-1){
            Some(mut winning_match) => {
                winning_match.push(ply);
                return Some(winning_match)
            },
            None => {}
        }
    };

    None
}
#[allow(dead_code)]
fn search_max(state : State, depth : usize) -> (Score,Vec<Ply>){
    if depth == 0{
        return (state.eval_heuristic(),vec![]);
    }

    let plies = state.valid_moves();

    plies.into_iter().map(|ply|{
        let mut copy = state.clone();
        copy.apply_move(ply);
        let (score, mut game) = search_max(copy, depth-1);
        game.push(ply);
        (score,game)

    })
    .max_by(|(a,_),(b,_)| a.cmp(b))
    .unwrap()
}
#[allow(dead_code)]
fn search_min(state : State, depth : usize) -> (Score,Vec<Ply>){
    if depth == 0{
        return (state.eval_heuristic(),vec![]);
    }

    let plies = state.valid_moves();

    plies.into_iter().flat_map(|ply|{
        let mut copy = state.clone();

        if (copy.to_play() == Player::White) | (copy.clone().pull_moving_piece(copy.to_play(),ply.from_tile) == PieceType::Flat){
            copy.apply_move(ply);
            let (score, mut game) = search_min(copy, depth-1);
            game.push(ply);
            Some((score,game))
        } else {None}

    })
    .min_by(|(a,_),(b,_)| a.cmp(b))
    .unwrap_or((Score::win_now(Player::White),vec![]))
}

#[allow(dead_code)]
struct Searcher(HashMap<ZobristHash,(Score,Vec<Ply>)>);
impl Searcher{
    #[allow(dead_code)]
    fn new()->Searcher{
        Searcher(HashMap::new())
    }
    #[allow(dead_code)]
    fn search_max(&mut self, state : State, depth : usize) -> (Score,Vec<Ply>){
        match self.0.entry(state.zobrist_hash()){
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(vacancy) => {
                let value = search_max(state, depth);
                vacancy.insert(value.clone());
                value
            }
        }
    }
    #[allow(dead_code)]
    fn search_min(&mut self, state : State, depth : usize) -> (Score,Vec<Ply>){
        match self.0.entry(state.zobrist_hash()){
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(vacancy) => {
                let value = search_min(state, depth);
                vacancy.insert(value.clone());
                value
            }
        }
    }
}


fn search_shortest_path(state : State, depth : usize) -> Option<Vec<Ply>>{
    if depth == 0{
        return if state.is_won().is_some() {Some(vec![])} else {None}
    }

    let plies = state.valid_moves();
    
    

    for ply in plies{
        let mut copy = state.clone();
        copy.apply_move(ply);

        let good = match copy.to_play(){
            Player::Black => true,
            Player::White => {
                let remaining_moves =((depth-1) / 2) as i32 ;
    
                let hor = copy.max_white_flat_hor().unwrap_or(-6) as i32;

                hor >= 4 - remaining_moves
            }
        };

        if good {
            match search_shortest_path(copy, depth-1){
                Some(mut winning_match) => {
                    winning_match.push(ply);
                    return Some(winning_match)
                },
                None => {}
            }
        }
    };

    None
}


fn main(){
    for depth in 0..=16{
        println!("Searching depth {}...",depth);
        let state = State::setup();
        let mut searcher = Searcher::new();
        let (score,mut plies) = searcher.search_min(state, depth);
        println!("{}",score);
        // if let Some(mut plies) = plies{
            plies.reverse();
            println!("{}", Vec::from_iter(plies.iter().map(|i| i.to_string())).join(", "));
            // break;
        // }
    }
}