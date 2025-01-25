use std::{collections::HashMap, sync::{Arc, RwLock}};

use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, FontId, TextStyle, Ui};
use macroquad::prelude::*;

use crate::assets::{get_assets_unchecked, set_pieceset, PieceSet};
use lazy_static::lazy_static;


pub const BG_COLOR : Color = color_u8!(0xee,0xee,0xee,0xff);

pub const WIDG_FILL_COLOR : Color = color_u8!(0xaa,0xaa,0xaa,0xff);

pub fn egui_ctx_setup(egui_ctx : &Context){
    egui_ctx.set_pixels_per_point(screen_height() / 720.0);
    egui_ctx.set_visuals(egui::Visuals::light());
}

pub fn color_to_color32(color : Color)->Color32{
    let (r,g,b) = (color.r,color.g,color.b);

    let (r,g,b) = ((r*255.0) as u8, (g*255.0) as u8, (b*255.0) as u8);
    Color32::from_rgb(r, g, b)
}

pub fn set_fonts(egui_ctx : &Context){
    let mut fonts = FontDefinitions::default();

    let assets = get_assets_unchecked();
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
    ts.insert(
        TextStyle::Monospace, 
        FontId { 
            size: 20.0, 
            family: FontFamily::Monospace 
        });

    let ws = &mut ui.visuals_mut().widgets;

    ws.inactive.bg_fill = color_to_color32(WIDG_FILL_COLOR);
    ws.inactive.weak_bg_fill = ws.inactive.bg_fill;
}

#[derive(Clone,PartialEq)]
pub struct BoardPalette{
    dark : Color,
    mid : Color,
    light : Color
}

impl BoardPalette{
    fn from_hex(dark : u32, mid : u32, light : u32) -> Self{
        BoardPalette{
            dark : Color::from_hex(dark),
            mid : Color::from_hex(mid),
            light : Color::from_hex(light)
        }
    }

    pub fn to_egui(&self) -> [[u8;3];3]{
        [self.dark,self.mid,self.light]
        .map(|c|{
            let carr : [u8;4] = c.into();
            [carr[0],carr[1],carr[2]]
        })
        
    }
    pub fn from_egui(eguis : [[u8;3];3]) -> Self{
        let [dark,mid,light] =
        eguis.map(|eg|
            Color::from_rgba(eg[0], eg[1], eg[2], 255)
        );
        
        BoardPalette { 
            dark, mid ,light
        }
    }


    pub fn sample(&self,mod3 : u8) -> Color{
        match mod3{
            0 => self.mid,
            1 => self.dark,
            2 => self.light,
            _ => unreachable!()
        }
    }


}
impl Default for BoardPalette{
    fn default()->BoardPalette{
        BoardPalette::from_hex(0x999999, 0xbbbbbb, 0xdddddd)
    }
}

lazy_static! {
    static ref BOARD_PALETTE : Arc<RwLock<BoardPalette>> = Arc::new(RwLock::new(BoardPalette::default()));

    pub static ref BOARD_PALETTES : HashMap<&'static str, BoardPalette> = HashMap::from([
        ("Standard", BoardPalette::default()),
        ("Warm",BoardPalette::from_egui([
            [149, 141, 141],
            [191, 184, 182],
            [222, 219, 216]
        ])),
        ("Tournament", BoardPalette::from_egui([
            [122, 164, 156],
            [202, 167, 167],
            [232, 230, 230],
        ])),
        ("Ocean", BoardPalette::from_hex(0x9898ae, 0x94d5b7, 0xeae2e2)),
        ("Wood", BoardPalette::from_egui([
            [160, 143, 114],
            [188, 173, 136],
            [208, 198, 168]
        ])),
        ("Honey", BoardPalette::from_egui([
            [208, 148, 76],
            [225, 176, 84],
            [238, 206, 112]
        ])),
        ("Lavender", BoardPalette::from_egui([
            [187, 189, 199],
            [215, 207, 225],
            [239, 226, 232]
        ]))


    ]);
}

#[inline]
pub fn get_board_palette()->BoardPalette{
    get_theme_config().board_palette()
}


#[derive(Clone,PartialEq)]
pub enum BoardPaletteConfig{
    Named(&'static str),
    Custom(BoardPalette)
}
impl BoardPaletteConfig{
    pub fn is_custom(&self)->bool{
        match self{
            BoardPaletteConfig::Custom(..) => true,
            _ => false
        }
    }
    pub fn to_palette(&self) -> BoardPalette{
        match self{
            BoardPaletteConfig::Named(name) => BOARD_PALETTES[name].clone(),
            BoardPaletteConfig::Custom(pal) => pal.clone()
        }
    }
}

#[derive(Clone)]
pub struct ThemeConfig{
    pub board_palette : BoardPaletteConfig,

    pieceset : PieceSet,
    
    
}

impl ThemeConfig{
    fn board_palette(&self) -> BoardPalette{
        self.board_palette.to_palette()
    }
}

impl Default for ThemeConfig{
    fn default() -> Self {
        ThemeConfig { 
            board_palette: BoardPaletteConfig::Named("Standard") ,
            pieceset : PieceSet::Standard
        }
    }
}

impl ThemeConfig{
    pub fn get_pieceset(&self) -> PieceSet {self.pieceset}
    pub async fn set_pieceset(&mut self, new_value : PieceSet){
        // this is bad . But they forced my hand
        if self.pieceset != new_value{
            self.pieceset = new_value;
            set_pieceset(self.pieceset).await.unwrap()
        }
    }
}

lazy_static!{
    pub static ref THEME_CONFIG : Arc<RwLock<ThemeConfig>> = Arc::new(RwLock::new(ThemeConfig::default()));
}




pub fn get_theme_config() -> ThemeConfig{
    THEME_CONFIG.read().unwrap().clone()
}


#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn board_color_conversions(){
        let pal = get_board_palette();
        let pal2 = BoardPalette::from_egui(pal.to_egui());

        assert!((pal.dark.r - pal2.dark.r).abs() < 0.01);
    }
}