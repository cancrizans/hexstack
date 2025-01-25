
use std::{collections::HashMap, str::FromStr};

use macroquad::color::Color;
use lazy_static::lazy_static;
use super::{Captured, HistoryEntry, PieceMap, Player, PlayerMap, Ply, Position};

pub struct MatchState{
    state : Position,
    valid_moves : Vec<Ply>,
    is_won : Option<Player>,

    history : Vec<HistoryEntry>,

    half_openings : PlayerMap<
        Result<Option<&'static HalfOpening>,HalfOpeningDetectionError>
        >
}

impl MatchState{
    #[allow(dead_code)]
    pub fn setup()->MatchState{
        Self::setup_from(Position::setup())
    }

    pub fn setup_from(state : Position) -> MatchState{
        let valid_moves = state.valid_moves();
        let mut match_state = MatchState{
            state,
            valid_moves,
            is_won : None,
            history : vec![],
            half_openings : PlayerMap::twin(Err(HalfOpeningDetectionError::NotEnoughMoves))
        };
        match_state.refresh();
        match_state
    }

    pub fn refresh(&mut self){
        self.valid_moves = self.state.valid_moves();
        self.is_won = self.state.is_won();

        for player in [Player::White,Player::Black]{
            self.half_openings[player] = self.detect_half_opening(player);
        }
    }

    pub fn is_won(&self) -> Option<Player>{
        self.is_won
    }

