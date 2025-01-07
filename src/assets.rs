use macroquad::prelude::*;
pub struct Assets{
    pub pieces : Texture2D,
    pub btn_takeback : Texture2D,
    pub btn_exit : Texture2D,
    pub btn_lines : Texture2D,
    pub avatars : Texture2D,
}

impl Assets{
    pub async fn load()->Self{
        Assets{
            pieces : load_texture("gfx/pieces_sm.png").await.unwrap(),
            btn_takeback : load_texture("gfx/btn_takeback.png").await.unwrap(),
            btn_exit : load_texture("gfx/btn_exit.png").await.unwrap(),
            btn_lines : load_texture("gfx/btn_lines.png").await.unwrap(),
            avatars :  load_texture("gfx/avatars.png").await.unwrap()
        }
    }
}