pub mod board;
pub use board::*;

pub mod matches;
pub use matches::*;

use core::f32;

use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, fmt::Display};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use macroquad::prelude::*;
use ::rand::seq::SliceRandom;

use crate::arrows;



#[derive(Copy,Clone, PartialEq, PartialOrd, Debug)]
/// Evaluation score. Can be finite or win-in-N.
/// Positive is for white, negative is for black.
pub struct Score(f32);

impl Score{
    const FINITE_THRESHOLD : f32 = 500.0;
    const WIN_BASELINE : f32 = 1000.0;

    const EVEN : Score = Score(0.0);

    /// construct a finite score from a "small" float.
    fn finite(val : f32) -> Score{
        assert!(val.abs() < Self::FINITE_THRESHOLD);
        Score(val)
    }

    /// construct a score that represents an immediate victory.
    pub fn win_now(winner : Player) -> Score{
        match winner{
            Player::Black => Score(-Self::WIN_BASELINE),
            Player::White => Score(Self::WIN_BASELINE)
        }
    }

    fn is_finite(&self) -> bool{
        self.0.abs() < Self::FINITE_THRESHOLD
    }

    fn sign_char(&self) -> char{
        if self.0 >= 0.0 {'+'} else {'-'}
    }

    fn moves(&self) -> u32{
        assert!(!self.is_finite());
        (Self::WIN_BASELINE - self.0.abs()).round() as u32
    }

    fn propagate(self) -> Score{
        if self.is_finite() {self} else {
            if self.0 > 0.0 {
                Score(self.0-1.0)
            } else {
                Score(self.0+1.0)
            }
        }
    }
}

impl Eq for Score{}

impl Ord for Score{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Display for Score{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_finite(){
            write!(f,"{:.3}",self.0)
        } else {
            write!(f,"{}∞ ({})",self.sign_char(),self.moves())
        }
    }
}


#[derive(Clone, Copy,Debug)]
/// Output of an engine node evaluation.
pub struct EvalResult{
    /// Computed score
    pub score : Score,
    /// Total number of sub-nodes examined in the computation of the score
    /// Does not include pruned branches or cache hits.
    pub nodes : usize,
}

impl EvalResult{
    fn immediate(score : Score) -> EvalResult{
        EvalResult{
            score, nodes : 1
        }
    }
}



#[derive(Clone,Debug, PartialEq, Eq)]
/// A game position.
pub struct Position{
    to_play : Player,
    pieces : [PieceMap;2],
}


impl Position{
    /// Reference to a player's pieces.
    #[inline]
    pub fn get_pieces(&self, color : Player) -> &PieceMap{
        match color{
            Player::White => &self.pieces[0],
            Player::Black => &self.pieces[1]
        }
    }
    #[inline]
    fn get_pieces_mut(&mut self, color : Player) -> &mut PieceMap{
        match color{
            Player::White => &mut self.pieces[0],
            Player::Black => &mut self.pieces[1]
        }
    }

  

    pub fn tabulation_hash(&self) -> u64{
        let mut s = DefaultHasher::new();
        self.get_pieces(Player::White).hash(&mut s);
        self.get_pieces(Player::Black).hash(&mut s);
        self.to_play.hash(&mut s);
        s.finish()
    }

    pub fn setup()->Position{
        
        let mut white_pieces = PieceMap::EMPTY;
        let mut black_pieces = PieceMap::EMPTY;

        let sbr = BOARD_RADIUS as i8;


        [
            (0,sbr, Species::Stack(Tall::Hand)),
            (1,sbr-1, Species::Stack(Tall::Star)),
            (0,sbr-1, Species::Stack(Tall::Blind)),
            (-1,sbr, Species::Stack(Tall::Blind)),

            (2,sbr-2, Species::Stack(Tall::Hand)),
            (-2,sbr, Species::Flat),

        ].into_iter().for_each(|(x,y, species)|{
            let z = -x-y;
            let t = Tile::from_xyz(x, y, z).unwrap();
            black_pieces.set(t, species);

            white_pieces.set(t.antipode(), species);

        });

        
        let pieces = [white_pieces,black_pieces];

        Position {  to_play: Player::White, pieces }
    }

