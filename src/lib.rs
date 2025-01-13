#![feature(hash_extract_if)]
use core::f32;

use std::collections::{HashSet, VecDeque};
use std::usize::MAX;
use std::{collections::HashMap, fmt::Display};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use futures::future::BoxFuture;
use futures::FutureExt;
use macroquad::prelude::*;

pub mod arrows;
pub mod board;

pub use board::{Player, Ply,Tall, Tile, Piece, PieceType,neighbours_attack, neighbours_move,};

use board::{ BoardMap, Captured, ZobristHash, BOARD_RADIUS};
use ::rand::seq::SliceRandom;
pub mod engine_debug;
pub mod game;
pub mod ui;
pub mod assets;
pub mod theme;


#[derive(Copy,Clone, PartialEq, PartialOrd, Debug)]
pub struct Score(f32);

impl Score{
    const FINITE_THRESHOLD : f32 = 500.0;
    const WIN_BASELINE : f32 = 1000.0;

    const EVEN : Score = Score(0.0);

    fn finite(val : f32) -> Score{
        assert!(val.abs() < Self::FINITE_THRESHOLD);
        Score(val)
    }

    pub fn win_now(player : Player) -> Score{
        match player{
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
pub struct EvalResult{
    pub score : Score,
    pub nodes : usize,
}

impl EvalResult{
    fn immediate(score : Score) -> EvalResult{
        EvalResult{
            score, nodes : 1
        }
    }
}



#[derive(Clone)]
pub struct State{
    // board : Board,
    to_play : Player,
    // pieces : BoardMap<Piece>,

    pieces : [HashMap<Tile,PieceType>;2],

    captured : HashMap<Player,Captured>,

    zobrist_hash : ZobristHash,
}


impl State{
    fn get_pieces(&self, color : Player) -> &HashMap<Tile,PieceType>{
        match color{
            Player::White => &self.pieces[0],
            Player::Black => &self.pieces[1]
        }
    }
    fn get_pieces_mut(&mut self, color : Player) -> &mut HashMap<Tile,PieceType>{
        match color{
            Player::White => &mut self.pieces[0],
            Player::Black => &mut self.pieces[1]
        }
    }

  

    pub fn zobrist_hash(&self) -> ZobristHash{
        self.zobrist_hash
    }

    pub fn setup()->State{
        // let board = Board::build();
        
        let mut white_pieces = HashMap::new();
        let mut black_pieces = HashMap::new();

        let sbr = BOARD_RADIUS as i8;

        let mut zobrist_hash = ZobristHash::CLEAR;
        [
            (0,sbr, PieceType::Stack(Tall::Hand)),
            (1,sbr-1, PieceType::Stack(Tall::Star)),
            (0,sbr-1, PieceType::Stack(Tall::Blind)),
            (-1,sbr, PieceType::Stack(Tall::Blind)),

            (2,sbr-2, PieceType::Stack(Tall::Hand)),
            (-2,sbr, PieceType::Flat),

        ].into_iter().for_each(|(x,y, species)|{
            let z = -x-y;
            let t = Tile::from_xyz(x, y, z).unwrap();
            black_pieces.insert(t, species);
            zobrist_hash.toggle_piece(&t, Player::Black, species);
            white_pieces.insert(t.antipode(), species);
            zobrist_hash.toggle_piece(&t, Player::White, species);
        });

        
        let pieces = [white_pieces,black_pieces];

        State {  to_play: Player::White, pieces , captured:HashMap::from([(Player::White,Captured::empty()),(Player::Black,Captured::empty())]) , zobrist_hash}
    }

    pub fn draw_attacks(&self, flip_board : bool, alpha:f32){
        for color in [Player::White,Player::Black]{
            self.get_pieces(color).iter().for_each(|(t,pt)|{
                let p = Piece{color, species : *pt};
                neighbours_attack(*t,p).into_iter()
                .flatten()
                .for_each(|target|{
                    let origin : Vec2 = t.to_world(flip_board).into();
                    
                    let target_cent : Vec2 = target.to_world(flip_board).into();
                    let dir = (target_cent-origin).normalize();

                    let start = origin + dir * 0.6;
                    let end = target_cent-dir * 0.6;

                    // let orth_disp = vec2(-dir.y,dir.x) * 0.1 * match p.color{
                    //     Player::Black => -1.0,
                    //     Player::White => 1.0,
                    // };

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
            piece_tex : Texture2D, 
            font : Font,
            flip_board : bool,
            draw_attacks : bool,
            draw_tile_numbers : bool,
        ){
        // Tile::draw_board(flip_board);

        if draw_attacks {
            self.draw_attacks(flip_board,1.0)
        }

        for color in [Player::White,Player::Black]{
            self.get_pieces(color).iter().for_each(|(t,&species)|{ 
                let (x,y) = t.to_world(flip_board);
                let piece = Piece{color, species};
                piece.draw(x,y, piece_tex , 1.0);
            });
        }

        
        
        self.captured.iter().for_each(|(player,capts,)|{
            let n_capt = 0.5*(capts.count().saturating_sub(1) as f32);
            let basey = match player {Player::White => 4.7, Player::Black => -4.7};
            capts.iter().enumerate().for_each(|(i,piece_type)|{
                let p = Piece{color : player.flip(), species : piece_type};
                let x = 0.6*(i as f32 - n_capt);
                let y = basey;
                p.draw(x,y, piece_tex, 0.5);
                // (0..count).for_each(|it|{
                //     // let y = basey + 0.6*(it as f32);
                    
                // });
                
            });
        });
        

        if draw_tile_numbers {
            Tile::draw_tile_numbers(font, flip_board);
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

        active_pieces.iter()
        .map(|(t,&species)|{
            let piece = Piece{color : active, species};

            neighbours_move(*t,piece).into_iter().flatten()
            .filter(|n|!opponent_pieces.contains_key(n))
            .filter(move |n| 
                if let Some(dest_occupant) = active_pieces.get(n){
                    (piece.species != PieceType::Flat) &
                    (*dest_occupant == PieceType::Flat) 
                } else {
                    true
                }
            )
            .map(move |n|
                Ply{ from_tile: *t, to_tile: n }
            )

        })
        .flatten().collect()
    }

    #[inline]
    pub fn pull_moving_piece(&mut self, color : Player, from_tile : Tile) -> PieceType{
        let mut hash = self.zobrist_hash();
        let pieces = self.get_pieces_mut(color);
        
        let pulled = match pieces.entry(from_tile){
            Occupied(mut entry) => {
                let original = entry.get().clone();
                

                match original {
                    PieceType::Flat | PieceType::Lone(..) => {
                        hash.toggle_piece(&from_tile,color, original);
                        entry.remove()
                    },
                    PieceType::Stack(tall) => {
                        hash.toggle_piece(&from_tile,color, original);
                        let replacement = PieceType::Flat;
                        hash.toggle_piece(&from_tile,color,replacement);
                        entry.insert(replacement);

                        PieceType::Lone(tall)
                    },
                }
            },
            Vacant(..) => panic!() 
        };

        self.zobrist_hash = hash;

        pulled
    }

    #[inline]
    pub fn stage_attack_scan(&mut self, attacking_player : Player) -> impl Iterator<Item = (Tile,PieceType)> + '_{
        let defending_player = attacking_player.flip();
        let double_attacked_tiles = self.double_attack_map(attacking_player);

        let target_tiles : HashSet<Tile> = HashSet::from_iter(
            self.get_pieces(defending_player).iter()
            .map(|(t,_)|*t)
        );

        let killed_tiles : Vec<Tile> = target_tiles.intersection(&double_attacked_tiles).map(|t|*t).collect();

        killed_tiles.into_iter().map(move |t|{
                let killed_piece = self.get_pieces_mut(defending_player).remove(&t).unwrap();
                self.zobrist_hash.toggle_piece(&t, defending_player,killed_piece);
                (t,killed_piece)
            })
            
    }

    pub fn to_play(&self)->Player{
        self.to_play
    }

    pub fn stage_translate(&mut self, ply : Ply){
        let active = self.to_play;
        let (from_tile,to_tile) = (ply.from_tile, ply.to_tile);

        
        let moving_piece = self.pull_moving_piece(self.to_play,from_tile);
        let mut hash = self.zobrist_hash;

        match self.get_pieces_mut(self.to_play).entry(to_tile){
            Vacant(vacancy) => {
                hash.toggle_piece(&to_tile,active, moving_piece);
                vacancy.insert(moving_piece);
            },

            Occupied(mut entry) => {
                let occupant = entry.get();
                
                match occupant{
                    PieceType::Flat => (),
                    _ => panic!()
                };
                match moving_piece{
                    PieceType::Lone(moving_tall) => {
                        hash.toggle_piece(&to_tile,active, *occupant);
                        let tgt_replacement = PieceType::Stack(moving_tall);
                        hash.toggle_piece(&to_tile,active, tgt_replacement);
                        entry.insert(tgt_replacement);
                    },
                    _ => panic!("Non-lone moving piece moving into flat.")
                }

                
            }
        } ;
        self.zobrist_hash = hash;
    }

    pub fn apply_move(&mut self, ply : Ply){
        self.stage_translate(ply);

        let attacking_player = self.to_play;

        let kills : Vec<(Tile, PieceType)> = self.stage_attack_scan(attacking_player).collect();

        self.captured.get_mut(&attacking_player).unwrap()
        .extend(kills.into_iter()
            .flat_map(|(_,killed_piece)|{
                killed_piece.unstack()
            })
            
        );


        self.to_play = self.to_play.flip();
        self.zobrist_hash.toggle_to_move();
    }

    pub fn compute_history_entry(&self, ply : Ply) -> HistoryEntry{
        let state_before = self.clone();

        let moves = state_before.valid_moves();

        let moved_piece = state_before.clone().pull_moving_piece(self.to_play,ply.from_tile);
        
        let mut state_simulate_kills = state_before.clone();
        state_simulate_kills.stage_translate(ply);
        let kills : Vec<(Tile, PieceType)> = state_simulate_kills.stage_attack_scan(state_simulate_kills.to_play).collect();


        let disambiguate = match moves.iter().filter(|&av|{
            (av.to_tile == ply.to_tile) & 
            (
                state_before.get_pieces(state_before.to_play).get(&av.from_tile).unwrap().to_lone() == moved_piece
            )
        }).count(){
            0 => panic!("No moves matching {:?} {:?} from move pool {:?}",moved_piece,ply, moves),
            1 => false,
            _ => true
        };

        let mut state_after = state_before.clone();
        state_after.apply_move(ply);

        HistoryEntry{
            ply, state_before, state_after, moved_piece, disambiguate, kills
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
        let _ = self.pieces[0].remove(location);
        let _ = self.pieces[1].remove(location);
    }

    pub fn double_attack_map(&self, attacking_player : Player) -> HashSet<Tile>{
        let mut single_attacks = HashSet::new();
        let mut double_attacks = HashSet::new();

        self.get_pieces(attacking_player).iter()
        .for_each(|(t,&species)|
            neighbours_attack(*t,Piece{color:attacking_player,species}).into_iter().flatten()
            .for_each(|n|
                if single_attacks.remove(&n){
                    double_attacks.insert(n);
                } else{
                    single_attacks.insert(n);
                }
                
            )
        );

        double_attacks
    }

    pub async fn moves_with_score(self, depth : usize, mquad_frame_await : bool) -> Vec<(Ply, EvalResult)>{
        
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

        let mut transp_table = TranspositionalTable::new();

        

        for m in self.valid_moves(){
            if mquad_frame_await{
                next_frame().await;
            }

            let mut copy = self.clone();
            copy.apply_move(m);
            let evaluation = copy.eval(depth-1, &mut transp_table).await;
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
    
    async fn eval(self, depth : usize, transp : &mut TranspositionalTable) -> EvalResult{
        self.eval_alphabeta(depth, Score::win_now(Player::Black), Score::win_now(Player::White), transp).await
    }

    fn is_won_home(&self) -> Option<Player>{
        for defender in [Player::White,Player::Black]{
            let attacker = defender.flip();
            if let Some(&species) = self.get_pieces(attacker).get(&Tile::corner(defender)){
                if species == PieceType::Flat{
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
        let opponent_house = Tile::corner(player.flip());
        let opponent_pieces = self.get_pieces(player.flip());
        let active_pieces = self.get_pieces(player);

        let mut active = HashSet::new();
        active.insert(opponent_house);

        

        for current_distance in 0..=4{
            let mut new_active = HashSet::new();
            for tile in active.drain(){
                if let Some(piece) = active_pieces.get(&tile){
                    match piece {
                        PieceType::Flat => return current_distance,
                        PieceType::Stack(..) => return current_distance+1,
                        PieceType::Lone(..) => {}
                    };
                };

                neighbours_move(tile, Piece{color : player.flip(), species:PieceType::Flat})
                .iter().for_each(|n|{
                    if let Some(n) = n{
                        match opponent_pieces.get(&n){
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
        match self.passed_flat_distance(player){
            0 => 200.0,
            1 => 30.0,
            2 => 10.0,
            3 => 5.0,
            4 => 1.0,
            _ => 0.0
        }
    }

    pub fn eval_heuristic(&self) -> Score{
        if let Some(winner)  = self.is_won_home(){
            return Score::win_now(winner);
        }
        

        let finite_score = [Player::White,Player::Black].into_iter()
        .map(|color|{
            let multiplier = match color{
                Player::Black => -1f32,
                Player::White => 1f32
            };
            self.get_pieces(color).iter().map(|(t,&species)|{

                let signed_piece_value = multiplier * species.value();
    
                let horizontal_pos = t.x()+2*t.y();
                assert!((-6..=6).contains(&horizontal_pos));
    
                let prox_x = (horizontal_pos as f32 * multiplier + 6.0) / 12.0;
    
                let prox_score = prox_x.powf(2.0);
    
                let location_score = 0.1 * prox_score * multiplier * species.positional_weight();
    
    
                signed_piece_value + location_score
            }
            ).sum::<f32>() as f32
        }).sum::<f32>() as f32;
        
        
        let mobility_score = 0.1 * (
            self.valid_moves_for(Player::White).len() as f32
            - self.valid_moves_for(Player::Black).len() as f32
        );


        let passed_flat_score = self.passed_flat_score(Player::White) - self.passed_flat_score(Player::Black);

        Score::finite(finite_score + mobility_score + passed_flat_score)
    
    }

    fn eval_alphabeta(self, depth : usize, alpha : Score, beta : Score, transp : &mut TranspositionalTable) -> BoxFuture<'_,EvalResult>{
        // const NODES_PER_FRAME : usize = 500;
        async move {
            if let Some(score) = transp.query(self.zobrist_hash, depth){
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
                                let mut moves_heuristic : Vec<(Ply, Score)> = self.valid_moves().into_iter().map(|ply|{
                                    let mut hc = self.clone();
                                    hc.apply_move(ply);
                                    (ply,hc.eval_heuristic())
                                }).collect();
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
                            copy.apply_move(m);
                            
                            let sub_zobhash = copy.zobrist_hash.clone();
                            let sub_result = copy.eval_alphabeta(depth-1, alpha, beta, transp).await;
                            transp.insert(sub_zobhash, depth-1, sub_result.score);

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
        }.boxed()
    }


    pub fn max_white_flat_hor(&self) -> Option<i8>{
        self.get_pieces(Player::White).iter()
        .filter(|(_,species)|
            match species{
                PieceType::Flat | PieceType::Stack(..) => true,
                _ => false
            }
        )

        .map(|(t,_)|t.x() + 2*t.y())
        .max()
    }
}

struct TranspositionalTable(HashMap<ZobristHash, (usize,Score)>);

impl TranspositionalTable{
    pub fn new()->Self{
        Self(HashMap::new())
    }

    pub fn query(&self, hash : ZobristHash, min_depth : usize) -> Option<Score>{
        if let Some((depth, score)) = self.0.get(&hash){
            if *depth >= min_depth {
                Some(*score)
            } else {None}
        } else {None}
    }

    pub fn insert(&mut self, hash : ZobristHash, depth : usize, score : Score){
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
    state_before : State,
    state_after : State,
    ply : Ply,
    moved_piece : PieceType,

    disambiguate : bool,
    kills : Vec<(Tile,PieceType)>,
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
                PieceType::Flat => "F",
                PieceType::Lone(tall) => match tall{
                    Tall::Blind => "B",
                    Tall::Hand => "A",
                    Tall::Star => "S"
                },
                PieceType::Stack(..) => "?"
            },

            move_rep,

            (0..self.kills.len()).map(|_|'*').collect::<String>()
        )
    }
}