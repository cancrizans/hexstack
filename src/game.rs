use std::collections::{HashMap, HashSet};

use crate::assets::Assets;
use crate::board::Piece;
use crate::theme::set_theme;
use crate::ui::{Button, MqUi};
use crate::{theme, EvalResult, HistoryEntry, Player, Ply, State, Tile};
use egui::{Color32, Id, Margin};
use itertools::Itertools;
use macroquad::audio::{play_sound, play_sound_once, PlaySoundParams};
use macroquad::prelude::*;

use macroquad::experimental::coroutines::{start_coroutine,Coroutine};
use ::rand::distributions::Open01;
use ::rand::Rng;

const MOVE_ANIM_DURATION : f32 = 0.15;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GamerSpec{
    Human,
    Gibberish,
    Noob,
    Decent,
    Sharp,
    Tough,
    Beastly
}

impl GamerSpec{
    pub fn name(&self) -> &str{
        self.texts().0
    }
    pub fn description(&self) -> &str{
        self.texts().1
    }

    pub fn texts(&self) -> (&str,&str){
        match self{
            GamerSpec::Human => ("Human", "Human player."),
            GamerSpec::Gibberish => ("Gibberish", "Random moves."),
            GamerSpec::Noob => ("Noob", "Poor player."),
            GamerSpec::Decent => ("Decent", "Solid player."),
            GamerSpec::Sharp => ("Sharp", "Serious challenge."),
            GamerSpec::Tough => ("Tough", "Very strong."),
            GamerSpec::Beastly => ("Beastly", "Is it even beatable?")
        }
    }

    fn make(self, assets : &Assets, allow_takeback : bool) -> Box<dyn Gamer>{
        match self{
            GamerSpec::Human => Human::new_boxed(assets, allow_takeback),
            GamerSpec::Gibberish => Bot::new_boxed(0,0.0),
            GamerSpec::Noob => Bot::new_boxed(1, 0.2),
            GamerSpec::Decent => Bot::new_boxed(2, 0.2),
            GamerSpec::Sharp => Bot::new_boxed(3, 0.4),
            GamerSpec::Tough => Bot::new_boxed(5, 0.6),
            GamerSpec::Beastly => Bot::new_boxed(6, 0.0)
        }
    }
}

#[derive(Clone, Copy)]
pub struct MatchConfig{
    pub gamers : [GamerSpec;2],
    pub gamer_one_color : Option<Player>,
    pub allow_takeback : bool,
}


struct FatGameState{
    state : State,
    valid_moves : Vec<Ply>,
    is_won : Option<Player>,

    history : Vec<HistoryEntry>,
}

impl FatGameState{
    fn setup()->FatGameState{
        let state = State::setup();
        let valid_moves = state.valid_moves();
        FatGameState{
            state,
            valid_moves,
            is_won : None,
            history : vec![]
        }
    }

    fn refresh(&mut self){
        self.valid_moves = self.state.valid_moves();
        self.is_won = self.state.is_won();
    }

    fn is_won(&self) -> Option<Player>{
        self.is_won
    }

    fn apply_move(&mut self, ply : Ply){
        assert!(self.is_won.is_none());

        let entry = self.state.compute_history_entry(ply);
        self.history.push(entry);

        self.state.apply_move(ply);
        self.refresh();
    }

    fn to_play(&self) -> Player{
        self.state.to_play
    }

    fn draw(&self, piece_tex : Texture2D, font : Font){
        
        self.state.draw(piece_tex, font, false, false, false);

        
    }


    fn state_clone(&self) -> State{
        self.state.clone()
    }

    fn undo_moves(&mut self, count : usize){
        (0..count).for_each(|_|
            if let Some(entry) = self.history.pop(){
                self.state = entry.state_before;
            }
        );

        self.refresh();
    }

}

#[derive(Clone, Copy)]
enum Decision{
    Move(Ply),
    TakeBack
}

trait Gamer{
    fn assign_puzzle(&mut self, state : State);
    fn poll_answer(&mut self) -> Option<Decision>;
    fn process(&mut self, ui : &MqUi, as_player : Player);

    fn avatar_offset(&self) -> usize;
}


struct Bot{
    depth : usize,
    blundering_probability : f32,

