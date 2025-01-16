use std::collections::HashMap;

use macroquad::color::Color;


use super::{Captured, HistoryEntry, PieceMap, Player, Ply, Position};

pub struct MatchState{
    state : Position,
    valid_moves : Vec<Ply>,
    is_won : Option<Player>,

    history : Vec<HistoryEntry>,
}

impl MatchState{
    #[allow(dead_code)]
    pub fn setup()->MatchState{
        Self::setup_from(Position::setup())
    }

    pub fn setup_from(state : Position) -> MatchState{
        let valid_moves = state.valid_moves();
        MatchState{
            state,
            valid_moves,
            is_won : None,
            history : vec![]
        }
    }

    pub fn refresh(&mut self){
        self.valid_moves = self.state.valid_moves();
        self.is_won = self.state.is_won();
    }

    pub fn is_won(&self) -> Option<Player>{
        self.is_won
    }

    pub fn apply_move(&mut self, ply : Ply){
        assert!(self.is_won.is_none());

        let entry = self.state.compute_history_entry(ply, self.current_captured());
        self.history.push(entry);

        self.state.apply_move(ply);
        self.refresh();
    }


    pub fn current_captured(&self) -> HashMap<Player,Captured>{
        HashMap::from([
            (Player::White, self.current_captured_color(Player::White)),
            (Player::Black, self.current_captured_color(Player::Black)),
        ])
    }
    fn current_captured_color(&self, color : Player) -> Captured{
        if let Some(last_entry) = self.history.last(){
            last_entry.captured_after.get(&color).unwrap().clone()
        } else {
            Captured::empty()
        }
    }

    pub fn to_play(&self) -> Player{
        self.state.to_play()
    }

    pub fn draw_position(&self, position : &Position, captures : &HashMap<Player,Captured>, arrows_alpha : f32){
        if arrows_alpha > 0.001{
            position.draw_attacks(false, arrows_alpha);
        }
        
        position.draw( false, false, false);
        for (&color, caps) in captures{
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
}