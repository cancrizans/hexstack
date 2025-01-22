
use egui::Margin;
use macroquad::prelude::*;
use ::rand::prelude::SliceRandom;

use crate::{assets::{mipmaps::set_cam, PieceSet}, theme::{color_to_color32, egui_ctx_setup, set_theme, BoardPalette, BoardPaletteConfig, BG_COLOR, BOARD_PALETTES, THEME_CONFIG}, Position, Tile};



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
                
                ui.separator();

                if false{
                    ui.heading("Pieceset");

                    for pset in [
                        PieceSet::Standard,
                        PieceSet::Minimal,
                        PieceSet::Ornate    
                    ]{
                        if ui.radio(cfg.get_pieceset()==pset, pset.name()).clicked(){
                            cfg.set_pieceset(pset);
                        }
                    }
                }

            });
        });
        egui_macroquad::draw();

        next_frame().await;
        if done{break};
    }
}