    pub fn half_opening(&self, player : Player) -> Result<Option<&'static HalfOpening>, HalfOpeningDetectionError>{
        self.half_openings[player]
    }

    pub fn apply_move(&mut self, ply : Ply){
        assert!(self.is_won.is_none());

        let entry = self.state.compute_history_entry(ply, self.current_captured());
        self.history.push(entry);

        self.state.apply_move(ply);
        self.refresh();
    }


    pub fn current_captured(&self) -> PlayerMap<Captured>{
        PlayerMap::new(
            self.current_captured_color(Player::White),
            self.current_captured_color(Player::Black),
        )
    }
    fn current_captured_color(&self, color : Player) -> Captured{
        if let Some(last_entry) = self.history.last(){
            last_entry.captured_after[color].clone()
        } else {
            Captured::empty()
        }
    }

    pub fn to_play(&self) -> Player{
        self.state.to_play()
    }

    pub fn draw_position(&self, position : &Position, captures : &PlayerMap<Captured>, arrows_alpha : f32){
        if arrows_alpha > 0.001{
            position.draw_attacks(false, arrows_alpha);
        }
        
        position.draw( false, false, false);
        for (color, caps) in captures{
            caps.draw(color);
        }
        
    }

    pub fn draw_present(&self, arrows_alpha : f32){
        self.draw_position(&self.state, &self.current_captured(),  arrows_alpha);
    }

    pub fn draw_past(&self, index : usize,  arrows_alpha : f32) -> Result<(),()>{
        if let Some(entry) = self.history.get(index){
            entry.ply.from_tile.draw_highlight_fill(Color::from_hex(0x95eeee), false);
            entry.ply.to_tile.draw_highlight_fill(Color::from_hex(0xa0ffff), false);
            for (tile,_) in &entry.kills{
                tile.draw_highlight_fill(Color::from_hex(0xddbbbb), false);
            }
            
            
            self.draw_position(&entry.state_after, &entry.captured_after,  arrows_alpha);
            Ok(())
        } else {
            Err(())
        }
    }


    pub fn state_clone(&self) -> Position{
        self.state.clone()
    }

    pub fn undo_moves(&mut self, count : usize){
        (0..count).for_each(|_|
            if let Some(entry) = self.history.pop(){
                self.state = entry.state_before;
            }
        );

        self.refresh();
    }

    pub fn history(&self) -> &Vec<HistoryEntry>{
        &self.history
    }

    pub fn get_pieces(&self, color : Player) -> &PieceMap{
        self.state.get_pieces(color)
    }

    fn beginning_state(&self) -> &Position{
        if let Some(first_entry) = self.history.first(){
            &first_entry.state_before
        } else {
            &self.state
        }
    }

    fn detect_half_opening(&self, player : Player) -> Result<Option<&'static HalfOpening>, HalfOpeningDetectionError>{
        use HalfOpeningDetectionError as HODE;
        use Player as P;
        if *self.beginning_state() != Position::setup(){
            return Err(HODE::NonStandardSetup)
        }

        let ply_index = match player{P::White => 2, P::Black=>3};
        if self.history.len() <= ply_index{
            Err(HODE::NotEnoughMoves)
        } else {
            let pos = self.history[ply_index].state_after.get_pieces(player);
            let hop = match player{
                P::White => HALF_OPENING_HASHMAP.get(pos),
                P::Black => HALF_OPENING_HASHMAP_FLIPPED.get(pos),
            }.map(|v|*v);
            Ok(hop)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HalfOpeningDetectionError{
    NonStandardSetup,
    NotEnoughMoves,
    
}


pub struct HalfOpening{
    pub name : Option<&'static str>,
    _white_moves : [Ply;2],
    white_position : PieceMap,
}

impl HalfOpening{
    fn new(name : Option<&'static str>, white_first_move : Ply, white_second_move : Ply)->Self{
        let mut pos = Position::setup();
        assert!(pos.valid_moves().contains(&white_first_move));
        pos.apply_move(white_first_move);
        pos.apply_move(*pos.valid_moves().first().unwrap());
        assert!(pos.valid_moves().contains(&white_second_move));
        pos.apply_move(white_second_move);

        HalfOpening{
            name, _white_moves : [white_first_move,white_second_move], white_position : pos.get_pieces(Player::White).clone()
        }
    }

    fn named(name : &'static str, white_first_move : Ply, white_second_move : Ply) -> Self{
        Self::new(Some(name),white_first_move,white_second_move)
    }

    #[allow(dead_code)]
    fn anon(white_first_move : Ply, white_second_move : Ply)->Self{
        Self::new(None,white_first_move,white_second_move)
    }

    pub fn name(&self)->&str{
        self.name.unwrap_or("[Anonymous]")
    }
}


lazy_static!{
    static ref HALF_OPENINGS : Vec<HalfOpening> = {
        let b_b5 = Ply::from_str("d6b5").unwrap();
        let a_a3 = Ply::from_str("a5a3").unwrap();
        let a_c5 = Ply::from_str("c7c5").unwrap();
        let s_a4 = Ply::from_str("b6a4").unwrap();
        let s_c5 = Ply::from_str("b6c5").unwrap();
        let b_e4 = Ply::from_str("c6e4").unwrap();

        let b_a4 = Ply::from_str("c6a4").unwrap();

        vec![
            HalfOpening::named("Devil", b_b5, a_a3),
            HalfOpening::named("Magician", s_a4, a_c5),
            HalfOpening::named("Hermit", s_c5,  a_a3),
            HalfOpening::named("Chariot", a_c5, a_a3),
            HalfOpening::named("Emperor", s_a4, a_a3),
            HalfOpening::named("Hierophant", b_b5, a_c5),
            HalfOpening::named("Sun", b_b5,s_a4),
            HalfOpening::named("Moon",b_b5,s_c5),

            HalfOpening::named("Fool",b_b5,b_e4),
            HalfOpening::named("Hanged Man", s_a4,b_e4),

            HalfOpening::named("Judgement", s_c5,b_e4),

            HalfOpening::named("Lovers", b_b5,b_a4),

            HalfOpening::named("Empress",a_c5,b_e4),
            

            HalfOpening::named("Seal", a_c5, Ply::from_str("c5d5").unwrap()),
        ]
    };

    static ref HALF_OPENING_HASHMAP : HashMap<PieceMap, &'static HalfOpening> = {
        let mut map = HashMap::new();
        for ho in HALF_OPENINGS.iter(){
            if map.insert(ho.white_position.clone(), ho).is_some(){
                panic!("Duplicate half opening");
            };
        };
        map
    };

    static ref HALF_OPENING_HASHMAP_FLIPPED : HashMap<PieceMap, &'static HalfOpening> = {
        let mut map = HashMap::new();
        for ho in HALF_OPENINGS.iter(){
            if map.insert(ho.white_position.clone().flip(), ho).is_some(){
                panic!("Duplicate half opening");
            };
        };
        map
    };
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::hint::black_box;
    #[test]
    fn test_half_openings(){
        for ho in HALF_OPENINGS.iter(){
            black_box(ho);
        };
        for (k,v) in HALF_OPENING_HASHMAP.iter(){
            black_box((k,v));
        };
    }
}