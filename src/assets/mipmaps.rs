use std::sync::RwLock;

use macroquad::{camera::{set_camera, Camera2D}, color::Color, file::FileError, math::{vec2, Rect, Vec2}, texture::{self, draw_texture_ex, load_image, DrawTextureParams, Texture2D}, window::{screen_height, screen_width}};
use image::{imageops::{self, FilterType}, ImageBuffer};
use lazy_static::lazy_static;

struct CamState{
    zoom : f32,
    vert_rez : u32
}
impl CamState{
    fn wp_to_px(&self, wps:Vec2) -> Vec2{
        self.zoom * (self.vert_rez as f32) * wps
    }
    
}

lazy_static!{
    static ref CAM_STATE : RwLock<CamState> = RwLock::new(CamState{zoom:1.0,vert_rez:720});
}

pub fn set_cam_rez(zoom : f32, target : Vec2, vert_rez : u32) -> Camera2D{
    let cam = Camera2D{
        zoom : zoom * vec2(screen_height()/screen_width(),-1.0),
        target,
        ..Default::default()
    };
    set_camera(&cam);
    *CAM_STATE.write().unwrap() = CamState{zoom,vert_rez};
    cam
}
pub fn set_cam(zoom : f32,target : Vec2)->Camera2D{
    set_cam_rez(zoom, target, 720)
}
pub fn set_cam_from_cam2d(camera : &Camera2D){
    set_cam(-camera.zoom.y, camera.target);
}

#[derive(Clone)]
pub struct MipMappedTexture2D{
    mips : Vec<Texture2D>
}

impl MipMappedTexture2D{
    #[inline]
    pub fn mip0(&self) -> Texture2D{
        self.mips[0]
    }
    pub fn width(&self) -> f32{
        self.mip0().width()
    }
    pub fn height(&self) -> f32{
        self.mip0().height()
    }

    pub fn draw(&self, 
        x:f32,y:f32,
        color:Color,
        
        params:DrawTextureParams){
        let src_size = params.source.unwrap_or(
            Rect::new(0.0,0.0,self.width(),self.height())
        );
        let dst_wp = params.dest_size.unwrap_or(vec2(self.width(), self.height()));

        let camera = CAM_STATE.read().unwrap();
        let dst_pix = camera.wp_to_px(dst_wp);

        let scale = 0.5*((dst_pix.x/src_size.w) +(dst_pix.y/src_size.h));
        
        let mip_level = ((0.5-scale.log2()).round().max(0.0) as usize).min(self.mips.len()-1);
        
        let mip_scale = 0.5f32.powi(mip_level as i32);

        let mip = self.mips[mip_level];
        let src_mip = Rect::new(
            src_size.x * mip_scale, src_size.y * mip_scale,
            src_size.w * mip_scale, src_size.h * mip_scale
        );

        // println!("Drawing mip lvl {} scale {}",mip_level,scale);
        draw_texture_ex(
            mip, 
            x, y, 
            color, 
            DrawTextureParams{
                dest_size : Some(dst_wp),
                source : Some(src_mip),
                rotation : params.rotation,
                flip_x : params.flip_x,
                flip_y : params.flip_y,
                pivot : None
            }
        );
    }
}

const MIN_MIP_SIZE : u16 = 8;

pub async fn load_mipmapped_texture(path : &str) -> Result<MipMappedTexture2D, FileError>{
    let mut img = load_image(path).await?;

    let mut mips = vec![Texture2D::from_image(&img)];

    while img.width.max(img.height) > MIN_MIP_SIZE {
        let (w,h) = (img.width as u32, img.height as u32);
        let mut buffer : ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(w,h,img.bytes).unwrap();
        buffer = imageops::resize(&buffer,
            (img.width>>1) as u32,
            (img.height>>1) as u32,
            FilterType::Lanczos3,
        );

        img = texture::Image{
            bytes : buffer.to_vec(),
            width : buffer.width() as u16,
            height : buffer.height() as u16,
        };
        

        mips.push(Texture2D::from_image(&img));
    };

    println!("Generated {} mips from path {}",mips.len(),path);

    Ok(MipMappedTexture2D{mips})
}