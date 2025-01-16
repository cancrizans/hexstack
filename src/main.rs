


#[allow(unused_imports)]
use hexstack::gameplay;
#[allow(unused_imports)]
use hexstack::gameplay::window_conf as game_window_conf;


use hexstack::theme;

use hexstack::assets::{Assets,ASSETS};

use hexstack::ui::match_config::match_config_ui;


#[macroquad::main(game_window_conf)]
async fn main(){
    
    let assets : Assets = Assets::loading_screen().await;
    ASSETS.get_or_init(||assets);

    egui_macroquad::cfg(|egui_ctx |{
        theme::set_fonts(egui_ctx);
    });

    let mut last_match_config = None;
    loop{
        let match_config = match_config_ui(last_match_config).await;    
        
        gameplay::main(match_config.clone()).await;
        
        last_match_config = Some(match_config);
    }
}