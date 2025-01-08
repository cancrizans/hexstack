use egui::{Align, Color32, Direction, FontFamily, FontId, Layout, Margin, TextStyle};
#[allow(unused_imports)]
use hexstack::engine_debug;
#[allow(unused_imports)]
use hexstack::engine_debug::window_conf as dbg_window_conf;
#[allow(unused_imports)]
use hexstack::game;
#[allow(unused_imports)]
use hexstack::game::window_conf as game_window_conf;
use hexstack::game::{GamerSpec, MatchConfig};
use hexstack::theme::{color_to_color32, set_theme};
use hexstack::{theme, Player};
use macroquad::window::{clear_background, next_frame, screen_height};

use hexstack::assets::Assets;
use macroquad::prelude::*;


async fn match_ui(assets : &Assets) -> MatchConfig{
    let choices = [
        GamerSpec::Human,
        GamerSpec::Gibberish,
        GamerSpec::Noob,
        GamerSpec::Decent,
        GamerSpec::Sharp,
        GamerSpec::Tough,
        GamerSpec::Beastly
    ];


    let mut match_config = MatchConfig{
        gamers : [GamerSpec::Human, GamerSpec::Noob],
        gamer_one_color : None,
        allow_takeback : true
    };


    let mut break_out = None;

    let mut time : f32 = 0.0;


    loop {
        clear_background(theme::BG_COLOR);

        egui_macroquad::ui(|egui_ctx|{

            egui_ctx.set_pixels_per_point(screen_height() / 720.0);

            egui_ctx.set_visuals(egui::Visuals::light());

            

            egui::CentralPanel::default()
            .frame(egui::Frame{
                fill : color_to_color32(theme::BG_COLOR),
                inner_margin : Margin::symmetric(160.0,80.0),
                ..Default::default()
            })
            .show(egui_ctx,|ui|{
                set_theme(ui);
                
                let layout = Layout{
                    main_dir : Direction::LeftToRight,
                    main_align : Align::Center,
                    ..Default::default()
                };

                ui.with_layout(layout,|ui|{
                    match_config.gamers.iter_mut().enumerate().for_each(|(i,g)|{
                        ui.vertical(|ui|{
                            ui.set_min_width(200.0);
                    
                            // ui.heading(format!("Player {}",i+1));

                            ui.add_space(200.0);
        
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
                                    ui.radio_value(&mut match_config.gamer_one_color, None, "Random");
                                    ui.radio_value(&mut match_config.gamer_one_color, Some(Player::White), "White");
                                    ui.radio_value(&mut match_config.gamer_one_color, Some(Player::Black), "Black");
                                },
                                1 => {
                                    ui.label(if let Some(p1_col) = match_config.gamer_one_color {
                                        match p1_col.flip() {
                                            Player::Black => "Black",
                                            Player::White => "White"
                                        }
                                    } else {"Random"});
                                },
                                _ => unreachable!("Player past index 1")
                            }
                        });
    
                    });


                    ui.separator();

                    ui.add_space(140.0);

                    ui.vertical(|ui|{
                        ui.add_space(300.0);
                        ui.checkbox(&mut match_config.allow_takeback, "Allow undo");

                        ui.add_space(30.0);
                        ui.horizontal(|ui|{
                            ui.style_mut().text_styles.insert(
                                TextStyle::Button, 
                                FontId { 
                                    size: 30.0, 
                                    family: FontFamily::Proportional 
                                });
                            if ui.button("Start Match").clicked(){
                                break_out = Some(());
                            }
                        })
                        
                    });
                    

                });
                

            });
        });

        egui_macroquad::draw();
        if let Some(()) = break_out{
            break;
        }

        set_camera(&Camera2D{
            target:vec2(0.0,0.0),
            zoom : vec2(screen_height()/screen_width(),-1.0),
            ..Default::default()
        });

        for pid in [0,1]{
            let x = -1.2 + (pid as f32)*0.6;
            let y = -0.5;

            let size = 0.4*Vec2::ONE;
            let mut base_color = match_config.gamer_one_color.unwrap_or(
                Player::White
            );
            if pid > 0 {base_color = base_color.flip()};

            let av_offset = if match_config.gamers[pid] == GamerSpec::Human {0} else {1};

            let (avatar_tex,src) = assets.get_avatar(
                base_color, 
                av_offset);

            if let Some(..) = match_config.gamer_one_color{
                draw_texture_ex(avatar_tex, 
                    x-size.x*0.5, 
                    y-size.y*0.5, 
                    WHITE, 
                    DrawTextureParams{
                        dest_size : Some(size),
                        source : Some(src),
                        ..Default::default()
                    });
            } else {
                let mut half_sz = size;
                half_sz.x *= 0.5;
                let mut left_src = src;
                left_src.w *= 0.5;
                draw_texture_ex(avatar_tex, 
                    x-size.x*0.5, 
                    y-size.y*0.5, 
                    WHITE, 
                    DrawTextureParams{
                        dest_size : Some(half_sz),
                        source : Some(left_src),
                        ..Default::default()
                    });

                let (_,mut right_src) = assets.get_avatar(
                    base_color.flip(), 
                    av_offset);
                right_src.w *= 0.5;
                right_src.x += right_src.w;

                draw_texture_ex(avatar_tex, 
                    x, 
                    y-size.y*0.5, 
                    WHITE, 
                    DrawTextureParams{
                        dest_size : Some(half_sz),
                        source : Some(right_src),
                        ..Default::default()
                    });
            }
        }

        time += get_frame_time();
        next_frame().await
    };

    match_config
}

#[macroquad::main(game_window_conf)]
async fn main(){
    


    let assets : Assets = Assets::loading_screen().await;

    egui_macroquad::cfg(|egui_ctx |{
        theme::set_fonts(egui_ctx, &assets);
    });

    loop{
        let match_config = match_ui(&assets).await;    
        game::main(&assets,match_config).await
    }
}