    pub fn draw_attacks(&self, flip_board : bool, alpha:f32){
        for color in [Player::White,Player::Black]{
            self.get_pieces(color).clone().into_iter().for_each(|(t,pt)|{
                let p = Piece{color, species : pt};
                neighbours_attack(t,p).into_iter()
                .flatten()
                .for_each(|target|{
                    let origin : Vec2 = t.to_world(flip_board).into();
                    
                    let target_cent : Vec2 = target.to_world(flip_board).into();
                    let dir = (target_cent-origin).normalize();

                    let start = origin + dir * 0.6;
                    let end = target_cent-dir * 0.6;


                    let mut color = p.color.to_color();
                    color.a = alpha;

                    arrows::draw_arrow(
                        start,// + orth_disp, 
                        end,// + orth_disp, 
                        color, 
                        0.1, 0.2, 0.4,
                    )
                });
            })
        }
    }

    pub fn draw(&self, 
            flip_board : bool,
            draw_attacks : bool,
            draw_tile_numbers : bool,
        ){
        

        if draw_attacks {
            self.draw_attacks(flip_board,1.0)
        }

        for color in [Player::White,Player::Black]{
            self.get_pieces(color).clone().into_iter().for_each(|(t,species)|{ 
                let (x,y) = t.to_world(flip_board);
                let piece = Piece{color, species};
                piece.draw(x,y,  1.0);
            });
        }

        
        
        // self.captured.iter().for_each(|(player,capts,)|{
            
        // });
        

        if draw_tile_numbers {
            Tile::draw_tile_numbers( flip_board);
        }

        // draw_text_ex(
        //     &format!("{} to play.", 
        //     match self.to_play{
        //         Player::Black => "Black",
        //         Player::White => "White",
        //     }),
        //     -2.0*(BOARD_RADIUS as f32), -2.0*(BOARD_RADIUS as f32),
        //     TextParams{
        //         font, 
        //         font_size : 32,
        //         font_scale : 1.0/32.0,
        //         color : BLACK,
        //         ..Default::default()
        //     }
        // );

        // let (font_size, font_scale, font_scale_aspect) = camera_font_scale(0.4);

        // draw_text_ex(
        //     &format!("{:?}", self.zobrist_hash),
        //     -3.0,
        //     5.0,
        //     TextParams{font,font_scale,font_scale_aspect,font_size,
        //         color : Color::from_rgba(0x11, 0x11, 0x11, 127),
        //         ..Default::default()
        //     }
        // );
        
        // let (font_size, font_scale, font_scale_aspect) = camera_font_scale(0.4);

        // draw_text_ex(
        //     &format!("{:?}", self.max_white_flat_hor()),
        //     0.0,
        //     0.0,
        //     TextParams{font,font_scale,font_scale_aspect,font_size,
        //         color : Color::from_rgba(0x11, 0x11, 0x11, 127),
        //         ..Default::default()
        //     }
        // );
        
    }

    pub fn valid_moves(&self) -> Vec<Ply>{
        self.valid_moves_for(self.to_play)
    }

    #[inline]
    fn valid_moves_for(&self, active : Player) -> Vec<Ply>{
        let active_pieces = self.get_pieces(active);
        let opponent_pieces = self.get_pieces(active.flip());

        let opponent_free = !opponent_pieces.occupied();
        let viable_destinations_talls = opponent_free.intersection(active_pieces.viable_tall_destinations());
        let viable_destinations_flats = opponent_free.intersection(!active_pieces.occupied());



        active_pieces.clone().into_iter()
        .map(|(from_tile,species)|{
            // let piece = Piece{color : active, species};

            BitSet::move_destinations_from_tile(from_tile, active, species)
            .intersection(match species{
                Species::Flat => viable_destinations_flats,
                _ => viable_destinations_talls
            }).into_iter()
            .map(move |n|
                Ply{ from_tile, to_tile: n }
            )

        })
        .flatten().collect()
    }

