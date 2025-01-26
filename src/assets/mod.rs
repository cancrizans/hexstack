use std::future::Future;
use std::fmt::{Debug, Display};
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, sync::OnceLock};
pub mod mipmaps;

use macroquad::experimental::coroutines::{Coroutine,start_coroutine};


use macroquad::{audio::{load_sound, play_sound, PlaySoundParams, Sound}, prelude::*};
use mipmaps::{load_mipmapped_texture, set_cam, MipMappedTexture2D};
use ::rand::seq::SliceRandom;

use lazy_static::lazy_static;

pub struct RandomClip{
    clips : Vec<(String,Sound)>
}

impl RandomClip{
    async fn load(paths : Vec<String>) -> Result<Self, FileError>{
        let mut clips = vec![];
        for path in paths{
            ASSET_LOAD_LOG.write().unwrap().set_message(format!("Loading audio clip {}...",path));
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
    Doodle,
    Ornate,
    Tiles,
    Wooden,
    Chess,
}
impl PieceSet{
    pub fn name(&self)->&'static str{
        match self{
            Self::Standard => "Standard",
            Self::Minimal => "Minimal",
            Self::Doodle => "Doodle",
            Self::Ornate => "Ornate",
            Self::Tiles => "3D Tiles",
            Self::Wooden => "3D Wooden",
            Self::Chess => "Chess",
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
    pub tex : MipMappedTexture2D,
    pub base_scale : f32,
    pub composition_mode : CompositionMode
}

impl PieceSetAsset{
    async fn make(fname : &'static str, base_scale : f32, composition_mode : CompositionMode) -> Result<PieceSetAsset,FileError>{
        Ok(PieceSetAsset{
            tex : load_mipmapped_texture(&format!("gfx/{}",fname)).await?,
            base_scale, composition_mode
        })
    }
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
        use CompositionMode as CM;
        let set = match spec{
            PieceSet::Standard => PieceSetAsset::make(
                "pieces_sm.png",1.7,
                CM::Precomposed,
            ),
            PieceSet::Chess => PieceSetAsset::make(
                "pieces_chess.png", 1.6,
                CM::Precomposed
            ),
            PieceSet::Minimal => PieceSetAsset::make(
                "pieces_minimal.png",
                 1.3,CM::Precomposed,
            ),

            PieceSet::Ornate => PieceSetAsset::make(
                "pieces_ornate.png",
                2.2, CM::ComposeOnFlat
            ),

            PieceSet::Tiles => PieceSetAsset::make(
                "pieces_3dtiles.png",
                2.1,CM::Precomposed
            ),
            PieceSet::Wooden => PieceSetAsset::make(
                "pieces_3dwooden.png",
                2.1,CM::Precomposed
            ),
            
            PieceSet::Doodle => PieceSetAsset::make(
                "pieces_doodle.png",
                1.7, CM::Precomposed
            ),
        };
        set.await
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
    pub avatars : MipMappedTexture2D,
    pub font : Font,
    pub font_bytes : Vec<u8>,
    pub title : Texture2D,

    pub piece_slide : RandomClip,

    pub mate : Sound,
    pub capture:Sound,

    pub diagrams : HashMap<&'static str, Texture2D>,

    
}


struct AssetLoadLog{
    message : String
}


impl AssetLoadLog{
    fn message(&self)->String{
        self.message.clone()
    }
    
    fn new()->Self{
        AssetLoadLog{message:String::new()}
    }
    fn set_message(&mut self, new_mess : String){
        self.message = new_mess;
    }
}
lazy_static!{
    static ref ASSET_LOAD_LOG : Arc<RwLock<AssetLoadLog>> = Arc::new(RwLock::new(AssetLoadLog::new()));
}

pub enum AssetLoadingError{
    File(FileError),
    Font(FontError)
}

impl Display for AssetLoadingError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            AssetLoadingError::File(fe) => write!(f,"FileError: {}",fe),
            AssetLoadingError::Font(fe) => write!(f,"FontError: {}",fe),
        }
    }
}
impl From<FileError> for AssetLoadingError{
    fn from(value: FileError) -> Self {
        AssetLoadingError::File(value)
    }
}
impl From<FontError> for AssetLoadingError{
    fn from(value: FontError) -> Self {
        AssetLoadingError::Font(value)
    }
}

