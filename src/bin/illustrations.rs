

use futures::stream::FlatMap;
use hexstack::{arrows::draw_arrow, assets::Assets, neighbours_attack, theme, Piece, Player, State, Tall, Tile};
use hexstack::PieceType;
use macroquad::prelude::*;

struct Illustration{
    name : String,

    tex : RenderTarget
}

impl Illustration{
    fn new(zoom : f32, name : String, shape : (u32,u32), target : Vec2) -> Self{
        let tex = render_target(2*shape.0, 2*shape.1);

        
        set_camera(&Camera2D{
            zoom : vec2((shape.1 as f32)/(shape.0 as f32), -1.0) * zoom,
            target,
            render_target : Some(tex),
            ..Default::default()
        });

        Illustration { name, tex}
    } 

    async fn dump(self){
        next_frame().await;
        let img = self.tex.texture.get_texture_data();
        let mut path_temp = std::env::temp_dir();
        path_temp.push("tmpdiag.png");
        img.export_png(&path_temp.to_str().unwrap());

        let path_final = format!("diags/{}.png",self.name);
        std::process::Command::new("magick")
            .arg("convert")
            .arg(path_temp)
            .arg("-resize").arg("50%")
            .arg(path_final)
            .status().unwrap();
            
    }
}

fn draw_piece(color : Player, species : PieceType, tile : Tile, tex : Texture2D){
    let (x,y) = tile.to_world(false).into();
    Piece{color,species}.draw(x, y, tex, 1.0);
}

