


#[allow(unused_imports)]
use hexstack::gameplay;
#[allow(unused_imports)]
use hexstack::gameplay::window_conf as game_window_conf;


use hexstack::theme;

use hexstack::assets::load_assets;

use hexstack::ui::match_config::match_config_ui;


#[macroquad::main(game_window_conf)]
async fn main(){
    
    load_assets().await;

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