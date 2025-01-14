
use itertools::Itertools;
use macroquad::prelude::*;
use std::{collections::{hash_map::{Entry, ExtractIf}, HashMap}, fmt::Display};
use lazy_static::lazy_static;
use ::rand::Rng;
use memoize::memoize;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Tile(u8);

pub const BOARD_RADIUS : i8 = 3;
const BOARD_SHORT_RADIUS : i8 = 2;



#[allow(dead_code)]
pub const BOARD_SIZE : usize = 29;

use crate::arrows::draw_arrow;

impl Tile{
    
    //mapping coordinate range -3..=3 to 0..=6
    //allows us to write an (x,y) Tile in a u8 like this
    // 0YYY0XXX
    // with the 3th and 7th unset bits for padding.
    // if delta-x is expressed in 3-bit 2's complement
    // then we can do shifts in a single sum + mask.
    const OFF_Y : i8 = BOARD_RADIUS;
    const OFF_X : i8 = BOARD_SHORT_RADIUS;

    #[inline]
    pub const fn new(value : u8) -> Option<Tile>{
        //assumption that value already has bits 3 and 7 unset.
        let ux_hi = value << 4;

        if !(ux_hi < 0x50) {
            return None
        }
        if !(value < 0x70){
            return None
        }

        // this is 5-z placed in high nibble
        let uz_sh = (value & 0xF0) + ux_hi;
        if ! ((0x20 <= uz_sh) & (uz_sh <= 0x80)){
            return None
        }

        
        Some(Tile(value))
        
    }

    #[inline]
    pub const fn from_uxy(ux : u8, uy : u8)-> Tile{
        Tile((uy << 4) | ux)
    }

    #[inline]
    pub const fn code(&self) -> u8{
        let (ux, uy) = (self.ux(),self.uy());

        let shift = match ux{
            4 => 0,
            3 => 5,
            2 => 5+6,
            1 => 5+6+7 - 1,
            0 => 5+6+7+6 - 2,
            
            _ => unreachable!()
        };

        shift + uy
    }

    pub fn glyph(&self) -> char{
        "abcdefghijklmnopqrstuvwxyzøñł".chars().nth(self.code() as usize).unwrap()
    }

    #[inline]
    pub const fn from_code(code : u8) -> Tile{
        let (ux,shift) = match code{
            0..5 =>  (5,0),
            5..11 => (4,5),
            11..18 => (3,11),
            18..24 => (2,17),
            24..29 => (1,22),
            _ => unreachable!()
        };
        let uy = code - shift;

        Tile((uy << 4) | ux)
    }

    #[inline]
    const fn ux(&self) -> u8{
        self.0 & 0xF
    }

    #[inline]
    pub const fn x(&self) -> i8{
        (self.ux() as i8) - Tile::OFF_X
    }

    #[inline]
    const fn uy(&self) -> u8{
        self.0 >> 4
    }

    #[inline]
    pub const fn y(&self) -> i8{
        (self.uy() as i8) - Tile::OFF_Y
    }

    
    #[inline]
    pub const fn z(&self) -> i8{
        -self.x()-self.y()
    }

    #[inline]
    pub const fn from_xyz(x:i8,y:i8,z:i8)->Option<Tile>{
        if x+y+z != 0 {
            panic!("Incorrect axial coords.")
        }
        
        let in_range = 
            (-BOARD_SHORT_RADIUS <= x)
            & (x <= BOARD_SHORT_RADIUS)
            & (-BOARD_RADIUS <= y)
            & (y <= BOARD_RADIUS)
            & (-BOARD_RADIUS <= z)
            & (z <= BOARD_RADIUS);
        
        
        if in_range {
            let ys = (y + Tile::OFF_Y) as u8;
            assert!(ys < 0x10);
            let xs = (x + Tile::OFF_X) as u8;
            assert!(xs < 0x10);
            Some(Tile::from_uxy(xs, ys))
        } else {
            None
        }
    }


