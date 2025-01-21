use std::future::Future;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, sync::OnceLock};


use macroquad::experimental::coroutines::{Coroutine,start_coroutine};


use macroquad::{audio::{load_sound, play_sound, PlaySoundParams, Sound}, prelude::*};
use ::rand::seq::SliceRandom;

use lazy_static::lazy_static;

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

#[derive(Clone, Copy,PartialEq, Eq)]
pub enum PieceSet{
    Standard,
    Minimal,
    Ornate
}
impl PieceSet{
    pub fn name(&self)->&'static str{
        match self{
            Self::Standard => "Standard",
            Self::Minimal => "Minimal",
            Self::Ornate => "Ornate"
        }
    }
}

#[derive(Clone)]
pub enum CompositionMode{
    Precomposed,
    ComposeOnFlat
}

#[derive(Clone)]
pub struct PieceSetAsset{
    pub tex : Texture2D,
    pub base_scale : f32,
    pub composition_mode : CompositionMode
}

// for reasons unknown to me, impl Drop
// with tex.delete() creates a black texture? Strange.
// We should manual delete.
// Might be related to mquad bugs, hence .delete() was
// removed in later versions (which we can't use).

// impl Drop for PieceSetAsset{
//     fn drop(&mut self) {
//         self.tex.delete();
//     }
// }

impl PieceSetAsset{
    async fn load(spec : PieceSet) -> Result<PieceSetAsset,FileError>{
        let set = match spec{
            PieceSet::Standard => PieceSetAsset{
                tex : load_texture("gfx/pieces_sm.png").await?,
                base_scale : 1.7,
                composition_mode : CompositionMode::Precomposed,
            },
            PieceSet::Minimal => PieceSetAsset{
                tex : load_texture("gfx/pieces_minimal.png").await?,
                base_scale : 1.3,
                composition_mode : CompositionMode::Precomposed,
            },

            PieceSet::Ornate => PieceSetAsset{
                tex : load_texture("gfx/pieces_ornate.png").await?,
                base_scale : 2.2,
                composition_mode : CompositionMode::ComposeOnFlat
            }
        };
        Ok(set)
    }
}

use crate::theme::egui_ctx_setup;
use crate::{theme, Player};

pub struct Assets{
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
                                egui_ctx_setup(egui_ctx);
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
        // Pieceset is not stored in here,
        // but we still load the default so it's
        // slotted in the loading screen.
        set_pieceset(PieceSet::Standard).await?;


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
        let tile_size = self.avatars.width() * 0.5;
        let avatar_src = Rect::new(
            tile_size *(avatar_offset as f32),
            tile_size * (match player {
                Player::Black => 1,
                Player::White => 0
            }) as f32,
            tile_size,tile_size
        );


        (self.avatars,avatar_src)
    }



    
    
}


lazy_static! {
    static ref ASSETS : OnceLock<Assets> = OnceLock::new();

    static ref PIECESET : Arc<RwLock<Option<PieceSetAsset>>> = Arc::new(RwLock::new(None));
}

pub async fn load_assets(){
    let assets : Assets = Assets::loading_screen().await;
    ASSETS.get_or_init(||assets);
}

pub fn get_pieceset() -> Option<PieceSetAsset>{
    PIECESET.read().unwrap().clone()
}
pub fn get_pieceset_unchecked() -> PieceSetAsset{
    get_pieceset().unwrap()
}
pub async fn set_pieceset(set : PieceSet) -> Result<(),FileError>{
    // if let Some(..) = get_pieceset(){
    //     println!("Invalidating old pieceset tex.");
    //     PIECESET.write().unwrap().as_mut().unwrap().tex.delete();
    // }

    let psa = PieceSetAsset::load(set).await?;
    println!("Loaded standard pieceset.");
    *PIECESET.write().unwrap() = Some(psa);
    Ok(())
}

pub fn get_assets_unchecked()->&'static Assets{
    ASSETS.get().expect("Assets not loaded.")
}

pub trait Content : Clone + Copy{
    fn load_unchecked(path : &str) -> impl Future<Output = Self> + Send;
}
impl Content for Texture2D{
    fn load_unchecked(path : &str) -> impl Future<Output = Self> + Send {
        async {
            load_texture(path).await.unwrap()
        }
    }
}

pub struct Asset<T> where T:Content + 'static + Debug{
    resource : Coroutine<T>,
    path : &'static str
}

#[allow(dead_code)]
impl<T> Asset<T> where T:Content + Debug{
    fn load(path : &'static str) -> Asset<T>{
        Asset{
            resource : start_coroutine(T::load_unchecked(path)),
            path
        }
    }

    pub fn get(&self) -> Option<T>{
        self.resource.retrieve()
    }

    pub fn get_unchecked(&self) -> T{
        if let Some(value) = self.get(){
            value
        } else {
            panic!("unchecked access to not loaded asset {}",self.path)
        }
    }
}