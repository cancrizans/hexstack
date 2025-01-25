use egui::{FontFamily, FontId, Margin, TextStyle};

use super::{editor::PositionEditor, engine_eval::EngineEvalUI, theme_config};
#[cfg(feature="networking")]
use super::net_client;



use crate::{ assets::{get_assets_unchecked, mipmaps::set_cam}, gameplay::{GamerSpec, MatchConfig}, theme::{self, egui_ctx_setup, set_theme}, Player, Tile};
use macroquad::window::{clear_background, next_frame, screen_height};

use macroquad::prelude::*;

enum Transition{
    Closed,
    Open
}
impl Transition{
    fn closed() -> Self{
        Transition::Closed
    }
    fn open(&mut self){
        *self = Transition::Open
    }
    fn pop(&mut self)-> bool{
        let was_open = match self{
            Transition::Open => true,
            Transition::Closed => false
        };
        *self = Transition::Closed;
        was_open
    }
}

pub async fn match_config_ui(last_match_config : Option<MatchConfig>) -> MatchConfig{
    let choices : Vec<GamerSpec> = [
        GamerSpec::Human,
        GamerSpec::Gibberish,
        GamerSpec::Noob,
        GamerSpec::Decent,
        GamerSpec::Sharp,
        GamerSpec::Tough,
        GamerSpec::GrandMaster
        
    ].into_iter().chain((5..=8).map(|depth|GamerSpec::Perfect { depth }))
    .collect();


    let mut match_config = last_match_config.unwrap_or(MatchConfig{
        gamers : [GamerSpec::Human, GamerSpec::Noob],
        gamer_one_color : None,
        allow_takeback : true,
        starting_position : None
    });


    let mut break_out = None;

    let mut time : f32 = 0.0;

    let mut open_engine_eval_ui = Transition::closed();
    let mut open_theming_ui = Transition::closed();
    #[cfg(feature="networking")]
    let mut open_net_ui = Transition::closed();
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
            egui_ctx_setup(egui_ctx);
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
                    match_config.gamers.iter_mut().enumerate().for_each(|(gamer_idx,gamer_spec)|{
                    
                        ui.vertical(|ui|{
                            ui.set_min_width(200.0);
                            ui.set_max_width(200.0);
                    
                            // ui.heading(format!("Player {}",i+1));

                            
        
                            
                            egui::ComboBox::from_id_source(format!("player{}",gamer_idx+1))
                            .selected_text(format!("{}",gamer_spec.name()))
                            .width(150.0)
                            .show_ui(ui,|ui|{
                                // ui.spacing_mut().item_spacing.y = 30.0;
                                choices.iter().for_each(|&gamer_option|{
                                    let lbl = egui::SelectableLabel::new(*gamer_spec == gamer_option, 
                                        egui::RichText::new(gamer_option.name())
                                        .size(18.0)
                                    );

                                    let mut resp = ui.add(lbl);
                                    if resp.clicked() && *gamer_spec!=gamer_option{
                                        *gamer_spec = gamer_option;
                                        resp.mark_changed();
                                    };

                                    
                                });
                            });

        
                            ui.label(gamer_spec.description());

                            ui.add_space(20.0);

                            ui.label("Plays as:");
                            
                            fn fpc_to_str(v : Option<Player>) -> &'static str{
                                match v{
                                    None => "Random",
                                    Some(Player::White) => "White",
                                    Some(Player::Black) => "Black"
                                }
                            }

                            match gamer_idx{
                                0 => {
                                    egui::ComboBox::from_id_source("playsas")
                                    .selected_text(fpc_to_str(match_config.gamer_one_color))
                                    .width(150.0)
                                    .show_ui(ui,|ui|{
                                        // ui.spacing_mut().item_spacing.y = 30.0;
                                        for sval in [None,Some(Player::White),Some(Player::Black)]{
                                            ui.selectable_value(
                                                &mut match_config.gamer_one_color, 
                                                sval, 
                                                egui::RichText::new(fpc_to_str(sval))
                                            );
                                        };
                                    });
                                },
                                1 => {
                                    ui.label(fpc_to_str(match_config.gamer_one_color.map(|p|p.flip())));
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
                    

                    ui.add_space(10.0);
                    match match_config.starting_position{
                        Some(..) => if ui.button("Default starting position").clicked(){
                            match_config.starting_position = None
                        },
                        None => if ui.button("Edit starting position").clicked(){
                            match_config.starting_position = Some(PositionEditor::setup())
                        }
                    };
                    if ui.button("Engine evaluation").clicked(){
                        open_engine_eval_ui.open();
                    };

                    
                });

                ui.horizontal(|ui|{
                    ui.add_enabled(
                        (match_config.gamers[0] == GamerSpec::Human) | (match_config.gamers[1] == GamerSpec::Human), 
                        egui::Checkbox::new(&mut match_config.allow_takeback, "Allow taking back moves")
                    );
                });

                ui.separator();

                ui.add_space(15.0);
                
                if ui.button("Themes and colors...").clicked(){
                    open_theming_ui.open();
                };
                
                

                ui.add_space(15.0);
                
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
                    };

                    #[cfg(feature="networking")]
                    if ui.add_sized(
                        [200.0,50.0],
                        egui::Button::new("Online")
                    ).clicked(){
                        open_net_ui.open()
                    }
                })
                
            
                    


            });
        });


        egui_macroquad::draw();
        
        if open_engine_eval_ui.pop(){
            let editor = match_config.starting_position.unwrap_or(PositionEditor::setup());
            let evaled_state = EngineEvalUI::new(editor).run().await;
            match_config.starting_position = Some(evaled_state);
            
        }

        if open_theming_ui.pop(){
            theme_config::theme_panel().await;
            
        }

        #[cfg(feature="networking")]
        if open_net_ui.pop(){
            net_client::net_client_ui().await;
        }

        
        if let Some(()) = break_out{
            break;
        }

        // let match_ui_cam = &Camera2D{
        //     target:vec2(0.0,0.0),
        //     zoom : vec2(screen_height()/screen_width(),-1.0),
        //     ..Default::default()
        // };
        let (match_ui_zoom, match_ui_target) = (1.0,Vec2::ZERO);
        set_cam(match_ui_zoom,match_ui_target);

        let assets = get_assets_unchecked();

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
                avatar_tex.draw( 
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
                avatar_tex.draw(
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

                avatar_tex.draw( 
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
            editor.process(editor_cam);
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

        set_cam(match_ui_zoom,match_ui_target);
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