    result_future : Option<Coroutine<Vec<(Ply,EvalResult)>>>,
    last_used_depth : Option<usize>,
}

impl Bot{
    fn new(depth : usize, blundering_probability : f32) -> Bot{
        Bot { 
            depth ,
            blundering_probability,
            result_future : None,
            last_used_depth : None
        }
    }
    
    fn new_boxed(depth : usize, blundering_probability : f32) -> Box<Bot>{
        Box::new(Self::new(depth,blundering_probability))
    }
}

impl Gamer for Bot{
    fn assign_puzzle(&mut self, state : State) {
        let mut depth = self.depth;

        let mut rng = ::rand::thread_rng();
        while rng.sample::<f32,Open01>(Open01) < self.blundering_probability {
            depth = depth.saturating_sub(1)
        }

        self.last_used_depth = Some(depth);

        self.result_future = Some(start_coroutine(state.moves_with_score(depth)));
    }

    fn poll_answer(&mut self) -> Option<Decision> {
        let answer = self.result_future.as_ref().map(|future|
            future.retrieve().map(|evals|evals.first().unwrap().0)
        ).flatten();

        if answer.is_some() {self.result_future = None;}

        answer.map(|rep|Decision::Move(rep))
    }

    fn process(&mut self, _ui : &MqUi, _as_player : Player){
        // let (x,y) = as_player.ui_info_pos().into();

        // let tag = format!("Bot {} {}",
        //     if let Some(lud) = self.last_used_depth{
        //         format!("({}-ply depth)", lud)
        //     } else {"".to_string()},
        //     match self.result_future{
        //         None => "",
        //         Some(..) => "thinking..."
        //     }
        // );

        // let (font_size, font_scale, font_scale_aspect) = camera_font_scale(0.8);
        // draw_text_ex(
        //     &tag,
        //     x-3.0,
        //     y,
        //     TextParams{font,font_scale,font_scale_aspect,font_size,
        //         color : Color::from_hex(0x111111),
        //         ..Default::default()
        //     }
        // );
    }

    fn avatar_offset(&self) -> usize {1}
}

struct Human{
    selected_tile : Option<Tile>,
    puzzle_state : Option<State>,
    available_moves : Option<HashSet<Ply>>,
    answer : Option<Decision>,
    piece_tex : Texture2D,

    allow_takeback : bool,

    btn_takeback : Button,
}

impl Human{
    fn new(assets : & Assets, allow_takeback : bool)->Self{
        Human { 
            selected_tile: None, 
            puzzle_state: None,
            available_moves : None,
            answer : None,
            piece_tex : assets.pieces,
            btn_takeback : Button::new(
                assets.btn_takeback,
                Rect::new(7.0,0.0,1.0,1.0),
                "Undo Move".to_string()
            ),

            allow_takeback,
         }
    }

    fn new_boxed(assets : &Assets, allow_takeback : bool)->Box<Self>{
        Box::new(Self::new(assets , allow_takeback))
    }

    fn reset(&mut self){
        self.puzzle_state = None;
        self.selected_tile = None;
        self.answer = None;
        self.available_moves = None;
    }

    fn mouse_tile(cam : &Camera2D) -> Option<Tile>{
        let mouse_px = mouse_position().into();
        let mouse_world = cam.screen_to_world(mouse_px);
        Tile::from_world(mouse_world.x, mouse_world.y, false)
    }
}

impl Gamer for Human{
    fn assign_puzzle(&mut self, state : State) {
        self.reset();
        self.available_moves = Some(HashSet::from_iter(state.valid_moves().into_iter()));
        self.puzzle_state = Some(state);
        
    }

    
    fn poll_answer(&mut self) -> Option<Decision> {
        if self.answer.is_some(){
            let output = self.answer.clone();
            self.reset();
            output
        } else {None}
        
    }

