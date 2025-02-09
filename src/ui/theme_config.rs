
use egui::Margin;
use macroquad::prelude::*;
use ::rand::prelude::SliceRandom;

use crate::{assets::{mipmaps::set_cam, PieceSet}, theme::{color_to_color32, egui_ctx_setup, set_theme, BoardPalette, BoardPaletteConfig, BoardTilesModeConfig, BG_COLOR, BOARD_PALETTES, THEME_CONFIG}, Position, Tile};



pub async fn theme_panel(){

    let mut random_position = Position::setup();
    let mut rng = ::rand::thread_rng();

    for _ in 0..16{
        if let Some(&ply) = random_position.valid_moves().choose(&mut rng){
            random_position.apply_move(ply);
        } else{
            break;
        }
    };

    loop{
        let mut done = false;
        let mut pieceset_toset = None;
        clear_background(BG_COLOR);

        set_cam(0.2,vec2(-2.0,0.0));
        Tile::draw_board(false);
        random_position.draw(false, false, false);




        set_default_camera();
        egui_macroquad::ui(|egui_ctx|{
            egui_ctx_setup(egui_ctx);

            egui::SidePanel::left(egui::Id::new("theming"))
            .frame(
                egui::Frame::none()
                .inner_margin(Margin::symmetric(30.0, 60.0))
                
            )
            .resizable(false).show_separator_line(false)
            .show(egui_ctx,|ui|{
                set_theme(ui);

                if ui.button("Back").clicked(){
                    done = true;
                }
                ui.separator();

                ui.heading("Board");


                let mut cfg = THEME_CONFIG.write().unwrap();

                ui.vertical(|ui|{
                    egui::ComboBox::from_id_source("boardmode")
                    .selected_text(cfg.board_mode.tiles.name())
                    .show_ui(ui,|ui|{
                        use BoardTilesModeConfig as B;
                        [B::None, B::Normal, B::WithBorder, B::Outline].into_iter()
                        .for_each(|ch|{
                            ui.selectable_value(&mut cfg.board_mode.tiles, ch, ch.name());
                        });
                    });
                    ui.checkbox(&mut cfg.board_mode.trigrid,"Tri grid");
                });

                match cfg.board_mode.tiles{
                    BoardTilesModeConfig::Normal|BoardTilesModeConfig::WithBorder => {
                        BOARD_PALETTES.iter().for_each(|(n,_)|{
                            ui.horizontal(|ui|{
                                let value = BoardPaletteConfig::Named(n);
                                let pii = value.to_palette();
                                for i in 0..3{
                                    let col = pii.sample(i);
                                    let bgcol = color_to_color32(col);
                                    egui::Frame::none()
                                        .fill(bgcol)
                                        .inner_margin(10.0)
                                        .outer_margin(0.0)
                                        .show(ui,|_ui|{});
                                        
                                }
                                ui.radio_value(
                                    &mut cfg.board_palette, 
                                    value, 
                                    *n);

                                
                                
                            });
                            
                        });

                        let resp_cust = 
                        ui.radio(
                            cfg.board_palette.is_custom(), 
                            
                            "Custom");
                        if resp_cust.clicked(){
                            cfg.board_palette = BoardPaletteConfig::Custom(
                                cfg.board_palette.to_palette()
                            )
                        }
                            
                        
                        match cfg.board_palette{
                            BoardPaletteConfig::Custom(ref mut pal) => {
                                let mut cols = pal.to_egui();

                                cols.iter_mut().enumerate().for_each(|(_,c)|{
                                    ui.color_edit_button_srgb(c);
                                });

                                *pal = BoardPalette::from_egui(cols);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                
                ui.separator();

                
                ui.heading("Pieceset");

                for pset in [
                    PieceSet::Standard,
                    PieceSet::Doodle,
                    // PieceSet::Minimal,
                    // PieceSet::Ornate,
                    PieceSet::Tiles ,
                    // PieceSet::Wooden,
                    PieceSet::Chess
                ]{
                    if ui.radio(cfg.get_pieceset()==pset, pset.name()).clicked(){
                        pieceset_toset = Some(pset)
                    }
                }
                

            });
        });
        egui_macroquad::draw();

        if let Some(pset) = pieceset_toset{
            THEME_CONFIG.write().unwrap().set_pieceset(pset)
            .await

        }
 
        next_frame().await;
        if done{break};
    }
}