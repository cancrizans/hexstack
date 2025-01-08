use coroutines::start_coroutine;
use macroquad::prelude::*;
pub struct Assets{
    pub pieces : Texture2D,
    pub btn_takeback : Texture2D,
    pub btn_exit : Texture2D,
    pub btn_lines : Texture2D,
    pub avatars : Texture2D,
    pub font : Font
}

impl Assets{

    pub async fn loading_screen() -> Self{
        let load_co = start_coroutine(Assets::load());
        let mut time : f32 = 0.0;

        
        loop {
            clear_background(WHITE);

            if time > 0.5{
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
        let font = load_ttf_font("gfx/Roboto-Regular.ttf").await.unwrap();
        font.set_filter(FilterMode::Linear);

        Assets{
            pieces : load_texture("gfx/pieces_sm.png").await.unwrap(),
            btn_takeback : load_texture("gfx/btn_takeback.png").await.unwrap(),
            btn_exit : load_texture("gfx/btn_exit.png").await.unwrap(),
            btn_lines : load_texture("gfx/btn_lines.png").await.unwrap(),
            avatars :  load_texture("gfx/avatars.png").await.unwrap(),
            font ,
        }
    }
}