    fn process(&mut self, ui : &MqUi, as_player : Player) {

        if let Some(av_moves) = &self.available_moves{
            if self.allow_takeback{
                if self.btn_takeback.process(&ui){
                    self.answer = Some(Decision::TakeBack);
                }
            }

            if let Some(selected) = self.selected_tile{
                av_moves.iter().filter(|p|p.from_tile == selected)
                    .for_each(|p|p.to_tile.draw_move_target(as_player,self.piece_tex, false));
            }

            if let Some(mouse_hover) = Self::mouse_tile(ui.camera){
                mouse_hover.draw_highlight_outline(0.05, WHITE, false); 

                if let Some(selected) = self.selected_tile{
                    if is_mouse_button_pressed(MouseButton::Left){
                        let candidate_ply = Ply{from_tile : selected, to_tile : mouse_hover};
                        if av_moves.contains(&candidate_ply){
                            self.answer = Some(Decision::Move(candidate_ply));
                        } else {
                            self.selected_tile = None;
                        }
                    }
                } else {
                    if is_mouse_button_pressed(MouseButton::Left){
                        if av_moves.iter().any(|p|p.from_tile == mouse_hover){
                            self.selected_tile = Some(mouse_hover);
                        }
                    }
                }

                
            }

            if let Some(selected) = self.selected_tile{
                selected.draw_highlight_outline(0.1, BLACK, false);
            }
               
        }
    
    }

    fn avatar_offset(&self) -> usize {0}
}


struct MoveAnimState{
    time : f32,
    ply : Ply,
    moved_piece : Piece,
    drawing_state : State,
    kills : Vec<(Tile,Piece)>
}

impl MoveAnimState{
    fn new(ply : Ply, game_state : State) -> Self{
        let mut drawing_state = game_state;

        let mut kill_copy_state = drawing_state.clone();
        kill_copy_state.stage_translate(ply);
        let kills = kill_copy_state.stage_attack_scan(drawing_state.to_play);

        let moved_piece = drawing_state.pull_moving_piece(ply.from_tile);
        kills.iter().for_each(|(kt,_)|drawing_state.clear_tile(kt));

        
        MoveAnimState { 
            time: 0.0, 
            ply, 
            moved_piece, 
            drawing_state ,
            kills
        }
    }

    fn tick(&mut self){
        self.time += get_frame_time()
    }
}

enum GameStateMachine{
    Setup,
    Polling,
    Animating(MoveAnimState),
    Won{
        winner : Player
    }
}



struct GameApp<'a>{
    assets : &'a Assets,
    
    game_state : FatGameState,
    
    piece_tex : Texture2D,
    


    gamers : HashMap<Player, Box<dyn Gamer>>,

    last_touched_tiles : Option<[Tile;2]>,
    last_kill_tiles : Vec<Tile>,

    app_state : GameStateMachine,

    attack_patterns_alpha : f32,
    attack_patterns_toggle : bool,

    smoothed_to_play : f32,

    tile_letters_toggle : bool,
    
    btn_tile_letters : Button,
    btn_toggle_lines : Button,
    btn_exit : Button,
    
}

impl<'a> GameApp<'a>{
    async fn new(
            assets : &'a Assets,
            match_config : MatchConfig
        )->GameApp<'a>{
        

        let mut gamers :HashMap<Player, Box<dyn Gamer>> = HashMap::new();
        
        let first_gamer_color = if let Some(color) = match_config.gamer_one_color{
            color
        } else {
            if ::rand::thread_rng().gen::<bool>() {
                Player::White
            } else {
                Player::Black
            }
        };
        
        let piece_tex = assets.pieces;
        
        let [gm0,gm1] = match_config.gamers.map(
            |s|s.make(assets, match_config.allow_takeback));

        
        gamers.insert(first_gamer_color, gm0);
        gamers.insert(first_gamer_color.flip(), gm1);


            // (Player::White, Box::new(Bot::new(3))),
            // (Player::Black, Box::new(Human::new()))
            // ]);
 
        let app_state = GameApp{
            assets,
            
            game_state : FatGameState::setup(),

            piece_tex,
            


            gamers ,
            
            last_touched_tiles : None,
            app_state : GameStateMachine::Setup,

            last_kill_tiles : vec![],
            attack_patterns_alpha : 0.0,
            attack_patterns_toggle : false,
            tile_letters_toggle : false,

            smoothed_to_play : 0.0,

            btn_tile_letters : Button::new(
                assets.btn_letters,
                Rect::new(7.0,1.0,1.0,1.0),
                "Show Tiles".to_string()
            ),

            btn_toggle_lines : Button::new(
                assets.btn_lines,
                Rect::new(7.0,2.0,1.0,1.0),
                "Show Patterns".to_string()
            ),

            btn_exit : Button::new(
                assets.btn_exit,
                Rect::new(7.0,3.0,1.0,1.0),
                "Exit".to_string()
            ),
        };

        
        