    #[inline]
    pub fn pull_moving_piece(&mut self, color : Player, from_tile : Tile) -> Species{
        
        let pieces = self.get_pieces_mut(color);
        
        // let pulled = match pieces.entry(from_tile){
        //     Occupied(mut entry) => {
        //         let original = entry.get().clone();
                

        //         match original {
        //             PieceType::Flat | PieceType::Lone(..) => {
        //                 hash.toggle_piece(&from_tile,color, original);
        //                 entry.remove()
        //             },
        //             PieceType::Stack(tall) => {
        //                 hash.toggle_piece(&from_tile,color, original);
        //                 let replacement = PieceType::Flat;
        //                 hash.toggle_piece(&from_tile,color,replacement);
        //                 entry.insert(replacement);

        //                 PieceType::Lone(tall)
        //             },
        //         }
        //     },
        //     Vacant(..) => panic!() 
        // };

        // self.zobrist_hash = hash;

        let pulled = pieces.pull_moving_piece(from_tile);
        // self.zobrist_hash.toggle_piece(&from_tile, color, original);
        // if let Some(remainder) = remainder{
        //     self.zobrist_hash.toggle_piece(&from_tile, color, remainder);
        // }

        pulled
    }

    #[inline]
    pub fn stage_attack_scan(&mut self, attacking : Player) -> PieceMap{
        let double_attacked_tiles = self.double_attack_map(attacking);
        let defending = attacking.flip();
        self.get_pieces_mut(defending).kill(double_attacked_tiles)
    }

    pub fn to_play(&self)->Player{
        self.to_play
    }

    pub fn stage_translate(&mut self, ply : Ply){
        let active = self.to_play;
        let (from_tile,to_tile) = (ply.from_tile, ply.to_tile);

        
        let moving_piece = self.pull_moving_piece(active,from_tile);

        self.get_pieces_mut(active).toss(to_tile, moving_piece);

    }

    

    pub fn apply_move(&mut self, ply : Ply) -> MoveApplyReport{
        self.stage_translate(ply);

        let attacking_player = self.to_play;

        let kills : PieceMap = self.stage_attack_scan(attacking_player);
        let has_captured =  kills.is_not_empty();

        self.to_play = self.to_play.flip();

        MoveApplyReport{
            has_captured
        }
    }

    pub fn compute_history_entry(&self, ply : Ply, captured_before : HashMap<Player,Captured>) -> HistoryEntry{
        let state_before = self.clone();
        let active = state_before.to_play();


        let moves = state_before.valid_moves();

        let moved_piece = state_before.clone().pull_moving_piece(self.to_play,ply.from_tile);
        
        let mut state_simulate_kills = state_before.clone();
        state_simulate_kills.stage_translate(ply);
        let killmap = 
            state_simulate_kills.stage_attack_scan(state_simulate_kills.to_play);
            
        let kills = killmap.clone().into_iter().collect();

        let mut captured_after = captured_before;

        
        captured_after.get_mut(&active).unwrap().extend(killmap.into_iter().map(|(_,species)|species));

        let disambiguate = match moves.iter().filter(|&av|{
            (av.to_tile == ply.to_tile) & 
            (
                state_before.get_pieces(state_before.to_play).get(av.from_tile).unwrap().to_lone() == moved_piece
            )
        }).count(){
            0 => panic!("No moves matching {:?} {:?} from move pool {:?}",moved_piece,ply, moves),
            1 => false,
            _ => true
        };

        let mut state_after = state_before.clone();
        state_after.apply_move(ply);

        HistoryEntry{
            ply, state_before, state_after, moved_piece, disambiguate, kills, captured_after
        }
    }

    // pub fn attack_map(&self, attacking_player : Player) -> HashMap<Tile, u8>{
    //     let bmap = self.attack_boardmap(attacking_player);
    //     bmap.into_iter().enumerate()
    //     .flat_map(|(code,amount)|
    //         match amount {
    //             0 => None,
    //             amount => Some((Tile::from_code(code as u8), amount))
    //         }
    //     ).collect()
    // }

    pub fn clear_tile(&mut self, location : &Tile){
        // not hash safe!
        let _ = self.pieces[0].clear_tile(*location);
        let _ = self.pieces[1].clear_tile(*location);
    }