    #[inline]
    pub const fn from_xyz_unchecked(x:i8,y:i8,z:i8)->Tile{
        match Tile::from_xyz(x, y, z){
            Some(tile) => tile,
            None => panic!("from_xyz_unchecked None")
        }
    }

    pub const fn antipode(&self) -> Tile{
        Tile::from_xyz_unchecked(-self.x(), -self.y(), -self.z())
    }

    #[allow(dead_code)]
    pub const fn mirror(&self) -> Tile{
        Tile::from_xyz_unchecked(self.x(), -self.y()-self.x(), -self.z()-self.x())
    }


    pub fn adjacent(&self) -> [Option<Tile>;6]{
        let (x,y,z) = (self.x(), self.y(), self.z());
        [
            (x+1,y-1,z), (x,y+1,z-1), (x-1,y,z+1),
            (x-1,y+1,z), (x,y-1,z+1), (x+1,y,z-1),
        ].map(|(x,y,z)|Tile::from_xyz(x, y, z))
    }

    #[inline]
    fn move_neighbours(&self, kind : &Piece) -> [Option<Tile>;6]{

        let white_offsets : [Option<Delta>;6] = match kind.species{
            PieceType::Flat => [
                Some(Delta::WH_FORWARD), 
                Some(Delta::WH_FRONTDOWN),
                Some(Delta::WH_FRONTUP),
                None, None, None
            ],
            
            PieceType::Lone(tall) | PieceType::Stack(tall)
            => {
                match tall {
                    Tall::Hand => [
                        Some(Delta::WH_FORWARD.scale(2)),
                        Some(Delta::WH_BACKUP),
                        Some(Delta::WH_BACKDOWN),
                        Some(Delta::WH_BACKWARD),
                        None,None
                    ],

                    Tall::Blind => [
                        Some(Delta::WH_FRONTUP.scale(2)),
                        Some(Delta::WH_BACKUP),
                        Some(Delta::WH_BACKWARD.scale(2)),
                        Some(Delta::WH_BACKDOWN),
                        Some(Delta::WH_FRONTDOWN.scale(2)),
                        None
                    ],


                    Tall::Star => [
                        Some(Delta::WH_DIAG),
                        Some(Delta::WH_DIAG.cycle()),
                        Some(Delta::WH_DIAG.cycle().cycle()),
                        Some(Delta::WH_DIAG.flip()),
                        Some(Delta::WH_DIAG.flip().cycle()),
                        Some(Delta::WH_DIAG.flip().cycle().cycle()),
                        ],
                }
            }
            
        };

        let offsets = match kind.color{
            Player::White => white_offsets,
            Player::Black => white_offsets.map(
                |o|
                o.map(|d|d.flip())
            )
        };
        
        offsets.map(|opt|
            opt.map(|off|self.shift(off)).flatten()
        )
        
    }


    

    pub fn to_world(&self, flip_board : bool) -> (f32,f32){
        const SQRT3 : f32 = 1.73205080757;
        const SQRT3_2 : f32 = 0.86602540378;
        let (x,y) = (1.5* (self.x() as f32) ,
                  SQRT3_2 * ( self.x() as f32) +    SQRT3 * (self.y() as f32));

        let world = (-y,x);
        if flip_board {(-world.0,-world.1)} else {world}
    }

    pub fn from_world(x : f32 , y : f32, flip_board : bool) -> Option<Tile>{
        const SQRT3 : f32 = 1.73205080757;
        const ONE_3 : f32 = 0.33333333333;

        
        let (x,y) = if flip_board{
            (-x as f32, -y as f32)
        } else{
            (x as f32, y as f32)
        };

        let (tx,ty) = (
            ONE_3 * 2.0 * y,
            ONE_3 * (-SQRT3 * x  - y)
        );

        let tx = tx.round() as i32;
        let tx = (tx.clamp(-100, 100)) as i8;
        let ty = ty.round() as i32;
        let ty = (ty.clamp(-100, 100)) as i8;

        Tile::from_xyz(tx, ty, -tx-ty)
    }

    pub fn mod3(&self) -> u8{
        (self.x()-self.y()).rem_euclid(3) as u8
    }

