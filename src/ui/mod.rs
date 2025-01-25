pub mod editor;
pub mod rulesheet;
pub mod match_config;
pub mod engine_eval;
pub mod theme_config;


use macroquad::prelude::*;

use crate::assets::get_assets_unchecked;

pub struct MqUi<'a>{
    pub camera : &'a Camera2D
}

impl<'a> MqUi<'a>{
    pub fn new( camera : &'a Camera2D)->Self{
        MqUi { camera }
    }
}

#[derive(Clone)]
pub struct Button{
    img : Texture2D,
    alpha : f32,
    rect : Rect,
    text : String,
}

impl Button{
    pub fn new(img : Texture2D, rect : Rect, text : String)->Button{
        Button { img, alpha: 0.0, rect, text}
    }


    pub fn process(&mut self, ui : &MqUi) -> bool{
        let mouse_px = mouse_position().into();
        let mouse_world = ui.camera.screen_to_world(mouse_px);

        let mouse_in_rect = self.rect.contains(mouse_world);

        let target_alpha = if mouse_in_rect{
            1.0
        } else {0.0};

        self.alpha += (target_alpha-self.alpha) * 6.0 * get_frame_time();

        let clicked = mouse_in_rect && is_mouse_button_pressed(MouseButton::Left);

        if clicked {self.alpha = 0.0;}

        draw_texture_ex(
            self.img, 
            self.rect.x, 
            self.rect.y, 
            Color::from_rgba(255, 255, 255, (64.0 + 191.0 * self.alpha) as u8), 
            DrawTextureParams{
                dest_size : Some(self.rect.size()),
                ..Default::default()
            });

        let font = get_assets_unchecked().font;

        if self.alpha > 0.001{
            let (font_size, font_scale, font_scale_aspect) = camera_font_scale(0.55);
            draw_text_ex(
                &self.text,
                self.rect.x + self.rect.w,
                self.rect.y + self.rect.h * 0.5 + 0.25,
                TextParams{font,font_scale,font_scale_aspect,font_size,
                    color : Color::from_rgba(0x11, 0x11, 0x11, (255.0 * self.alpha) as u8),
                    ..Default::default()
                }
            );
        }

        clicked
    }

}


pub fn draw_text_centered(
    text : &str,
    font : Font,
    world_font_size : f32,
    center : Vec2,
    color : Color,
){
    let (x,y) = center.into();
    let (font_size, font_scale, font_scale_aspect) 
        = camera_font_scale(world_font_size);
    let center = get_text_center(
        text, Some(font), font_size, font_scale, 0.0);
    draw_text_ex(text,x-center.x,y-center.y, TextParams{
        font,
        font_size, font_scale, font_scale_aspect,
        color,
        ..Default::default()
    });
}