        app_state
    }
    
    fn ask(&mut self){
        self.gamers.get_mut(&self.game_state.to_play()).unwrap().assign_puzzle(self.game_state.state.clone());
        self.app_state = GameStateMachine::Polling;
    }

    fn apply_move(&mut self, ply : Ply){
        self.last_kill_tiles = vec![];

        
        self.assets.piece_slide.play();
        self.app_state = GameStateMachine::Animating(MoveAnimState::new(ply,self.game_state.state_clone()));

        self.game_state.apply_move(ply);
        self.last_touched_tiles = Some([ply.from_tile,ply.to_tile]);


        
    }

    fn undo_moves(&mut self, count : usize){
        self.game_state.undo_moves(count);
    
        self.last_kill_tiles = vec![];
        self.last_touched_tiles = None;

        self.ask();    
    }

    async fn process(&mut self) -> bool{
        let delta_t = get_frame_time();

        self.attack_patterns_alpha += 5.0 *(
            (if self.attack_patterns_toggle {1.0} else {0.0}) - self.attack_patterns_alpha
        ) * delta_t;

        self.smoothed_to_play += 6.0 * (
            (match self.game_state.to_play() {Player::Black => 1.0, Player::White => 0.0}) - self.smoothed_to_play
        ) * delta_t;


        let cam = Camera2D{
            target : vec2(0.0,0.0),
            zoom : 0.13*vec2(screen_height()/screen_width(),-1.0),
            ..Default::default()
        };
        set_camera(&cam);
        let mqui = MqUi::new(self.assets, &cam);



        egui_macroquad::ui(|egui_ctx| {
            egui_ctx.set_visuals(egui::Visuals::light());
            egui_ctx.set_pixels_per_point(screen_height() / 720.0);
            egui::SidePanel::new(egui::panel::Side::Left,Id::new("game_panel"))
            .frame(egui::Frame{
                fill: Color32::TRANSPARENT,
                inner_margin : Margin::same(20.0),
                ..Default::default()
            })
            .show(egui_ctx, |ui| {
                set_theme(ui);
                
                

                egui::ScrollArea::vertical()
                .max_width(300.0)
                .id_source("history")
                .show(ui,|ui|{
                    ui.set_max_width(150.0);
                    ui.set_min_width(150.0);
                    ui.vertical(|ui|{
                        self.game_state.history.iter().chunks(2)
                        .into_iter().enumerate().for_each(|(i,mut plies)|{
                            let move_num = i+1;
                            let p1 = plies.next().unwrap();
        
                            
                            ui.horizontal(|ui|{
                                ui.label(egui::RichText::new(format!("{}.",move_num)).strong());
                                ui.label(format!("{}",p1));
            
                                if let Some(p2) = plies.next() {
                                    ui.label(format!("{}",p2));
                                };
                                
                            });
                            
                            
                        });
                        
                        ui.label("").scroll_to_me(None);
                    });
                });
            });
        });
        egui_macroquad::draw();


        match &mut self.app_state{
            GameStateMachine::Setup => {
                self.ask();
            },
            GameStateMachine::Polling => {
                if let Some(_winner) = self.game_state.is_won() {

                } else {
                    let to_move = self.game_state.to_play();
                    let gamer = self.gamers.get_mut(&to_move).unwrap();
        
                    match gamer.poll_answer() {
                        None => {},
                        Some(reply) => 
                            match reply {
                                Decision::Move(ply) => {
                                    self.apply_move(ply);
                                },
                                Decision::TakeBack => {
                                    self.undo_moves(2);
                                }
                            }
                        
                        
                    }
                }
            },
            GameStateMachine::Animating(ref mut anim_state) => {
                anim_state.tick();
                if anim_state.time > MOVE_ANIM_DURATION{

                    
                    self.last_kill_tiles = anim_state.kills.iter().map(|(t,_)|*t).collect();

                    if let Some(winner) = self.game_state.is_won(){
                        play_sound_once(self.assets.mate);
                        self.app_state = GameStateMachine::Won { winner }
                    } else {
                        if anim_state.kills.len()>0{
                            play_sound(self.assets.capture,PlaySoundParams{
                                looped : false, volume : 0.5
                            });
                        }
                        self.ask();
                    }
                }
                
            },

            GameStateMachine::Won { .. } => {}

        }

        

        Tile::draw_board(false);

        match self.app_state{
            GameStateMachine::Won { winner } => {
                self.game_state.state.pieces.iter().for_each(|(t,p)|{
                    let col = if p.color == winner {
                        Color::from_hex(0x66dd66)
                    } else {
                        Color::from_hex(0xdd6666)
                    };

                    t.draw_highlight_fill(col, false);
                });
            },
            _ => {}
        }

        if let Some([from,to]) = self.last_touched_tiles{
            for (t,col) in [(from, Color::from_rgba(0xee, 0xdd,0x11, 90)), (to, Color::from_hex(0xeedd11))]{
                t.draw_highlight_fill(col, false)
            }
        }

        self.last_kill_tiles.iter().for_each(|kt|
            kt.draw_highlight_fill(Color::from_hex(0xddaaaa), false)
        );

        if self.attack_patterns_alpha > 0.001{
            self.game_state.state.draw_attacks(false, self.attack_patterns_alpha);
        }

        match &self.app_state{
            GameStateMachine::Animating(anim_state) => {
                anim_state.drawing_state.draw(self.piece_tex, self.assets.font, false,false,false);
                anim_state.ply.draw(false);



                let start:Vec2 = anim_state.ply.from_tile.to_world(false).into();
                let end = anim_state.ply.to_tile.to_world(false).into();

                let t = (anim_state.time / MOVE_ANIM_DURATION).clamp(0.0, 1.0);
                let lambda = t*t*(3.0-2.0*t);
                let pos = start.lerp(end,lambda );
                anim_state.moved_piece.draw(pos.x, pos.y, self.piece_tex, 1.0);


                let kill_scale = (1.0-t).powf(2.0) * 1.3;

                anim_state.kills.iter().for_each(|(t,p)|
                    {   
                        let (x,y) = t.to_world(false);
                        p.draw(x, y, self.piece_tex, kill_scale);
                    }
                );
            },
            _ => {

                self.game_state.draw(self.piece_tex, self.assets.font);
                
            }
        };

        // self.game_state.draw_history(self.assets.font);

        if self.tile_letters_toggle{
            Tile::draw_tile_numbers(self.assets.font, false);
        }

        for player in [
                self.game_state.to_play(),
                self.game_state.to_play().flip()
            ]{
            let gamer = self.gamers.get_mut(&player).unwrap();
            gamer.process(&mqui,player);


            let strength = match player{
                Player::Black => self.smoothed_to_play,
                Player::White => 1.0 - self.smoothed_to_play
            };
            

            let size = vec2(2.0,2.0).lerp(vec2(3.0,3.0), strength);
            let pos = player.ui_info_pos() - size*0.5;

            let (avatar_tex,avatar_src) = self.assets.get_avatar(player, gamer.avatar_offset());

            draw_texture_ex(
                avatar_tex, 
                pos.x, 
                pos.y, 
                Color::from_vec(vec4(1.0,1.0,1.0,strength*0.5+0.5)), 
                DrawTextureParams{
                    source : Some(avatar_src),
                    dest_size : Some(size),
                    ..Default::default()
                }
            );
        }


        
        if self.btn_tile_letters.process(&mqui){
            self.tile_letters_toggle ^= true;
        }

        if self.btn_toggle_lines.process(&mqui){
            self.attack_patterns_toggle ^= true;
        };
        if self.btn_exit.process(&mqui){
            return true;
        };

        false
    }
}

pub fn window_conf()->Conf{
    Conf{
        window_title : "board game".to_owned(),
        window_resizable : false,
        window_width : 1280,
        window_height : 720,
        ..Default::default()
    }
}


pub async fn main(assets : &Assets,match_config : MatchConfig) {
    let mut state = GameApp::new(
        assets,
        match_config
    ).await;

    loop{
        clear_background(theme::BG_COLOR);        
        
        let quit = state.process().await; 
        if quit{
            break;
        }

        next_frame().await
    }
}
