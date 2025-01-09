

use hexstack::{arrows::draw_arrow, assets::Assets, neighbours_attack, theme, Piece, PieceType, Player, State, Tall, Tile};
use macroquad::prelude::*;

struct Illustration{
    zoom : f32,
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

        Illustration { zoom, name, tex}
    } 

    async fn dump(self){
        next_frame().await;
        let img = self.tex.texture.get_texture_data();
        let path_png = format!("diags/{}.png",self.name);
        img.export_png(&path_png);

        let path_webp = format!("diags/{}.webp",self.name);
        std::process::Command::new("magick")
            .arg("convert")
            .arg(path_png)
            .arg("-resize").arg("50%")
            .arg(path_webp)
            .status().unwrap();
            
    }
}

#[macroquad::main("Illustration builder")]
async fn main(){
    const FULL_BOARD_SH : (u32,u32) = (700,500);
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
                    s + 0.6* disp.normalize(),
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

    std::process::Command::new("rm")
        .arg("diags/*.png")
        .spawn()
        .unwrap();
        
}