// Mquad doesn't have webp feature for image
async fn load_texture_webp(path : &str) -> Result<Texture2D,FileError>{

    let bytes = load_file(path).await?;
    let img = 
    image::load_from_memory_with_format(&bytes, image::ImageFormat::WebP)
    .unwrap_or_else(|e| panic!("{}", e))
    .to_rgba8();
    let width = img.width() as u16;
    let height = img.height() as u16;
    let bytes = img.into_raw();

    let tex = Texture2D::from_rgba8(width, height, &bytes);

    Ok(tex)
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
                                        &format!("Error loading assets: {}", error)
                                    )
                                });
                            });
                            egui_macroquad::draw();
                            next_frame().await
                        }
                    }
                }
            }

            
            set_cam(0.05, Vec2::ZERO);

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
            
            set_default_camera();
            draw_text(&ASSET_LOAD_LOG.read().unwrap().message(), 
            screen_width()*0.5 - 100.0,screen_height()*0.5 + 120.0, 30.0,Color::from_hex(0x111111));
            

            time += get_frame_time();
            next_frame().await
        }

    }


    
    pub async fn load()->Result<Assets,AssetLoadingError> {
        fn set_message(new_mess : String){
            ASSET_LOAD_LOG.write().unwrap().set_message(new_mess);
        }

        // Pieceset is not stored in here,
        // but we still load the default so it's
        // slotted in the loading screen.
        set_message("Loading pieceset...".to_string());
        set_pieceset(PieceSet::Standard);


        set_message("Loading font...".to_string());
        let font = load_ttf_font(FONT_PATH).await?;
        font.set_filter(FilterMode::Linear);

        let font_bytes = macroquad::file::load_file(&FONT_PATH)
            .await?;

        
        set_message("Loading diagrams...".to_string());
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
                    load_texture_webp(&format!("diags/{}.webp",name)).await?
            );
        };

        async fn tex(fname : &'static str) -> Result<Texture2D,FileError>{
            let path = format!("gfx/{}",fname);
            set_message(format!("Loading {}...",path));
            load_texture(&path).await
        }
        async fn sound(fname : &'static str) -> Result<Sound,FileError>{
            let path = format!("audio/{}",fname);
            set_message(format!("Loading {}...",path));
            load_sound(&path).await
        }

        Ok(Assets{
            btn_takeback : tex("btn_takeback.png").await?,
            btn_exit : tex("btn_exit.png").await?,
            btn_lines : tex("btn_lines.png").await?,
            btn_letters : tex("btn_letters.png").await?,
            btn_rules : tex("btn_rules.png").await? ,
            avatars :  load_mipmapped_texture("gfx/avatars.png").await?,
            font ,
            font_bytes,
            title  : tex("title.png").await?,

            piece_slide : RandomClip::load([1,3,4,5,7,8].into_iter().map(|n|format!("audio/slide{}.ogg",n)).collect()).await?,

            mate : sound("mate.ogg").await?,
            capture : sound("bopp.ogg").await?,
            diagrams,
        })
    }

    pub fn get_avatar(&self, player : Player, avatar_offset : usize) -> (MipMappedTexture2D,Rect){
        let tile_size = self.avatars.width() * 0.5;
        let avatar_src = Rect::new(
            tile_size *(avatar_offset as f32),
            tile_size * (match player {
                Player::Black => 1,
                Player::White => 0
            }) as f32,
            tile_size,tile_size
        );


        (self.avatars.clone(),avatar_src)
    }



    
    
}

#[derive(Clone)]
pub enum PieceSetPromise{
    Ready(PieceSetAsset),
    Pending(Coroutine<PieceSetAsset>),
}

lazy_static! {
    static ref ASSETS : OnceLock<Assets> = OnceLock::new();

    static ref PIECESET : Arc<RwLock<Option<PieceSetPromise>>> = Arc::new(RwLock::new(None));

    static ref DUMMY_PIECESET : PieceSetAsset = {
        PieceSetAsset { tex: MipMappedTexture2D::empty(), base_scale: 1.0, composition_mode: CompositionMode::Precomposed }
    };
}

pub async fn load_assets(){
    let assets : Assets = Assets::loading_screen().await;
    ASSETS.get_or_init(||assets);
}

pub fn get_pieceset_unchecked() -> PieceSetAsset{
    let prom : PieceSetPromise = PIECESET.read().unwrap().clone().unwrap();

    match prom {
        PieceSetPromise::Ready(val) => val,
        PieceSetPromise::Pending(co) => {
            match co.retrieve(){
                Some(val) => {
                    *PIECESET.write().unwrap() = Some(PieceSetPromise::Ready(val.clone()));
                    val
                },
                None => {
                    DUMMY_PIECESET.clone()
                }
            }
        }
    }
}
pub fn set_pieceset(set : PieceSet){
    // if let Some(..) = get_pieceset(){
    //     println!("Invalidating old pieceset tex.");
    //     PIECESET.write().unwrap().as_mut().unwrap().tex.delete();
    // }

    let co = start_coroutine(async move{
        PieceSetAsset::load(set).await.unwrap()
    });
    // println!("Loaded standard pieceset.");
    *PIECESET.write().unwrap() = Some(PieceSetPromise::Pending(co));
    
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