use egui::{Color32, FontFamily, FontId, Margin, TextStyle};
#[allow(unused_imports)]
use hexstack::engine_debug;
#[allow(unused_imports)]
use hexstack::engine_debug::window_conf as dbg_window_conf;
#[allow(unused_imports)]
use hexstack::game;
#[allow(unused_imports)]
use hexstack::game::window_conf as game_window_conf;
use hexstack::game::{Human,Bot,Gamer};
use macroquad::window::next_frame;


#[macroquad::main(game_window_conf)]
async fn main(){

    let mut choice = None;
    loop {
        egui_macroquad::ui(|egui_ctx|{
            egui::CentralPanel::default()
            .frame(egui::Frame{
                inner_margin : Margin::symmetric(90.0,40.0),
                ..Default::default()
            })
            .show(egui_ctx,|ui|{
                ui.style_mut().text_styles.insert(TextStyle::Button, FontId { size: 32.0, family: FontFamily::Proportional });
                let choices   = [
                    (
                        "Human v. Human", 
                        (None,None)
                    ),

                    (
                        "Human v. 0-ply bot (random moves)", 
                        (None, Some(0))
                    ),

                    (
                        "Human v. 1-ply bot", 
                        (None, Some(1))
                    ),

                    (
                        "Human v. 2-ply bot", 
                        (None,Some(2))
                    ),

                    (
                        "Human v. 4-ply bot", 
                        (None,Some(4))
                    ),

                    (
                        "Human v. 6-ply bot", 
                        (None,Some(6))
                    ),

                    (
                        "6-ply bot v. 6-ply bot", 
                        (Some(6),Some(6))
                    ),
                ];

                choices.into_iter().for_each(|(t,(p1,p2))|{
                    if ui.button(t).clicked(){
                        choice = Some([p1,p2]);
                    }
                });
            });
        });

        egui_macroquad::draw();
        if choice.is_some() {break;}
        next_frame().await
    }

    game::main(choice.unwrap()).await
}