#[macroquad::main("Illustration builder")]
async fn main(){
    const FULL_BOARD_SH : (u32,u32) = (420,300);
    const FULL_BOARD_ZOOM : f32 = 0.22;

    let assets = Assets::load().await;

    let i = Illustration::new(
        FULL_BOARD_ZOOM, "board".to_string(), FULL_BOARD_SH, Vec2::ZERO);

    clear_background(theme::BG_COLOR);
    
    Tile::draw_board(false);

    let wcorn = Tile::corner(Player::White);
    wcorn.draw_highlight_outline(0.2, GRAY, false);
    wcorn.draw_highlight_outline(0.1, WHITE, false);
    Tile::corner(Player::Black).draw_highlight_outline(0.1, BLACK, false);

    // Tile::draw_tile_numbers(assets.font, false);

    i.dump().await;



    let i = Illustration::new(FULL_BOARD_ZOOM, "board_setup".to_string(), FULL_BOARD_SH, Vec2::ZERO);

    clear_background(theme::BG_COLOR);
    Tile::draw_board(false);

    let state = State::setup();
    state.draw(assets.pieces, assets.font, false,false,false);

    i.dump().await;



    let center_tile = Tile::from_xyz(0, 0,0).unwrap();

    
    const MOV_SH : (u32,u32) = (300,300);

    for (n,pt, zoom) in [
        ("flat", PieceType::Flat,0.4),
        ("arm",PieceType::Lone(Tall::Hand),0.27),
        ("blind",PieceType::Lone(Tall::Blind),0.27),
        ("star", PieceType::Lone(Tall::Star),0.3)
    ]{
        for (cn,color) in [("white",Player::White), ("black",Player::Black)]{

            let start_tile = match pt {
                PieceType::Lone(tall) => match tall{
                    Tall::Star => match color{
                        Player::White => Tile::from_xyz(0, -1, 1).unwrap(),
                        Player::Black => Tile::from_xyz(0, 1, -1).unwrap(),
                    },
                    _ => center_tile
                },
                _ => center_tile
            };

            let i = Illustration::new(zoom,format!("mov_{}_{}",n,cn),MOV_SH, start_tile.to_world(false).into());

            clear_background(theme::BG_COLOR);
            Tile::draw_board(false);

            let pis = Piece{color, species : pt};

            let s : Vec2 = start_tile.to_world(false).into();

            neighbours_attack(start_tile, pis).into_iter()
            .for_each(|n|{
                let e : Vec2 = n.to_world(false).into();
                let disp : Vec2 = e-s;

                draw_arrow(
                    s + 0.7* disp.normalize(),
                    e,
                    Color::from_hex(0x111111),
                    0.10,
                    0.4,
                    0.6
                );
            });

            let (x,y) = start_tile.to_world(false);
            pis.draw(x,y, assets.pieces, 1.0);

            i.dump().await;
        }
    };
    

    let i = Illustration::new(
        0.5, "stack_pre".to_string(), (400,200), vec2(0.0,0.0)
    );

    Tile::draw_board(false);
    let (x,y) = Tile::from_xyz(0, 1, -1).unwrap().to_world(false).into();
    Piece{color : Player::White, species : PieceType::Flat}.draw(x, y, assets.pieces, 1.0);

    let (x,y) = Tile::from_xyz(0, -1, 1).unwrap().to_world(false).into();
    Piece{color : Player::White, species : PieceType::Lone(Tall::Hand)}.draw(x, y, assets.pieces, 1.0);


    i.dump().await;

    let i = Illustration::new(
        0.5, "stack_post".to_string(), (400,200), vec2(0.0,0.0)
    );

    Tile::draw_board(false);
    let (x,y) = Tile::from_xyz(0, 1, -1).unwrap().to_world(false).into();
    Piece{color : Player::White, species : PieceType::Stack(Tall::Hand)}.draw(x, y, assets.pieces, 1.0);

    let (sx,sy) = Tile::from_xyz(0, -1, 1).unwrap().to_world(false).into();
    
    draw_arrow((sx,sy).into(), (-1.0,y).into(), Color::from_hex(0x111111),
    0.10,
    0.4,
    0.6);

    i.dump().await;

    // std::process::Command::new("rm")
    //     .arg("diags/*.png")
    //     .spawn()
    //     .unwrap();
    
    for (name, post) in [("attack_pre",false), ("attack_post",true)]{
        let i = Illustration::new(
            0.35, name.to_string(), (350,350), vec2(-0.35,-0.5)
        );

        clear_background(theme::BG_COLOR);
        Tile::draw_board(false);

        let c00 = Tile::from_xyz(0, 0,0).unwrap();
        let c01 = Tile::from_xyz(0, 1,-1).unwrap();
        let c10 = Tile::from_xyz(-1, 0,1).unwrap();
        let c11 = Tile::from_xyz(-1, 1,0).unwrap();

        let cm10 = Tile::from_xyz(1, 0, -1).unwrap();
        let c21 = Tile::from_xyz(-2, 1, 1).unwrap();
        let oritile = Tile::from_xyz(0,-1,1).unwrap();

        if post {
            c11.draw_highlight_fill(Color::from_hex(0xc07070),false);
        }

        draw_piece(Player::White, PieceType::Flat, if post {c00} else {oritile}, assets.pieces);
        draw_piece(Player::White, PieceType::Flat, c10, assets.pieces);
        draw_piece(Player::Black, PieceType::Flat, c01, assets.pieces);
        draw_piece(Player::Black, PieceType::Flat, c11, assets.pieces);


        if post {
            let dull : Color = Color::new(0.0,0.0,0.0,0.4);

            for (n1,n2,col) in [
                (c00,c01,WHITE), (c10,c11,WHITE), (c00,c11,WHITE),
                (c00,cm10, dull), (c10, c21, dull)
            ]{
                let s :Vec2 = n1.to_world(false).into();
                let e : Vec2 = n2.to_world(false).into();
                let ss = s.lerp(e, 0.35);
                let ee = s.lerp(e, 0.65); 
                draw_arrow(ss,ee, col, 0.15, 0.2, 0.4);
            }

            let ori = oritile.to_world(false).into();
            draw_arrow(
                ori, 
                ori.lerp(c00.to_world(false).into(), 0.6), 
                Color::new(0.1,0.1,0.1,0.8),0.15, 0.3, 0.6);

        }

        i.dump().await;
    }
        
}