    pub fn all_tiles() -> impl Iterator<Item = Tile>{
        let range = -(BOARD_RADIUS as i8)..=BOARD_RADIUS as i8;
        range.clone().cartesian_product(range)
            .map(|(x,y)|Tile::from_xyz(x, y, -x-y))
            .flatten()
    }

    pub fn draw_highlight_outline(&self, thickness : f32, color : Color, flip_board : bool){
        let (x,y) = self.to_world(flip_board);
        draw_hexagon(x, y, 1.0, thickness, true, color, Color::from_rgba(0, 0,0,0));
    }

    pub fn draw_highlight_fill(&self, color : Color, flip_board : bool){
        let (x,y) = self.to_world(flip_board);
        draw_hexagon(x, y, 1.0, 0.0, true,BLACK, color);
    }

    pub fn draw_move_target(&self, color : Player, piece_tex : Texture2D, flip_board : bool){
        let (x,y) = self.to_world(flip_board);
        const R : f32 = 1.0;
        let src_off = match color{
            Player::White => 0.0,
            Player::Black => 1.0
        };

        draw_texture_ex(
            piece_tex,
            x-R,
            y-R,
            WHITE, DrawTextureParams{
                dest_size : Some(vec2(2.0*R,2.0*R)),
                source : Some(Rect::new(0.0,128.0*(1.0 + 2.0*src_off),128.0,128.0)),
                ..Default::default()
            }
        )
        // draw_circle(x, y, R, WHITE);
        // draw_circle_lines(x, y, R, 0.1, BLACK);
    }

    fn tile_color(&self) -> Color{
        match self.mod3(){
            0 => Color::from_hex(0xbbbbbb),
            1 => Color::from_hex(0x999999),
            2 => Color::from_hex(0xdddddd),
            _ => unreachable!()
        }
    }

    pub fn draw_board(flip_board : bool){

        // self.edges.iter().for_each(|(et,t,n)|{
        //     let (x1,y1) = t.to_world();
        //     let (x2,y2) = n.to_world();
        //     let (xm,ym) = (0.5*(x1+x2),0.5*(y1+y2));
        //     draw_line(x1, y1, xm, ym, 0.1, et.to_color());
        // });
        Self::all_tiles().for_each(|t|{
            let (x,y) = t.to_world(flip_board);

            let tile_color = t.tile_color();

            draw_hexagon(x, y, 
                1.0, 
                0.0,//0.05, 
                true,
                Color::from_hex(0x111111),
                tile_color);

            
        });

        
    }

    pub fn draw_tile_numbers(font : Font, flip_board : bool){
        Self::all_tiles().for_each(|t|{
            let (x,y) = t.to_world(flip_board);
            let (x,y) = (x,y+0.4);

            let mut tcol = t.tile_color();
            tcol.a = 0.8;
            // draw_rectangle(tx-0.03, ty-0.4, 0.5, 0.5, tcol);
            draw_circle(x, y, 0.3, tcol);

            let text = &format!("{}",t);
            let (font_size, font_scale, font_scale_aspect) = camera_font_scale(0.5);
            let center = get_text_center(text, Some(font), font_size, font_scale, 0.0);
            draw_text_ex(text,x-center.x,y-center.y, TextParams{
                font,
                font_size, font_scale, font_scale_aspect,
                color : Color::from_rgba(0x11, 0x11, 0x11, 160),
                ..Default::default()
            });

        });
    }

    const CORNER_BLACK : Tile = Tile::from_xyz_unchecked(0,BOARD_RADIUS as i8,-(BOARD_RADIUS as i8));
    const CORNER_WHITE : Tile = Tile::from_xyz_unchecked(0, -(BOARD_RADIUS as i8),BOARD_RADIUS as i8);

    pub const fn corner(color : Player) -> Tile{
        match color{
            Player::Black => Self::CORNER_BLACK,
            Player::White => Self::CORNER_WHITE,
        }
    }

