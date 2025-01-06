use std::collections::HashSet;

use egui::{Color32, FontFamily, FontId, Id, Layout, Margin, TextStyle, Ui};
use hexstack::{Ply, EvalResult, State, Tile};
use itertools::Itertools;
use macroquad::prelude::*;

use macroquad::experimental::coroutines::{start_coroutine,Coroutine};


struct AppState{
    history : Vec<(State,Ply)>,
    game_state : State,
    computing_moves_future : Option<Coroutine<Vec<(Ply,EvalResult)>>>,
    available_moves : Option<Vec<(Ply,EvalResult)>>,
    evaluating_depth : usize,
    last_eval_depth : usize,

    piece_tex : Texture2D,
    font : Font,

    // attack_maps : HashMap<Player, HashMap<Tile, u8>>,

    selected_tile : Option<Tile>,

    display_attacks : bool,
    display_tile_numbers : bool,
    flip_board : bool,


    hovered_move : Option<Ply>,
    hovered_tile : Option<Tile>,

    max_depth : usize,
}

impl AppState{
    async fn new()->AppState{
        let font = load_ttf_font("gfx/Roboto-Regular.ttf")
            .await
            .unwrap();
        font.set_filter(FilterMode::Linear);

        // let chars : [char;7] = std::array::from_fn(|i| coord_to_char(i as i8-3));

        // font.populate_font_cache(&chars, 32);

        let app_state = AppState{
            history : vec![],
            game_state : State::setup(),
            computing_moves_future : None,
            available_moves : None,
            piece_tex : load_texture("gfx/pieces_sm.png").await.unwrap(),
            font ,
            // attack_maps : HashMap::new(),
            display_attacks : false,
            display_tile_numbers : false,
            flip_board : false,
            selected_tile : None,
            evaluating_depth : 0,
            last_eval_depth : 0,

            hovered_move : None,
            hovered_tile : None,
            max_depth : 4,
        };
        // app_state.refresh();
        app_state
    }
    
    fn setup(&mut self){
        self.game_state = State::setup();
        self.history = vec![];
        self.refresh();
    }

    fn recompute_engine_eval(&mut self){
        let comp_corout = start_coroutine(self.game_state.clone().moves_with_score(self.evaluating_depth));
        // let future = self.game_state.clone().moves_with_score(6); 
        // let boxed = Box::pin(future);
        self.computing_moves_future = Some(comp_corout);
    }

    fn refresh(&mut self){
        self.available_moves = None;
        self.evaluating_depth = 0;

        self.recompute_engine_eval();
        
        // self.attack_maps = HashMap::new();

        // for attacking_player in [Player::White,Player::Black]{
        //     self.attack_maps.insert(attacking_player, self.game_state.attack_map(attacking_player));
        // }
        
        self.selected_tile = None
    }



    fn click_tile(&mut self, tile : Tile){
        if let Some(from_tile) = self.selected_tile{
            let to_tile = tile;
            let ply = Ply{ from_tile, to_tile };
            
            if let Some(moves) = &self.available_moves{
                let plies : HashSet<Ply> = moves.iter().map(|(p,_)|p.clone()).collect();
                if plies.contains(&ply){
                    self.apply_move(ply);
                    return;
                }
            };

            self.selected_tile = None;
        } else {
            self.selected_tile = Some(tile);
        }   
    }

    fn apply_move(&mut self, ply : Ply){
        self.history.push((self.game_state.clone(),ply));
        self.game_state.apply_move(ply);
        self.refresh();
    }

    fn undo_move(&mut self){
        if let Some((prev_state,_)) = self.history.pop(){
            self.game_state = prev_state;
            self.refresh();
        }
    }

    fn ui_control_panel(&mut self, ui : &mut Ui){
        ui.horizontal(|ui|{
            if ui.button("New game").clicked(){
                self.setup();
            }
        });

        ui.horizontal(|ui|{
            ui.checkbox(&mut self.display_attacks, "Display attacks");
            ui.checkbox(&mut self.flip_board, "Flip board");
            ui.checkbox(&mut self.display_tile_numbers, "Tile numbers")
        });
    }

