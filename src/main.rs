use std::fmt::format;

use egui::{Color32, FontFamily, FontId, Margin, TextStyle};
#[allow(unused_imports)]
use hexstack::engine_debug;
#[allow(unused_imports)]
use hexstack::engine_debug::window_conf as dbg_window_conf;
#[allow(unused_imports)]
use hexstack::game;
#[allow(unused_imports)]
use hexstack::game::window_conf as game_window_conf;
use hexstack::game::GamerSpec;
use hexstack::Player;
use macroquad::window::next_frame;

use hexstack::assets::Assets;

async fn match_ui() -> ([GamerSpec;2],Option<Player>){
    let choices = [
        GamerSpec::Human,
        GamerSpec::Gibberish,
        GamerSpec::Noob,
        GamerSpec::Decent,
        GamerSpec::Sharp,
        GamerSpec::Tough,
        GamerSpec::Beastly
    ];

    let mut gamers = [GamerSpec::Human, GamerSpec::Noob];
    let mut break_out = None;

    let mut p1_color = None;



    loop {
        egui_macroquad::ui(|egui_ctx|{
            egui_ctx.set_visuals(egui::Visuals::light());

            egui::CentralPanel::default()
            .frame(egui::Frame{
                fill : Color32::WHITE,
                inner_margin : Margin::symmetric(160.0,80.0),
                ..Default::default()
            })
            .show(egui_ctx,|ui|{
                ui.style_mut().text_styles.insert(
                    TextStyle::Heading, 
                    FontId { 
                        size: 32.0, 
                        family: FontFamily::Proportional 
                    });
                ui.style_mut().text_styles.insert(
                    TextStyle::Body, 
                    FontId { 
                        size: 16.0, 
                        family: FontFamily::Proportional 
                    });
                ui.style_mut().text_styles.insert(
                    TextStyle::Button, 
                    FontId { 
                        size: 16.0, 
                        family: FontFamily::Proportional 
                    });
                
                ui.horizontal(|ui|{
                    gamers.iter_mut().enumerate().for_each(|(i,g)|{
                        ui.vertical(|ui|{
                            ui.set_min_width(200.0);
                    
                            ui.heading(format!("Player {}",i+1));

                            ui.add_space(20.0);
        
                            ui.vertical(|ui|{
                                choices.iter().for_each(|c|{
                                    ui.radio_value(g, *c, c.name());
        
                                });
                            });
        
                            ui.label(g.description());

                            ui.add_space(20.0);

                            ui.label("Plays as:");

                            match i{
                                0 => {
                                    ui.radio_value(&mut p1_color, None, "Random");
                                    ui.radio_value(&mut p1_color, Some(Player::White), "White");
                                    ui.radio_value(&mut p1_color, Some(Player::Black), "Black");
                                },
                                1 => {
                                    ui.label(if let Some(p1_col) = p1_color {
                                        match p1_col.flip() {
                                            Player::Black => "Black",
                                            Player::White => "White"
                                        }
                                    } else {"Random"});
                                },
                                _ => unreachable!()
                            }
                        });
    
                    });

                    ui.separator();

                    if ui.button("Start Game").clicked(){
                        break_out = Some(());
                    }

                });
                

            });
        });

        egui_macroquad::draw();
        if let Some(()) = break_out{
            break;
        }
        next_frame().await
    };

    (gamers,p1_color)
}

#[macroquad::main(game_window_conf)]
async fn main(){

    let assets : Assets = Assets::load().await;

    loop{
        let (gamers,p1_color) = match_ui().await;    
        game::main(&assets,gamers, p1_color).await
    }
}