    const fn shift(self, delta : Delta) -> Option<Tile>{
        let value = self.0.wrapping_add(delta.0) & 0b11110111;
        Tile::new(value)
        // Tile::from_xyz(self.x()+delta.dx(), self.y()+delta.dy(), self.z()+delta.dz())
    }

    
}


#[memoize]
pub fn neighbours_move(tile : Tile, piece : Piece) -> [Option<Tile>;6]{
    tile.move_neighbours(&piece)
}

#[inline]
pub fn neighbours_attack(tile : Tile, piece : Piece) -> [Option<Tile>;6]{
    neighbours_move(tile, piece)
}



#[inline]
const fn u3_to_i3(v : u8) -> i8{
    let low = (v & 0b11) as i8;
    if v&0b100 != 0{
        low - 4
    } else {
        low
    }
}

#[inline]
const fn i3_to_u3(v : i8) -> u8{
    (v as u8) & 0b111
}



#[derive(Clone, Copy)]
struct Delta(u8);

impl Delta{
    const fn from_xyz(dx:i8,dy:i8,dz:i8)->Delta{
        if dx + dy + dz != 0 {
            panic!()
        }

        let udx = i3_to_u3(dx);
        let udy = dy as u8;

        Delta((udy<<4)|udx)
    }

    #[inline]
    const fn dx(&self)->i8{
        u3_to_i3(self.0)
    }
    #[inline]
    const fn dy(&self)->i8{
        (self.0 as i8) >> 4
    }
    #[inline]
    const fn dz(&self) -> i8{
        -self.dx()-self.dy()
    }

    const WH_FORWARD : Delta = Delta::from_xyz(0,1,-1);
    const WH_FRONTUP : Delta = Delta::from_xyz(-1, 1, 0);
    const WH_FRONTDOWN : Delta = Delta::from_xyz(1,0,-1);

    const WH_BACKWARD : Delta = Self::WH_FORWARD.mirror();
    const WH_BACKUP : Delta = Self::WH_FRONTUP.mirror();
    const WH_BACKDOWN : Delta = Self::WH_FRONTDOWN.mirror();

    const WH_DIAG : Delta = Delta::from_xyz(2,-1,-1);

    const fn flip(self) -> Delta{
        Delta::from_xyz(-self.dx(), -self.dy(), -self.dz())
    }
    const fn mirror(self) -> Delta{
        Delta::from_xyz(self.dx(), self.dz(), self.dy())
    }

    const fn scale(self, factor : i8) -> Delta{
        Delta::from_xyz(
            self.dx() * factor, 
            self.dy() * factor, 
            self.dz() * factor)
    }

    const fn cycle(self) -> Delta{
        Delta::from_xyz(self.dy(), self.dz(), self.dx())
    }
}


#[allow(dead_code)]
pub const fn coord_to_char(v : i8) -> char{
    match v{
        -3 => 'α',
        -2 => 'β',
        -1 => 'γ',
        0 => 'δ',
        1 => 'ε',
        2 => 'ζ',
        3 => 'η',
        _ => 'X'
    }
}


impl Display for Tile{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let row = self.x();
        let row_name = match row{
            -2 => 'a',
            -1 => 'b',
            0 => 'c',
            1 => 'd',
            2 => 'e',
            _ => unreachable!()
        };
        
        let tile_nr = -self.y() + match row{
            -2..=0 => 4,
            1 => 3,
            2 => 2,
            _ => unreachable!()
        };

        write!(f,"{}{}",row_name,tile_nr)
    }
}







#[derive(Clone, Copy,PartialEq, Eq,Hash, Debug)]
pub enum Player{White,Black}
impl Player{
    pub fn flip(&self)->Player{
        match self{
            Player::White=>Player::Black,
            Player::Black=>Player::White
        }
    }
    pub fn to_color(&self) -> Color{
        match self{
            Player::Black => Color::from_hex(0x000000),//Color::from_hex(0x8ec8fd),
            Player::White => Color::from_hex(0xffffff),
        }
    }

    pub fn ui_info_pos(&self) -> Vec2 {
        vec2(5.0,5.0) * match self{
            Player::White => 1.0,
            Player::Black => -1.0,
        } 
    }
}

