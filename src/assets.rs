use coroutines::start_coroutine;

use macroquad::{audio::{load_sound, play_sound, PlaySoundParams, Sound}, prelude::*};
use ::rand::seq::SliceRandom;

pub struct RandomClip{
    clips : Vec<(String,Sound)>
}

impl RandomClip{
    async fn load(paths : Vec<String>) -> Self{
        let mut clips = vec![];
        for path in paths{
            clips.push((path.clone(),load_sound(&path).await.unwrap()));
        };

        RandomClip{clips}
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
    pub avatars : Texture2D,
    pub font : Font,
    pub font_bytes : Vec<u8>,
    pub title : Texture2D,

    pub piece_slide : RandomClip,

    pub mate : Sound,
    pub capture:Sound,
}

impl Assets{

    pub async fn loading_screen() -> Self{
        let load_co = start_coroutine(Assets::load());
        let mut time : f32 = 0.0;

        
        loop {
            clear_background(theme::BG_COLOR);

            if time > 0.2{
                if let Some(assets) = load_co.retrieve(){
                    return assets;
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


    pub async fn load()->Self{
        let font = load_ttf_font(FONT_PATH).await.unwrap();
        font.set_filter(FilterMode::Linear);

        let font_bytes = macroquad::file::load_file(&FONT_PATH)
            .await.unwrap();

        Assets{
            pieces : load_texture("gfx/pieces_sm.png").await.unwrap(),
            btn_takeback : load_texture("gfx/btn_takeback.png").await.unwrap(),
            btn_exit : load_texture("gfx/btn_exit.png").await.unwrap(),
            btn_lines : load_texture("gfx/btn_lines.png").await.unwrap(),
            btn_letters : load_texture("gfx/btn_letters.png").await.unwrap(),
            avatars :  load_texture("gfx/avatars.png").await.unwrap(),
            font ,
            font_bytes,
            title : load_texture("gfx/title.png").await.unwrap(),

            piece_slide : RandomClip::load([1,3,4,5,7,8].into_iter().map(|n|format!("audio/slide{}.ogg",n)).collect()).await,

            mate : load_sound("audio/mate.ogg").await.unwrap(),
            capture : load_sound("audio/bopp.ogg").await.unwrap(),
        }
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