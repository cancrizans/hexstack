use crate::{assets::{Assets, ASSETS}, theme::{self, set_theme}};
use egui::Margin;
use macroquad::prelude::*;

struct Page{
    text : &'static str,
    diags : &'static [&'static str],
}

const PAGES : &[Page] = &[
    Page{
        text : "White goes first and plays from right to left. \
            Black plays from left to right. \
            The rightmost tile is White's house. \
            The leftmost tile is Black's house.",

        diags : &[
            "board"
        ]
    },

    Page{
        text : "Flats move like this: one tile forward, or \
        one forward to the side.",
        diags : &[
            "mov_flat_white",
            "mov_flat_black"
        ]
    },

    Page{
        text : "Arms move like this: either two tiles forward, \
        jumping over obstacles, or one tile backwards, or \
        one tile backwards to the side. \
        They cannot move only one tile forward.",
        diags : &["mov_arm_white","mov_arm_black"]
    },

    Page{
        text : "Blinds move like this: either two tiles forward \
        to the side, jumping over obstacles, or one tile \
        backwards to the side, or two tiles backwards to the \
        side, again jumping over obstacles. \
        They cannot move straight forward.",
        diags : &["mov_blind_white","mov_blind_black",]
    },
    Page{
        text : "The White Star is always on a dark tile, and \
        can jump diagonally to any of the closest dark tiles. \
        The Black Star is always on a light tile, and can jump \
        diagonally to any of the closest light tiles.",
        diags : &["mov_star_white","mov_star_black",]
    },

    Page{
        text : "After a player moves, they attack their opponent automatically. \
            All of their pieces attack, even if they weren't moved. \
            Pieces attack in the same way they move. If at least two \
            attacking pieces attack a defending piece, it gets captured. \
            Only the player that just moved attacks.
        ",
        diags : &["attack_pre","attack_post"]

    },
    Page{
        text : "Tall-pieces (Arm, Blind, or Star) can move onto a flat \
            of the same color to form a stack. The flat underneath is \
            inactive: it can neither move nor attack. \
        ",
        diags : &["stack_pre","stack_post"]

    },

    Page{
        text : "If a player has no legal moves left, they lose by stalemate. \
        If a player's flat reaches the opponent's house tile, they win by house \
        capture.",
        diags : &[]
    },
];

pub async fn read_rulesheet(){
    let mut page_num = 0;

    loop{
        let mut done = false;
        clear_background(theme::BG_COLOR);

        let page = &PAGES[page_num];

        egui_macroquad::ui(|egui_ctx|{
            egui_ctx.set_pixels_per_point(screen_height() / 720.0);
            egui_ctx.set_visuals(egui::Visuals::light());
            egui::SidePanel::right(egui::Id::new("rulesheet"))
            .frame(
                egui::Frame::none()
                .inner_margin(Margin::symmetric(100.0,100.0))
                
                // .fill(panel_col)

            )
            .resizable(false).show_separator_line(false).exact_width(700.0)
            .show(egui_ctx,|ui|{
                set_theme(ui);

                ui.heading("tokonoma");
                ui.label(format!("Rules cheatsheet ({}/{})", page_num+1, PAGES.len()));

                
                

                ui.horizontal(|ui|{
                    if ui.add_enabled(
                        page_num > 0, 
                        egui::Button::new("Previous")
                        
                    ).clicked() {page_num -= 1;}

                    if ui.add_enabled(
                        page_num < PAGES.len() - 1, 
                        egui::Button::new("Next")
                        
                    ).clicked() {page_num += 1;}
                    
                    
                    if ui.button("Return to Game").clicked(){
                        done = true;
                    };
                });
                ui.add_space(20.0);

                ui.separator();
                ui.add_space(20.0);

                ui.add_sized(
                    [400.0,400.0],
                    egui::Label::new(page.text)
                    .wrap(true)   
                );
            });
        });
        egui_macroquad::draw();

        set_camera(&Camera2D{
            zoom: 1.0*vec2(screen_height()/screen_width(), -1.0),
            target : Vec2::ZERO,
            ..Default::default()
        });

        let n_diags = page.diags.len();
        let assets = ASSETS.get().unwrap();
        for (i,diag_name) in page.diags.iter().enumerate(){
            let diag = *assets.diagrams.get(diag_name).unwrap();

            let dest = vec2(diag.width()/diag.height(),1.0) * 0.85;

            let center = vec2(-0.8,1.0 * ((i as f32) -0.5*(n_diags as f32 - 1.0)));

            let offset = center - dest*0.5;
            

            draw_texture_ex(
                diag, 
                offset.x,offset.y, 
                WHITE, 
                DrawTextureParams{
                    dest_size : Some(dest),
                    ..Default::default()
                }
            );
        }


        if done {break};

        next_frame().await
    }
}