#[derive(Clone, Copy,Debug,Hash,PartialEq, Eq)]
pub enum Tall{
    Hand,
    Blind,
    Star
}

impl Tall{
    fn value(&self) -> f32{
        match self{
            Self::Hand => 2.0,
            Self::Blind => 3.0,
            Self::Star => 3.0,
        }
    }
}



#[derive(Clone, Copy,PartialEq, Eq,Hash, Debug)]
pub enum PieceType{
    Flat,
    Lone(Tall),
    Stack(Tall),
}

impl PieceType{
    pub fn unstack(self) -> Box<dyn Iterator<Item=PieceType>>{
        match self{
            PieceType::Stack(tall) => Box::new([
                PieceType::Flat,
                PieceType::Lone(tall),
            ].into_iter()),
            _ => Box::new([self].into_iter())
        }
    }

    const NUM_VALUES : usize = 7;

    pub fn value(&self) -> f32{
        match self {
            PieceType::Flat => {
                2.0
            },
            PieceType::Lone(tall) => tall.value(),
            PieceType::Stack(tall) => (PieceType::Flat.value() + tall.value()) - 0.2,
        }
    }

    #[inline]
    pub const fn code(&self) -> u8{
        match self{
            PieceType::Flat => 0,
            PieceType::Lone(tall) => match tall{
                Tall::Hand => 1,
                Tall::Blind => 2,
                Tall::Star => 3
            },
            PieceType::Stack(tall) => match tall{
                Tall::Hand => 4,
                Tall::Blind => 5,
                Tall::Star => 6
            }
        }
    }

    pub const fn from_code(code : u8) -> PieceType{
        match code {
            0 => PieceType::Flat,
            1 => PieceType::Lone(Tall::Hand),
            2 => PieceType::Lone(Tall::Blind),
            3 => PieceType::Lone(Tall::Star),
            4 => PieceType::Stack(Tall::Hand),
            5 => PieceType::Stack(Tall::Blind),
            6 => PieceType::Stack(Tall::Star),
            _ => unreachable!()
        }
    }

    pub fn positional_weight(&self) -> f32{
        const FLAT_POS_WEIGHT : f32 = 1.0;
        const TALL_POS_WEIGHT : f32 = 0.1;

        match self{
            PieceType::Flat => FLAT_POS_WEIGHT,
            PieceType::Lone(..) => TALL_POS_WEIGHT,
            PieceType::Stack(..) => FLAT_POS_WEIGHT + TALL_POS_WEIGHT,
        }
    }

    pub fn to_lone(&self) -> PieceType{
        match self{
            PieceType::Flat | PieceType::Lone(..) => *self,
            PieceType::Stack(tall) => PieceType::Lone(*tall)
        }
    }
}

#[derive(Clone, Copy,PartialEq, Eq,Hash, Debug)]
pub struct Piece{
    pub color : Player,
    pub species : PieceType
}


impl Piece{
    pub fn draw(&self, x : f32, y: f32, piece_tex : Texture2D, scale: f32){
        
        // let col = self.color.to_color();
        // let outcol = self.color.flip().to_color();
        
        let sx_single = match self.species{
            PieceType::Flat => 0,
            PieceType::Lone(tall) | PieceType::Stack(tall) => match tall{
                Tall::Hand => 1,
                Tall::Star => 2,
                Tall::Blind => 3
            } 
        };

        let sx = sx_single;

        let sy = match self.color{
            Player::Black => 2,
            Player::White => 0
        } + match self.species{
            PieceType::Stack(..) => 1,
            _ => 0
        };

        let tile_size = 128.0;
        let world_size = 1.7 * scale;

        let sx = sx as f32;
        let sy = sy as f32;
        draw_texture_ex(piece_tex, 
            x - world_size * 0.5, y - world_size * 0.5,
                WHITE, DrawTextureParams{
            dest_size : Some(vec2(1.0, 1.0) * world_size),
            source : Some(Rect{x:sx*tile_size,y: sy*tile_size,w:tile_size,h:tile_size}),
            ..Default::default()
            });
        

    }

