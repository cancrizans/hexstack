
use std::{io, net::TcpListener};

use hexstack::theme::BG_COLOR;
use tungstenite::
    handshake::server::{Request, Response}
;

use macroquad::prelude::*;

#[macroquad::main("Tokonoma Server")]
async fn main() {
    #[cfg(feature="networking")]
    {
        let server = TcpListener::bind("127.0.0.1:3012").unwrap();
        server.set_nonblocking(true).unwrap();

        loop{
            clear_background(BG_COLOR);
            
            match server.accept() {
                Ok(_stream) => {
                    println!("Yabba dabba doo");
                    let _callback = |req: &Request, response: Response| {
                        println!("Received a new ws handshake");
                        println!("The request's path is: {}", req.uri().path());
                        println!("The request's headers are:");
                        for (header, _value) in req.headers() {
                            println!("* {header}");
                        }
                        let outp: Result<tungstenite::http::Response<()>, ()> = Ok(response);
                        outp
                    };
                    
                    // let mut _websocket = accept_hdr(stream, callback);

                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    
                    // skip
                },
                Err(e) => {
                    println!("{:?}",e)
                },
            }

            next_frame().await

            
        }
    }
}