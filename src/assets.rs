use std::collections::HashMap;

use coroutines::start_coroutine;

use macroquad::{audio::{load_sound, play_sound, PlaySoundParams, Sound}, prelude::*};
use ::rand::seq::SliceRandom;

pub struct RandomClip{
    clips : Vec<(String,Sound)>
}

impl RandomClip{
    async fn load(paths : Vec<String>) -> Result<Self, FileError>{
        let mut clips = vec![];
        for path in paths{
            clips.push((path.clone(),load_sound(&path).await?));
        };

        Ok(RandomClip{clips})
    }

    pub fn play(&self){
        let (_,clip) = self.clips.choose(&mut ::rand::thread_rng()).unwrap();
        play_sound(*clip, PlaySoundParams{
            looped : false,
            volume : 0.5
        });
        // println!("{}",name);
        
    }
}

const FONT_PATH : &'static str = "gfx/Lexend-Light.ttf";



use crate::{theme, Player};

pub struct Assets{
    pub pieces : Texture2D,
    pub btn_takeback : Texture2D,
    pub btn_exit : Texture2D,
    pub btn_lines : Texture2D,
    pub btn_letters : Texture2D,
    pub btn_rules : Texture2D,
    pub avatars : Texture2D,
    pub font : Font,
    pub font_bytes : Vec<u8>,
    pub title : Texture2D,

    pub piece_slide : RandomClip,

    pub mate : Sound,
    pub capture:Sound,

    pub diagrams : HashMap<&'static str, Texture2D>
}

impl Assets{

    pub async fn loading_screen() -> Self{
        let load_co = start_coroutine(Assets::load());
        let mut time : f32 = 0.0;

        
        loop {
            clear_background(theme::BG_COLOR);

            if time > 0.2{
                if let Some(result) = load_co.retrieve(){
                    match result{
                        Ok(assets) => {return assets;},

                        Err(error) => loop {
                            clear_background(theme::BG_COLOR);
                            set_default_camera();
                            
                            egui_macroquad::ui(|egui_ctx|{
                                egui_ctx.set_pixels_per_point(screen_height() / 720.0);
                                egui_ctx.set_visuals(egui::Visuals::light());
                                egui::CentralPanel::default()
                                .show(egui_ctx, |ui|{
                                    ui.label(
                                        &format!("Error loading assets: {:?}", error)
                                    )
                                });
                            });
                            egui_macroquad::draw();
                            next_frame().await
                        }
                    }
                }
            }

            set_camera(&Camera2D{
                target: vec2(0.0,0.0),
                zoom : 0.05*vec2(screen_height()/screen_width(),1.0),
                ..Default::default()
            });

            let th = time * 2.0;
            
            const N : usize = 7;

            (0..N).for_each(|u|{
                let pang = (u as f32)*std::f32::consts::PI * 2.0 / (N as f32);
                let ang = th + pang;

                let s = 0.75+0.25*ang.sin();

                let mut col = Color::from_hex(0x111111);
                col.a = 0.5*(ang.cos()+1.0);

                let cang = pang * 1.0;
                let center = vec2(cang.cos(),cang.sin())*4.0;
                draw_circle(center.x,center.y, s * 1.0,col);
            });
            

            time += get_frame_time();
            next_frame().await
        }

    }


    pub async fn load()->Result<Assets,FileError> {

        // unwrap font error because I don't wanna cast
        // and I can't update mquad :(
        let font = load_ttf_font(FONT_PATH).await.unwrap();
        font.set_filter(FilterMode::Linear);

        let font_bytes = macroquad::file::load_file(&FONT_PATH)
            .await?;

        let mut diagrams = HashMap::new();
        for name in [
            "board",
            "mov_flat_white","mov_flat_black",
            "mov_arm_white","mov_arm_black",
            "mov_blind_white","mov_blind_black",
            "mov_star_white","mov_star_black",
            "attack_pre","attack_post",
            "stack_pre","stack_post"

        ]{
            diagrams.insert(name, 
                load_texture(&format!("diags/{}.png",name)).await?
            );
        }
    


        Ok(Assets{
            pieces : load_texture("gfx/pieces_sm.png").await?,
            btn_takeback : load_texture("gfx/btn_takeback.png").await?,
            btn_exit : load_texture("gfx/btn_exit.png").await?,
            btn_lines : load_texture("gfx/btn_lines.png").await?,
            btn_letters : load_texture("gfx/btn_letters.png").await?,
            btn_rules : load_texture("gfx/btn_rules.png").await?,
            avatars :  load_texture("gfx/avatars.png").await?,
            font ,
            font_bytes,
            title : load_texture("gfx/title.png").await?,

            piece_slide : RandomClip::load([1,3,4,5,7,8].into_iter().map(|n|format!("audio/slide{}.ogg",n)).collect()).await?,

            mate : load_sound("audio/mate.ogg").await?,
            capture : load_sound("audio/bopp.ogg").await?,
            diagrams,
        })
    }

    pub fn get_avatar(&self, player : Player, avatar_offset : usize) -> (Texture2D,Rect){
        let avatar_src = Rect::new(
            (128 * avatar_offset) as f32,
            (128 * match player {
                Player::Black => 1,
                Player::White => 0
            }) as f32,
            128.0,128.0
        );


        (self.avatars,avatar_src)
    }



    
    
}