    pub fn attack(&self) -> u8{
        match self.species {
            PieceType::Flat | PieceType::Lone(..) => 1,
            PieceType::Stack(..) => 1
        }
    }

    pub const fn defence(&self) -> u8{
        2
        // match self.species{
        //     PieceType::Flat | PieceType::Lone(..) => 2,
        //     PieceType::Stack(..) => 2
        // }
    }

    pub fn unstack(self) -> Box<dyn Iterator<Item=Piece>>{
        let color = self.color;
        match self.species{
            PieceType::Stack(tall) => Box::new([
                Piece{color, species: PieceType::Flat},
                Piece{color, species:PieceType::Lone(tall)},
            ].into_iter()),
            _ => Box::new([self].into_iter())
        }
    }

    pub fn value(&self) -> f32{
        self.species.value()
    }
}



#[derive(Clone, Copy,PartialEq,Eq, Hash, Debug)]
pub struct Ply{
    pub from_tile : Tile,
    pub to_tile : Tile
}

impl Ply{
    pub fn draw(&self, flip_board : bool){
        let (from_tile,to_tile) = (self.from_tile,self.to_tile) ; 

        draw_arrow(
            from_tile.to_world(flip_board).into(),
                to_tile.to_world(flip_board).into(),
                GREEN, 
                0.1,
                0.4,
                0.4
            );
        
    }

}


impl Display for Ply{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}{}",self.from_tile,self.to_tile)
        
    }
}



#[derive(Clone)]
pub struct BoardMap<T : Clone>{
    // data : [Option<T>;BOARD_SIZE],
    data : HashMap<Tile,T>
}

impl<T : Clone> BoardMap<T>{
    #[inline]
    pub fn new() -> BoardMap<T>{
        // BoardMap{data : [const { None };BOARD_SIZE]}
        BoardMap{data : HashMap::new()}
    }

    #[inline]
    pub fn get(&self, key : &Tile) -> Option<&T>{
        // self.data[key.code() as usize].as_ref()
        self.data.get(key)
    } 

    
    pub fn get_by_code(&self, code : u8) -> Option<&T>{
        // self.data[code as usize].as_ref()
        self.data.get(&Tile::from_code(code))
    }

    #[inline]
    pub fn get_mut(&mut self, key : Tile) -> Option<&mut T>{
        // &mut self.data[key.code() as usize]
        self.data.get_mut(&key)
    }

    #[inline]
    pub fn insert(&mut self, key : Tile, value : T) -> Option<T>{
        // let ptr = self.get_mut(key);
        // std::mem::replace(ptr, Some(value))
        self.data.insert(key, value)
    }

    #[inline]
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_,Tile, T>{
        // self.data.iter().enumerate()
        // .map(|(i,ov)| match ov{
        //     None => None,
        //     Some(v) => Some((Tile::from_code(i as u8),v))
        // })
        // .flatten()
        
        self.data.iter()
    }

    #[inline]
    pub fn remove(&mut self, key : &Tile) -> Option<T>{
        self.data.remove(key)
    }

    pub fn remove_by_code(&mut self, _code : u8) -> Option<T>{
        todo!()
        // let ptr = &mut self.data[code as usize];
        // std::mem::replace(ptr, None)
    }


    #[inline]
    pub fn extract_if<F>(&mut self, predicate : F) -> ExtractIf<'_,Tile,T,F>
        where F : FnMut(&Tile, &mut T) -> bool {
            self.data.extract_if(predicate)
        }
    #[inline]
    pub fn entry(&mut self, key : Tile) -> Entry<'_, Tile, T>{
        self.data.entry(key)
    }
}



const ZOBRIST_TABLE_SIZE : usize = BOARD_SIZE * PieceType::NUM_VALUES * 2 + 1;

lazy_static! {
    static ref ZOBRIST_SALT : [u64; ZOBRIST_TABLE_SIZE] = {
        let mut table = [0; ZOBRIST_TABLE_SIZE];
        let mut rng = ::rand::thread_rng();
        (0..ZOBRIST_TABLE_SIZE).for_each(|i|{
            table[i] = rng.gen::<u64>();
        });
        table
    };
}