    #[inline]
    pub fn double_attack_map(&self, attacking_player : Player) -> BitSet{
        let attacking_pieces = self.get_pieces(attacking_player);
        let mut double_attacks = DoubleCounterBitset::new();
        attacking_pieces.clone().into_iter()
        .for_each(|(t,species)|
            double_attacks.add(BitSet::move_destinations_from_tile(t, attacking_player, species))
        );
        double_attacks.get_doubles()
    }

    pub async fn moves_with_score(self, depth : usize, mquad_frame_await : bool, transp : Option<Arc<Mutex<TranspositionalTable>>>) -> Vec<(Ply, EvalResult)>{
        
        if depth == 0{
            let mut depth0_moves : Vec<(Ply, EvalResult)> = self.valid_moves().into_iter()
            .map(|m| (m,EvalResult{score:Score::EVEN, nodes: 0}))
            .collect();
            
            depth0_moves.shuffle(&mut ::rand::thread_rng());

            return depth0_moves;
        }

        let heuristic = self.eval_heuristic();
        if !heuristic.is_finite(){
            return vec![]
        }

        let mut scored_moves : Vec<(Ply, EvalResult)> = vec![];
        // let mut nodes_accum = 0;

        let transp_table = if let Some(transp) = transp{
            transp
        } else {
            Arc::new(Mutex::new(TranspositionalTable::new()))
        };

        

        for m in self.valid_moves(){
            if mquad_frame_await{
                next_frame().await;
            }

            let mut copy = self.clone();
            copy.apply_move(m);
            let evaluation = copy.eval(depth-1,transp_table.clone());
            scored_moves.push((m, evaluation));
            // nodes_accum += evaluation.nodes;

            // if depth > 3{
            //     while nodes_accum >= NODES_PER_FRAME{
            //         nodes_accum -= NODES_PER_FRAME;
            //         next_frame().await
            //     }
            // }
        };
        
        let mut rng = ::rand::thread_rng();
        scored_moves.shuffle(&mut rng);

        match self.to_play{
            Player::White => scored_moves.sort_by(|(_,s1),(_,s2)| s1.score.partial_cmp(&s2.score).unwrap().reverse()),
            Player::Black => scored_moves.sort_by(|(_,s1),(_,s2)| s1.score.partial_cmp(&s2.score).unwrap()),
        }

        scored_moves
    }
    
    #[inline]
    fn eval(self, depth : usize, transp : Arc<Mutex<TranspositionalTable>>) -> EvalResult{
        self.eval_alphabeta(depth, Score::win_now(Player::Black), Score::win_now(Player::White), transp, 0)
    }

    fn is_won_home(&self) -> Option<Player>{
        for defender in [Player::White,Player::Black]{
            let attacker = defender.flip();
            if let Some(species) = self.get_pieces(attacker)
                .get(Tile::corner(defender)){
                if species == Species::Flat{
                    return Some(attacker);
                }
            }
        };
        None
    }

    pub fn is_won(&self) -> Option<Player>{
        if let Some(winner) = self.is_won_home(){
            return Some(winner)
        };

        if self.valid_moves().len() == 0{
            return Some(self.to_play.flip())
        };

        None
    }

    #[inline]
    fn passed_flat_distance(&self, player : Player) -> u8{
        const MAX_FLAT_DIST : u8 = 8;
        let opponent_house = Tile::corner(player.flip());
        let opponent_pieces = self.get_pieces(player.flip());
        let active_pieces = self.get_pieces(player);

        let mut target_diffusion = BitSet::tile_mask(&opponent_house);

        // we only count opponent pieces as obstacles, except for a piece
        // on the house tile, to prevent getting blinded by "goalkeeping".
        let walkable = (!opponent_pieces.occupied())&target_diffusion ;


        let mut flats_diffusion = active_pieces.locate_lone_flats();
        if !flats_diffusion.is_not_empty(){return MAX_FLAT_DIST;}

        for current_distance in 0..=5{
            if target_diffusion.intersection(flats_diffusion).is_not_empty(){
                return current_distance;
            }

            if current_distance % 2 == 0{
                flats_diffusion = flats_diffusion.generate_move_destinations(player, Species::Flat);
                flats_diffusion &= walkable;
            } else {
                target_diffusion = target_diffusion.generate_move_destinations(player.flip(), Species::Flat);   
                target_diffusion &= walkable;
            }
        };

        MAX_FLAT_DIST
    }
    
