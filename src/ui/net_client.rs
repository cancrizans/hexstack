use std::net::TcpStream;

use egui::Margin;
use macroquad::prelude::*;
use
{
   ws_stream_wasm       :: *                        ,
};

use crate::theme::{egui_ctx_setup, BG_COLOR};
use macroquad::experimental::coroutines::{start_coroutine,Coroutine};
use send_wrapper::SendWrapper;
// type Socket = WebSocket<MaybeTlsStream<TcpStream>>;


enum Connection{
    None,
    Pending(Coroutine<Result<(WsMeta,WsStream),WsErr>>),
    Failed(WsErr),
    Connected
    {
        wsmeta : WsMeta,
        stream : WsStream
    }
}


pub async fn net_client_ui(){

    let mut connection = Connection::None;

    loop{
        let mut do_connect = false;

        clear_background(BG_COLOR);
        set_default_camera();
        egui_macroquad::ui(|egui_ctx|{
            egui_ctx_setup(egui_ctx);

            egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                .inner_margin(Margin::symmetric(30.0, 60.0))
                
            )
            .show(egui_ctx,|ui|{
                ui.label("Connection");

                let enabled_btn = match &connection{
                    Connection::Pending(..) => false,
                    _ => true
                };

                if ui.add_enabled(enabled_btn, egui::Button::new("Connect"))
                .clicked(){
                    do_connect = true;
                }

                match &connection{
                    Connection::Connected { wsmeta,stream }
                     => {ui.label(format!("{:?} {:?}",wsmeta,stream));},
                    Connection::Failed(e) => {ui.label(format!("Connection failed: {:?}.",e));},
                    Connection::Pending(..) => {ui.label("Connecting...");},
                    Connection::None => {}
                };
            });
        });
        egui_macroquad::draw();

        if do_connect{
            connection = Connection::Pending(
                start_coroutine(
                    SendWrapper::new(WsMeta::connect( "ws://127.0.0.1:3012", None ))
                )
            );
            do_connect = false;
        }

        match connection{
            Connection::Pending(ref corout) => {
                match corout.retrieve(){
                    None => {},
                    Some(result) => {
                        connection = match result{
                            Ok((wsmeta,stream)) => Connection::Connected { wsmeta,stream },
                            Err(e) => Connection::Failed(e)
                        }
                        
                    }
                }
            },
            _ => {}
            
        };

        next_frame().await;
    }
}