#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct ZobristHash(u64);

impl ZobristHash{
    pub const CLEAR : ZobristHash = ZobristHash(0);


    pub fn toggle_piece(&mut self, tile : &Tile, color : Player, species : PieceType){
        let piece = Piece { color, species};
        self.toggle_piece_old(tile, &piece);
    }

    #[inline]
    pub fn toggle_piece_old(&mut self, tile : &Tile, piece : &Piece){
        let index = 2*(
            (piece.species.code() as usize) * BOARD_SIZE +
            (tile.code() as usize)
        ) + match piece.color{
            Player::White => 0,
            Player::Black => 1
        };

        self.0 ^= ZOBRIST_SALT[index];
    }

    pub fn toggle_to_move(&mut self){
        self.0 ^= ZOBRIST_SALT[ZOBRIST_TABLE_SIZE-1];
    }
}


#[derive(PartialEq,Eq,Debug,Clone)]
pub struct Captured([u8;7]);
impl Captured{
    pub fn empty()->Captured{
        Captured([0;7])
    }
    #[inline]
    pub fn push(&mut self, pt : PieceType){
        let idx = pt.code();
        self.0[idx as usize] += 1;
    }

    pub fn iter_counts(&self) -> impl Iterator<Item = (PieceType,u8)> + '_{
        self.0.iter().enumerate()
        .filter(|(_,count)|**count > 0)
        .map(|(code , count)| 
            (PieceType::from_code(code as u8),*count)
        )
    }
    pub fn iter(&self) -> impl Iterator<Item = PieceType> + '_{
        self.0.iter().enumerate()
        .flat_map(|(code,count)| {
            let pt = PieceType::from_code(code as u8);
            std::iter::repeat_n(pt, *count as usize)
        })
    }

    pub fn count(&self) -> usize{
        self.0.into_iter().map(|c|c as usize).sum()
    }

    pub fn len(&self) -> usize{
        self.0.iter().filter(|&&count|count > 0)
        .count()
    }

    pub fn extend(&mut self, iterator : impl IntoIterator<Item = PieceType>){
        iterator.into_iter().for_each(|pt| self.push(pt))
    }
}

#[derive(Copy,Clone,PartialEq, Eq)]
pub struct BitSet(u64);

impl BitSet{
    pub fn new()->BitSet{
        BitSet(0)
    }

    #[inline]
    const fn tile_to_bit(tile : &Tile) -> u8{
        tile.uy() | (tile.ux() << 3)
    }

    #[inline]
    const fn bit_to_tile(bit : u8) -> Tile{
        Tile::from_uxy(bit >> 3, bit &0b111)
    } 

    #[inline]
    const fn tile_mask(tile : &Tile) -> u64{
        1<<Self::tile_to_bit(tile)
    }

    pub const fn intersection(self, other : BitSet) -> BitSet{
        BitSet(self.0 & other.0)
    }

    pub fn set(&mut self, location : &Tile){
        self.0 |= Self::tile_mask(location)
    }

    pub fn unset(&mut self, location : &Tile){
        self.0 &= !Self::tile_mask(location)
    }

    pub fn remove(&mut self, location : &Tile) -> bool{
        let mask = Self::tile_mask(location);
        let removed = self.0 & mask > 0;
        self.0 &= !mask;
        removed
    }

    const BITS : [u8;29] = [
          02,03,04,05,06,
         09,10,11,12,13,14,
        16,17,18,19,20,21,22,
         24,25,26,27,28,29,
          32,33,34,35,36,
    ];

    pub fn into_iter(self) -> impl Iterator<Item = Tile>{
        Self::BITS.into_iter().flat_map(move |bit|{
            if self.0 & (1<<bit) > 0{
                Some(Self::bit_to_tile(bit))
            } else {None}
        })
    }
}