    #[inline]
        fn _deprecated_passed_flat_distance(&self, player : Player) -> u8{
        //Not good.
        //Should be rewritten as diffusion process on bitboard
        let opponent_house = Tile::corner(player.flip());
        let opponent_pieces = self.get_pieces(player.flip());
        let active_pieces = self.get_pieces(player);

        let mut active = HashSet::new();
        active.insert(opponent_house);

        

        for current_distance in 0..=4{
            let mut new_active = HashSet::new();
            for tile in active.drain(){
                if let Some(piece) = active_pieces.get(tile){
                    match piece {
                        Species::Flat => return current_distance,
                        Species::Stack(..) => return current_distance+1,
                        Species::Lone(..) => {}
                    };
                };

                neighbours_move(tile, Piece{color : player.flip(), species:Species::Flat})
                .iter().for_each(|n|{
                    if let Some(n) = n{
                        match opponent_pieces.get(*n){
                            None => {new_active.insert(*n);},
                            _ => {}
                        }
                    }
                });
            }

            active = new_active;
        };

        8
        
    }

    #[inline]
    fn passed_flat_score(&self, player : Player) -> f32{
        let distance = self.passed_flat_distance(player);
        if self.to_play == player {
            match distance{
                0 => 200.0,
                1 => 200.0,
                2 => 30.0,
                3 => 10.0,
                4 => 5.0,
                5 => 1.0,
                _ => 0.0
            }
        }
        else {
            match distance{
                0 => 200.0,
                1 => 100.0,
                2 => 25.0,
                3 => 7.0,
                4 => 3.0,
                5 => 0.5,
                _ => 0.0
            }
        }
    }

    /// number of available moves
    fn mobility(&self, color : Player) -> u32{
        self.get_pieces(color).clone().into_iter()
        .map(|(t,pt)|
            BitSet::move_destinations_from_tile(t, color, pt)
            .count()
        ).sum()
    }

    pub fn eval_heuristic(&self) -> Score{
        if let Some(winner)  = self.is_won_home(){
            return Score::win_now(winner);
        }
        
        let white_moves_count = self.mobility(Player::White);
        if white_moves_count == 0{
            return Score::win_now(Player::Black)
        }
        let black_moves_count = self.mobility(Player::Black);
        if black_moves_count == 0{
            return Score::win_now(Player::White)
        }

        let mobility_score = 0.1 * (
            white_moves_count as f32
            - black_moves_count as f32
        );

        


        let finite_score = [Player::White,Player::Black].into_iter()
        .map(|color|{
            let double_attacked = self.double_attack_map(color.flip());

            let multiplier = match color{
                Player::Black => -1f32,
                Player::White => 1f32
            };

            let masked_pieces = self.get_pieces(color).mask(!double_attacked);

            Species::ALL.into_iter().map(|species|{
                let instances = masked_pieces.locate_species(species);
                if !instances.is_not_empty(){
                    return 0.0
                }

                let signed_piece_value = multiplier * species.value() * (instances.count() as f32);

                signed_piece_value 
                // + instances.into_iter().map(|t|{
                //     // let signed_piece_value = multiplier * species.value();
        
                //     let horizontal_pos = t.x()+2*t.y();
                //     assert!((-6..=6).contains(&horizontal_pos));
        
                //     let prox_x = (horizontal_pos as f32 * multiplier + 6.0) / 12.0;
        
                //     let prox_score = prox_x.powf(2.0);
        
                //     let location_score = 0.1 * prox_score * multiplier * species.positional_weight();
        
        
                //     location_score
                // }).sum::<f32>()

            }).sum::<f32>() as f32

            // self.get_pieces(color).clone().into_iter().map(|(t,species)|{
                
            //     if double_attacked.get(&t){
            //         return 0.0
            //     }

            //     let signed_piece_value = multiplier * species.value();
    
            //     let horizontal_pos = t.x()+2*t.y();
            //     assert!((-6..=6).contains(&horizontal_pos));
    
            //     let prox_x = (horizontal_pos as f32 * multiplier + 6.0) / 12.0;
    
            //     let prox_score = prox_x.powf(2.0);
    
            //     let location_score = 0.1 * prox_score * multiplier * species.positional_weight();
    
    
            //     signed_piece_value + location_score
            // }
            // ).sum::<f32>() as f32
        }).sum::<f32>() as f32;
        
        
        


        let passed_flat_score = self.passed_flat_score(Player::White) 
            - self.passed_flat_score(Player::Black);

        const TEMPO_BONUS : f32 = 0.5;
        let tempo_bonus_score = match self.to_play {
            Player::White => TEMPO_BONUS,
            Player::Black => - TEMPO_BONUS
        };

        Score::finite(finite_score + mobility_score + passed_flat_score + tempo_bonus_score)
    
    }

