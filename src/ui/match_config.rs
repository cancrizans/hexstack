use egui::{FontFamily, FontId, Margin, TextStyle};

use super::editor::PositionEditor;



use crate::{gameplay::{GamerSpec, MatchConfig}, theme::{self, set_theme}, Player, Tile};
use macroquad::window::{clear_background, next_frame, screen_height};

use crate::assets::Assets;
use macroquad::prelude::*;

pub async fn match_config_ui(assets : &Assets, last_match_config : Option<MatchConfig>) -> MatchConfig{
    let choices = [
        GamerSpec::Human,
        GamerSpec::Gibberish,
        GamerSpec::Noob,
        GamerSpec::Decent,
        GamerSpec::Sharp,
        GamerSpec::Tough,
        GamerSpec::Beastly
    ];


    let mut match_config = last_match_config.unwrap_or(MatchConfig{
        gamers : [GamerSpec::Human, GamerSpec::Noob],
        gamer_one_color : None,
        allow_takeback : true,
        starting_position : None
    });


    let mut break_out = None;

    let mut time : f32 = 0.0;


    loop {
        clear_background(theme::BG_COLOR);

        {
            let time_long = time * 0.03 + 0.05;
            set_camera(&Camera2D{
                target : vec2(5.0*time_long.cos(),2.0),
                rotation : 40.0 + 10.0 * (time*0.0342).sin(),
                zoom: 0.3*vec2(screen_height()/screen_width(), 1.0),
                ..Default::default()
            });
            Tile::draw_board(false);
        }

        set_default_camera();
        let mut panel_col = theme::BG_COLOR;
        panel_col.a = 0.6;
        // let panel_col = egui::Color32::from_rgba_premultiplied(bg_col32.r(), bg_col32.g(), bg_col32.b(), 30);
        draw_rectangle(screen_width()*0.55, 0.0, screen_width()*0.5, screen_height(), panel_col);

        egui_macroquad::ui(|egui_ctx|{

            egui_ctx.set_pixels_per_point(screen_height() / 720.0);

            egui_ctx.set_visuals(egui::Visuals::light());

            
            

            egui::SidePanel::right(egui::Id::new("match_ui"))
            .frame(
                egui::Frame::none()
                .inner_margin(Margin::symmetric(75.0,0.0))
                // .fill(panel_col)

            )
            .resizable(false).show_separator_line(false)
            .show(egui_ctx,|ui|{
                set_theme(ui);
                
                // let layout = Layout{
                //     main_dir : Direction::TopDown,
                //     ..Default::default()
                // };

                ui.add_space(250.0);

                
                ui.horizontal(|ui|{
                    ui.set_min_width(400.0);
                    ui.set_max_width(400.0);
                    match_config.gamers.iter_mut().enumerate().for_each(|(i,g)|{
                    
                        ui.vertical(|ui|{
                            ui.set_min_width(200.0);
                            ui.set_max_width(200.0);
                    
                            // ui.heading(format!("Player {}",i+1));

                            
        
                            
                            egui::ComboBox::from_id_source(format!("player{}",i+1))
                            .selected_text(format!("{}",g.name()))
                            .show_ui(ui,|ui|{
                                choices.iter().for_each(|choice|{
                                    ui.selectable_value(g, *choice, choice.name());
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
                });

                ui.add_space(30.0);

                ui.separator();

                ui.add_space(30.0);

                ui.horizontal(|ui|{
                    ui.checkbox(&mut match_config.allow_takeback, "Allow undo");

                    ui.add_space(10.0);
                    match match_config.starting_position{
                        Some(..) => if ui.button("Default starting position").clicked(){
                            match_config.starting_position = None
                        },
                        None => if ui.button("Edit starting position").clicked(){
                            match_config.starting_position = Some(PositionEditor::setup())
                        }
                    }
                });
                

                ui.add_space(30.0);
                
                ui.separator();

                

                ui.add_space(40.0);
                ui.horizontal(|ui|{
                    ui.style_mut().text_styles.insert(
                        TextStyle::Button, 
                        FontId { 
                            size: 30.0, 
                            family: FontFamily::Proportional 
                        });
                    let start_button = ui.add_sized(
                        [200.0,50.0],
                        egui::Button::new("Start Match")
                    );
                    if start_button.clicked(){
                        break_out = Some(());
                    }
                })
                
            
                    


            });
        });

        egui_macroquad::draw();
        if let Some(()) = break_out{
            break;
        }

        let match_ui_cam = &Camera2D{
            target:vec2(0.0,0.0),
            zoom : vec2(screen_height()/screen_width(),-1.0),
            ..Default::default()
        };
        set_camera(match_ui_cam);

        for pid in [0,1]{
            let x = 0.55 + (pid as f32)*0.6;
            let y = -0.55;

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

        if let Some(ref mut editor) = match_config.starting_position{
            let editor_cam = &Camera2D{
                target : vec2(6.0,-1.5),
                zoom : vec2(screen_height()/screen_width(), -1.0) * 0.12,
                ..Default::default()
            };
            editor.process(editor_cam, assets);
        } else {
            let title_dest_size = vec2(assets.title.width()/assets.title.height(),1.0) * 1.2;
            draw_texture_ex(
                assets.title, 
                -1.64, 
                -1.0, 
                WHITE, DrawTextureParams{
                    dest_size : Some(title_dest_size),
                    ..Default::default()
                });
        }

        set_camera(match_ui_cam);
        let (font_size, font_scale, font_scale_aspect) = camera_font_scale(0.08);
        draw_text_ex(&format!("version {}",env!("CARGO_PKG_VERSION")),
            -1.7,0.9,TextParams { 
                font: assets.font, font_size, font_scale, font_scale_aspect, color:Color::from_hex(0x111111),
                ..Default::default()
            },
        );

        time += get_frame_time();
        next_frame().await
    };

    match_config
}