#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_tiles(){
        let sbr = BOARD_RADIUS as i8;
        (-sbr..=sbr).for_each(|y:i8|
            (-sbr..=sbr).for_each(|x:i8|{
                let z = -x-y;
                

                if let Some(t) = Tile::from_xyz(x, y, z){
                    assert_eq!(t.x(),x);
                    assert_eq!(t.y(),y);
                    assert_eq!(t.z(),z);
                }
            })
        );
        
        // (0..BOARD_SIZE).for_each(|code|
        //     {
        //         let code = code as u8;
        //         let tile = Tile::from_code(code);
        //         let compute_code = tile.code();

        //         assert_eq!(code, compute_code);

        //         let rebuild = Tile::from_code(compute_code);

        //         assert_eq!(tile,rebuild);
        //     }
        // );
    }

    #[test]
    fn test_3bit(){
        (0..4).for_each(|u|{
            assert_eq!(u, u3_to_i3(u) as u8);
            assert_eq!(u as i8, u3_to_i3(u));
        });

        assert_eq!(i3_to_u3(-4),4);
        assert_eq!(u3_to_i3(4),-4);

        (-4..4).for_each(|i|{
            assert_eq!(i, u3_to_i3(i3_to_u3(i)));
        });
    }

    #[test]
    fn test_delta(){
        assert_eq!(Delta::from_xyz(1, 0, -1).0,1);
        assert_eq!(Delta::from_xyz(-1, 0, 1).0,7);
        assert_eq!(Delta::from_xyz(0, 1, -1).0,0x10);
        assert_eq!(Delta::from_xyz(0, -1, 1).0,0xF0);

        (-2..=2).cartesian_product(-3..=3)
        .flat_map(|(x,y)|{
            Tile::from_xyz(x, y, -x-y)
        })
        .for_each(|t|{

            (-2..=2).cartesian_product(-2..=2)
            .map(|(x,y)|(x,y,-x-y))
            .filter(|(_,_,z)|(-2..=2).contains(z))
            .for_each(|(dx,dy,dz)|{
                let delta = Delta::from_xyz(dx, dy, dz);

                assert_eq!(dx,delta.dx());
                assert_eq!(dy,delta.dy());
                assert_eq!(dz,delta.dz());

                let transl = t.shift(delta);
                let transl_manual = Tile::from_xyz(
                    t.x()+dx, 
                    t.y()+dy, 
                    t.z() +dz);

                assert_eq!(transl,transl_manual);

            });
        }); 


    }


    #[test]
    fn test_in_board(){
        (-2..=2).cartesian_product(-3..=3)
        .for_each(|(x,y)|{
            let tile_xyz = Tile::from_xyz(x, y, -x-y);

            

            if let Some(tile_xyz) = tile_xyz{

                assert_eq!(Tile::from_uxy(tile_xyz.ux(), tile_xyz.uy()), tile_xyz);
                
                if Tile::new(tile_xyz.0).is_none(){
                    panic!("Tile x,y = {},{} is some from_xyz (value ${:02X}) but none on new.",x,y,tile_xyz.0);
                }
            }
        });


        (0..=255).for_each(|u|{
            let uu = u & 0b01110111;

            if let Some(tile) = Tile::new(uu){
                assert!(Tile::from_xyz(tile.x(), tile.y(), tile.z()).is_some())
            }
        });
    }

    #[test]
    fn test_bitsets(){
        (-2..=2).cartesian_product(-3..=3)
        .for_each(|(x,y)|{
            let tile_xyz = Tile::from_xyz(x, y, -x-y);

            if let Some(tile_xyz) = tile_xyz{
                let bit = BitSet::tile_to_bit(&tile_xyz);
                if !BitSet::BITS.contains(&bit){
                    panic!("tile {} ({}/{}) maps to bit ${:02X} which is invalid.", 
                    tile_xyz,
                    tile_xyz.ux(),
                    tile_xyz.uy(),
                    bit)
                };
            }
        });

        // BitSet::BITS.iter().for_each(|b|{
        //     assert!(BitSet::bit_to_tile(b))
        // });
    }
}