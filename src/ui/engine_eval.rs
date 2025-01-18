use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::{assets::ASSETS, theme::{self, set_theme}, tokonoma::{Captured, EvalResult, HistoryEntry, TranspositionalTable}, Player, Ply, Position};
use coroutines::stop_all_coroutines;
use egui::Margin;
use macroquad::prelude::*;
use macroquad::experimental::coroutines::{start_coroutine,Coroutine,stop_coroutine};

use super::editor::PositionEditor;

type EngineResults = Vec<(Ply, HistoryEntry, EvalResult)>;

pub struct EngineEvalUI{
    editor : PositionEditor,

    max_depth : usize,
    job : Option<Coroutine<(usize,EngineResults)>>,
    results : Option<(usize,EngineResults)>,

    table : Arc<Mutex<TranspositionalTable>>,

    last_position_hash : u64,
}

impl EngineEvalUI{
    pub fn new(start_position : Position)->EngineEvalUI{
        let editor = PositionEditor::from_state(start_position);
        let hash = editor.tabulation_hash();
        EngineEvalUI{
            editor ,
            max_depth : 6,
            job : None,
            results : None,
            table : Arc::new(Mutex::new(TranspositionalTable::new())),
            last_position_hash : hash,
        }
    }


    fn set_dirty(&mut self){
        self.results = None;
        if let Some(..) = &self.job{
            // stop_coroutine(_coroutine);
            stop_all_coroutines();
            self.job = None;
        }
    }

    async fn the_job(position : Position, depth : usize, transp : Arc<Mutex<TranspositionalTable>>) -> (usize, EngineResults){
        let mquad_frame_await = depth > 5;
        let dummy_captures = HashMap::from([(Player::White,Captured::empty()),(Player::Black,Captured::empty())]);

        let results : EngineResults = position.clone()
        .moves_with_score(depth, mquad_frame_await, Some(transp))
        .await
        .into_iter().map(move |(ply,eval)|(ply,position.compute_history_entry(ply, dummy_captures.clone()),eval))
        .collect();


        (depth, results)
    }

    fn start_scan(&mut self,  depth : usize){
        let position =  self.editor.get_state_clone();
        self.job = Some(start_coroutine(Self::the_job(position, depth, self.table.clone())))
    }

    fn recompute(&mut self){
        self.set_dirty();
        

        self.start_scan(0);
    }

    fn apply_move(&mut self, ply : Ply){
        let mut pos = self.editor.get_state_clone();
        pos.apply_move(ply);
        self.editor.set_position(pos);
    }

    pub async fn run(&mut self) -> Position{
        let mut quit = false;
        self.recompute();

        loop {
            clear_background(theme::BG_COLOR);

            if self.editor.tabulation_hash() != self.last_position_hash{
                self.recompute();
                self.last_position_hash = self.editor.tabulation_hash();
            }

            if let Some(job) = &self.job {
                if let Some((depth, results)) = job.retrieve(){
                    self.results = Some((depth,results));

                    if depth < self.max_depth{
                        self.start_scan(depth+1);
                    }
                }
            }

            self.editor.process(&Camera2D{
                target : vec2(5.0,-1.5),
                zoom : vec2(screen_height()/screen_width(), -1.0) * 0.135,
                ..Default::default()
            });

            set_default_camera();

            egui_macroquad::ui(|egui_ctx|{
                egui_ctx.set_pixels_per_point(screen_height() / 720.0);
                egui_ctx.set_visuals(egui::Visuals::light());
                egui::SidePanel::right(egui::Id::new("engine_eval"))
                .frame(
                    egui::Frame::none()
                    .inner_margin(Margin::symmetric(75.0,30.0))
                    // .fill(panel_col)

                )
                .resizable(false).show_separator_line(true)
                .show(egui_ctx, |ui|{
                    set_theme(ui);

                    ui.horizontal(|ui|{
                        if ui.button("Play from here").clicked(){
                            quit = true;
                        }
                    });
                    

                    ui.horizontal(|ui|{
                        ui.label(format!("Max base depth: {} plies", self.max_depth));
                        if ui.button("-").clicked(){
                            self.max_depth = self.max_depth.saturating_sub(1);
                            self.recompute();
                        };
                        if ui.button("+").clicked(){
                            self.max_depth = (self.max_depth+1).min(8);
                            self.recompute();
                        }
                    });


                    let mut move_to_apply = None;
                    if let Some((last_depth,results)) = &self.results{
                        egui::ScrollArea::vertical().id_source("engine_evals").show(ui,|ui|{
                            ui.label(format!("Computed at {}-ply depth.",last_depth));
                            results.iter().for_each(|(ply,entry, eval_result)|{
                                if ui.add(egui::Label::new(
                                    format!("{} {} [{}]", eval_result.score, entry, eval_result.nodes)
                                ).sense(egui::Sense::click()))
                                .clicked(){
                                    move_to_apply = Some(*ply);
                                };
                            }); 
                                
                        });
                    };
                    if let Some(ply) = move_to_apply{self.apply_move(ply);}
                });
            });
            egui_macroquad::draw();


            next_frame().await;
            if quit{
                break;
            }
        }

        self.editor.get_state_clone()
    }
}