    const MAX_QSEARCH_DEPTH : usize = 2;

    fn eval_alphabeta(self, 
        depth : usize, 
        alpha : Score, beta : Score, transp : Arc<Mutex<TranspositionalTable>>,
        qsearch_depth : usize
    
    ) -> EvalResult{
        // const NODES_PER_FRAME : usize = 500;
        
        if let Some(score) = transp.lock().unwrap().query(self.tabulation_hash(), depth){
            return EvalResult{score, nodes : 1}
        }

        let heuristic = self.eval_heuristic();
        if !heuristic.is_finite(){
            return EvalResult::immediate(heuristic);
        }

        let mut alpha = alpha;
        let mut beta = beta;
        match depth{
            0 => EvalResult::immediate(heuristic),
            _ => {
                let unsorted_moves = self.valid_moves();

                if unsorted_moves.len() == 0 {
                    EvalResult::immediate(Score::win_now(self.to_play.flip()))
                } else {
                    let sorted_moves = match depth{
                        1 => unsorted_moves,
                        _ => {
                            let mut moves_heuristic : Vec<(Ply, Score)> = vec![];
                            for ply in self.valid_moves(){
                                let mut hc = self.clone();
                                hc.apply_move(ply);
                                moves_heuristic.push((ply,hc.eval_alphabeta(depth-2,alpha,beta,transp.clone(),qsearch_depth).score))
                            };
                            match self.to_play{
                                Player::White => moves_heuristic.sort_by(|(_,s1),(_,s2)| s1.partial_cmp(&s2).unwrap().reverse()),
                                Player::Black => moves_heuristic.sort_by(|(_,s1),(_,s2)| s1.partial_cmp(&s2).unwrap()),
                            };
                            moves_heuristic.into_iter().map(|(ply,_)|ply).collect()
                        }
                    };
                    let mut value = Score::win_now(self.to_play.flip());
                    let mut nodes_count = 1;
                    for m in sorted_moves {
                        let mut copy = self.clone();
                        let application_report = copy.apply_move(m);

                        let nonquiescent = (qsearch_depth < Self::MAX_QSEARCH_DEPTH) 
                            & application_report.has_captured;
                        
                        let sub_depth = if nonquiescent{
                            depth
                        } else {
                            depth-1
                        };

                        let sub_qsearch_depth = if nonquiescent {qsearch_depth+1} else {qsearch_depth};
                        
                        let sub_tabhash = copy.tabulation_hash();
                        let sub_result = copy.eval_alphabeta(sub_depth, alpha, beta, transp.clone(), sub_qsearch_depth);
                        transp.lock().unwrap().insert(sub_tabhash, sub_depth, sub_result.score);

                        let sub_score = sub_result.score.propagate();
                        nodes_count += sub_result.nodes;

                        value = match self.to_play{
                            Player::White => value.max(sub_score),
                            Player::Black => value.min(sub_score),
                        };

                        match self.to_play{
                            Player::White => {
                                if value >= beta {break;}
                                alpha = alpha.max(value);
                            },
                            Player::Black => {
                                if value <= alpha {break;}
                                beta = beta.min(value);
                            },
                        }

                        
                    };
                    
                    EvalResult{
                        score : value,
                        nodes : nodes_count
                    }
                }
            }
        }
        
    }


