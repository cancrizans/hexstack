use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, FontId, TextStyle, Ui};
use macroquad::prelude::*;

use crate::assets::{Assets, ASSETS};

pub const BG_COLOR : Color = color_u8!(0xee,0xee,0xee,0xff);

pub const WIDG_FILL_COLOR : Color = color_u8!(0xaa,0xaa,0xaa,0xff);

pub fn color_to_color32(color : Color)->Color32{
    let (r,g,b) = (color.r,color.g,color.b);

    let (r,g,b) = ((r*255.0) as u8, (g*255.0) as u8, (b*255.0) as u8);
    Color32::from_rgb(r, g, b)
}

pub fn set_fonts(egui_ctx : &Context){
    let mut fonts = FontDefinitions::default();

    let assets = ASSETS.get().unwrap();
    // Install my own font (maybe supporting non-latin characters):
    fonts.font_data.insert("my_font".to_owned(),
        FontData::from_owned(assets.font_bytes.clone()));

    // Put my font first (highest priority):
    fonts.families.get_mut(&FontFamily::Proportional).unwrap()
        .insert(0, "my_font".to_owned());

    // // Put my font as last fallback for monospace:
    // fonts.families.get_mut(&FontFamily::Monospace).unwrap()
    //     .push("my_font".to_owned());

    egui_ctx.set_fonts(fonts);
}


pub fn set_theme(ui : &mut Ui){
    let ts = &mut ui.style_mut().text_styles;
    ts.insert(
        TextStyle::Heading, 
        FontId { 
            size: 32.0, 
            family: FontFamily::Proportional 
        });
    ts.insert(
        TextStyle::Body, 
        FontId { 
            size: 20.0, 
            family: FontFamily::Proportional 
        });
    ts.insert(
        TextStyle::Button, 
        FontId { 
            size: 20.0, 
            family: FontFamily::Proportional 
        });

    let ws = &mut ui.visuals_mut().widgets;

    ws.inactive.bg_fill = color_to_color32(WIDG_FILL_COLOR);
    ws.inactive.weak_bg_fill = ws.inactive.bg_fill;
}