use macroquad::prelude::*;
use hexstack::assets::mipmaps::*;

#[macroquad::main("Mipmaps")]
async fn main(){
    let mipped : MipMappedTexture2D = 
        load_mipmapped_texture("gfx/pieces_sm.png").await.unwrap();


    let mut time:f32 = 0.0;

    loop {
        clear_background(WHITE);

        let camera_zoom = 2.0f32.powf(-1.0+2.0*time.sin());
        set_cam(
            camera_zoom, Vec2::ZERO
        );

        mipped.draw(0.0, 0.0, 
            WHITE, 
            
            DrawTextureParams{
                dest_size : Some(vec2(1.0,1.0)),
                ..Default::default()
            }
        );

        time += get_frame_time();
        next_frame().await;
    }
}