    pub fn max_white_flat_hor(&self) -> Option<i8>{
        self.get_pieces(Player::White).clone().into_iter()
        .filter(|(_,species)|
            match species{
                Species::Flat | Species::Stack(..) => true,
                _ => false
            }
        )

        .map(|(t,_)|t.x() + 2*t.y())
        .max()
    }


    

    pub fn paint(&mut self, location : &Tile, brush : Option<Piece>){
        self.clear_tile(location);
        if let Some(piece) = brush{
            self.get_pieces_mut(piece.color).set(*location, piece.species);
        }

        self.recompute_hash();
    }

    pub fn flip_to_move(&mut self){
        self.to_play = self.to_play.flip();
        self.recompute_hash();
    }

    fn recompute_hash(&mut self){

    }
}

pub struct MoveApplyReport{
    has_captured : bool
}

pub struct TranspositionalTable(HashMap<u64, (usize,Score)>);

impl TranspositionalTable{
    pub fn new()->Self{
        Self(HashMap::new())
    }

    pub fn query(&self, hash : u64, min_depth : usize) -> Option<Score>{
        if let Some((depth, score)) = self.0.get(&hash){
            if *depth >= min_depth {
                Some(*score)
            } else {None}
        } else {None}
    }

    pub fn insert(&mut self, hash : u64, depth : usize, score : Score){
        match self.0.entry(hash){
            Vacant(vacancy) => {vacancy.insert((depth,score));},

            Occupied(mut entry) => {
                let occupant = entry.get();
                if depth > occupant.0 {
                    entry.insert((depth,score));
                }
            }
        }
    }
}




pub fn draw_attack_map(player : Player, attack_map : &HashMap<Tile, u8>, flip_board : bool){
    attack_map.iter().for_each(|(&t,&a)|{
        if a > 0{
            let (cx,cy) = t.to_world(flip_board);

            let x = cx + match player{
                Player::White => -0.2,
                Player::Black => 0.1
            };
            let y = cy + 0.5;

            const RECT_SZ : f32 = 0.1;
            const RECT_OUT : f32 = 0.13;
            draw_rectangle(x-RECT_OUT, y-RECT_OUT, 2.0*RECT_OUT, 2.0*RECT_OUT, BLACK);
            draw_rectangle(x-RECT_SZ, y-RECT_SZ, 2.0*RECT_SZ, 2.0*RECT_SZ, player.to_color());
            
            
        }

    });
}



#[derive(Clone)]
pub struct HistoryEntry{
    pub state_before : Position,
    pub state_after : Position,
    pub ply : Ply,
    pub moved_piece : Species,

    pub disambiguate : bool,
    pub kills : Vec<(Tile,Species)>,

    pub captured_after : HashMap<Player,Captured>,
}

impl Display for HistoryEntry{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let move_rep = if self.disambiguate{
            format!("{}{}",self.ply.from_tile,self.ply.to_tile)
        } else {
            format!("{}",self.ply.to_tile)
        };

        write!(f,"{}{}{}",
            match self.moved_piece{
                Species::Flat => "F",
                Species::Lone(tall) => match tall{
                    Tall::Blind => "B",
                    Tall::Hand => "A",
                    Tall::Star => "S"
                },
                Species::Stack(..) => "?"
            },

            move_rep,

            (0..self.kills.len()).map(|_|'*').collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_zobrist(){
        let mut counter = 0;
        let mut rng = ::rand::thread_rng();
        let mut already_seen = HashMap::new();
        const MAX_ITS : usize = 10000;
        while counter < MAX_ITS{
            let mut state = Position::setup();

            while let Some(&ply) = state.valid_moves().choose(&mut rng){
                state.apply_move(ply);

                let tabhash = state.tabulation_hash();

                match already_seen.entry(tabhash){
                    Occupied(entry) => {
                        assert_eq!(*entry.get(),state)
                    },
                    Vacant(vacancy) => {
                        vacancy.insert(state.clone());
                    }
                };
                

                counter += 1;

                if counter >= MAX_ITS{
                    break
                }
            }

        }

    }
}