    fn ui_history(&mut self, ui : &mut Ui){
        
        egui::ScrollArea::horizontal()
        .max_height(60.0)
        .id_source("history")
        .show(ui,|ui|{
            ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui|{
                self.history.iter().chunks(2).into_iter().enumerate().for_each(|(i,mut plies)|{
                    let move_num = i+1;
                    let (_,p1) = plies.next().unwrap();

                    
                    
                    ui.label(egui::RichText::new(format!("{}.",move_num)).strong());
                    ui.label(format!("{}",p1));

                    if let Some((_,p2)) = plies.next() {
                        ui.label(format!("{}",p2));
                    };
                    
                });

                if self.history.len() > 0{
                    let undo_btn = ui.button("Undo");
                    if undo_btn.clicked(){
                        self.undo_move();
                    }
                    undo_btn.scroll_to_me(None);
                }
            })
            
        });
    }

    fn ui_engine_eval(&mut self, ui : &mut Ui){
        ui.heading("Engine Evaluation");
        

        ui.horizontal(|ui|{
            let ranger = ui.add(egui::DragValue::new(&mut self.max_depth).clamp_range(0..=7));
            if ranger.changed(){
                self.evaluating_depth = self.max_depth;
                self.recompute_engine_eval();
            }

            ui.label("Max search depth.");
        });

        ui.label(format!("Computed at {}-ply depth.",self.last_eval_depth));

        egui::ScrollArea::vertical().id_source("engine_evals").show(ui,|ui|{
            ui.style_mut().text_styles.insert(TextStyle::Button, FontId { size: 16.0, family: FontFamily::Proportional });

            self.hovered_move = None;    
            self.available_moves.clone().map(|plies|
                plies.iter().for_each(|(p,ev)|{
                    let button = ui.button(format!("{} {} [{}]",ev.score,p,ev.nodes));
                    if button.hovered(){
                        self.hovered_move = Some(p.clone())
                    }

                    if button.clicked(){
                        self.apply_move(p.clone());
                    }
                }));
        });
    }

    fn egui_ui(&mut self, ui : &mut Ui){

        let styles = ui.style_mut();
        
        styles.text_styles.insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
        styles.text_styles.insert(TextStyle::Button, FontId::new(18.0, FontFamily::Proportional));
        styles.text_styles.insert(TextStyle::Heading, FontId::new(32.0,FontFamily::Proportional));

        let visuals = ui.visuals_mut();


        visuals.window_fill = Color32::WHITE;
        visuals.panel_fill = Color32::WHITE;
        
        

        visuals.override_text_color = Some(Color32::from_gray(20));

        let widgets_bg = Color32::from_gray(200);
        visuals.widgets.inactive.bg_fill = widgets_bg;
        visuals.widgets.inactive.weak_bg_fill = widgets_bg;

        ui.vertical(|ui|{
                    ui.vertical(|ui|{
                        self.ui_control_panel(ui);
        
                        self.ui_history(ui);
                    });

                    ui.separator();
        
                    ui.vertical(|ui|{
                        self.ui_engine_eval(ui);
                    })
                });
          
        
    }

    async fn process(&mut self, mouse_world : Vec2){
        if let Some(future) = &mut self.computing_moves_future{
            match future.retrieve(){
                None => {},
                Some(value) => {
                    self.available_moves = Some(value);
                    self.last_eval_depth = self.evaluating_depth;

                    if self.evaluating_depth >= self.max_depth{
                        self.computing_moves_future = None
                    } else {
                        self.evaluating_depth += 1;
                        self.recompute_engine_eval();
                    }
                }
            }
        }


        if let Some(ref moves) =  self.available_moves{
            if let Some((ply,_)) = moves.first(){
                if is_key_down(KeyCode::Space){
                    self.apply_move(ply.clone());
                }
            }
        }
            
        
        egui_macroquad::ui(|egui_ctx| {
            egui_ctx.set_visuals(egui::Visuals::light());
            egui::SidePanel::new(egui::panel::Side::Left,Id::new("game_panel"))
            .frame(egui::Frame{
                fill: Color32::TRANSPARENT,
                inner_margin : Margin::same(20.0),
                ..Default::default()
            })
            .show(egui_ctx, |ui| {
                self.egui_ui(ui);
            });
        });

        self.hovered_tile = Tile::from_world(mouse_world.x,mouse_world.y, self.flip_board);

        if let Some(hovered_tile) = self.hovered_tile{
            if is_mouse_button_pressed(MouseButton::Left) {
                self.click_tile(hovered_tile);
            }
        }

        self.game_state.draw(
            self.piece_tex, self.font,
            self.flip_board,
            self.display_attacks,
            self.display_tile_numbers
        );

        if let Some(t) = self.selected_tile{
            t.draw_highlight(0.2, GREEN, self.flip_board);


            if let Some(moves) = &self.available_moves {
                moves.iter()
                .filter(|(m,_)|m.from_tile == t)
                .map(|(m,_)|m.to_tile)
                .for_each(|target|target.draw_move_target(self.flip_board));
            };
        }

        if let Some(hovered_move) = self.hovered_move{
            hovered_move.draw(self.flip_board);
        }

        if let Some(selected_tile) = self.hovered_tile{
            selected_tile.draw_highlight(0.1,GRAY,self.flip_board);
        }

        egui_macroquad::draw();
    }
}

fn window_conf()->Conf{
    Conf{
        window_title : "board game".to_owned(),
        window_resizable : true,
        window_width : 1280,
        window_height : 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = AppState::new().await;

    state.setup();


    loop{
        clear_background(Color::from_hex(0xeeeeee));        
        
        let cam = Camera2D{
            target : vec2(-3.0,0.0),
            zoom : 0.13*vec2(screen_height()/screen_width(),-1.0),
            ..Default::default()
        };
        set_camera(&cam);

        let mouse_px = mouse_position().into();
        let mouse_world = cam.screen_to_world(mouse_px);

        state.process(mouse_world).await; 

        next_frame().await
    }
}
