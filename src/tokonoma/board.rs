
use itertools::Itertools;
use macroquad::prelude::*;
use std::{fmt::Display, ops::{Index, IndexMut}, str::FromStr};
use memoize::memoize;
use crate::{arrows::draw_arrow, assets::{get_assets_unchecked, get_pieceset_unchecked, CompositionMode}, theme::get_board_palette};
use super::bitboards::BitSet;


pub const BOARD_RADIUS : i8 = 3;
const BOARD_SHORT_RADIUS : i8 = 2;

pub const BOARD_SIZE : usize = 29;
pub const ROW_OFFSET : u8 = 12;


// #[inline]
// const fn tile_to_bit(tile : &Tile) -> u8{
//     tile.uy() + tile.ux() * Self::ROW_OFFSET
// }

// const ROW_OFFSET : u8 = ROW_OFFSET;

// #[inline]
// const fn bit_to_tile(bit : u8) -> Tile{
//     Tile::from_uxy(bit / Self::ROW_OFFSET, bit % Self::ROW_OFFSET)
// } 



#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Tile(u8);
impl Tile{
    
    const OFF_Y : i8 = BOARD_RADIUS;
    const OFF_X : i8 = BOARD_SHORT_RADIUS;

    #[inline]
    pub const fn to_bit(&self) -> u8{
        self.0
    }

    #[inline]
    pub const fn from_bit_unchecked(bit : u8) -> Tile{
        Tile(bit)
    }

    #[inline]
    pub const fn new(value : u8) -> Option<Tile>{
        if BitSet::BOARD_MASK.get_at_bit(value){
            Some(Tile(value))
        } else {
            None
        }
        // //assumption that value already has bits 3 and 7 unset.
        // let ux_hi = value << 4;

        // if !(ux_hi < 0x50) {
        //     return None
        // }
        // if !(value < 0x70){
        //     return None
        // }

        // // this is 5-z placed in high nibble
        // let uz_sh = (value & 0xF0) + ux_hi;
        // if ! ((0x20 <= uz_sh) & (uz_sh <= 0x80)){
        //     return None
        // }

        
        // Some(Tile(value))
        
    }

    #[inline]
    pub const fn from_uxy(ux : u8, uy : u8)-> Tile{
        // Tile((uy << 4) | ux)

        Tile(ux * ROW_OFFSET + uy)
    }

    #[inline]
    pub const fn code(&self) -> u8{
        self.0
        // let (ux, uy) = (self.ux(),self.uy());

        // let shift = match ux{
        //     4 => 0,
        //     3 => 5,
        //     2 => 5+6,
        //     1 => 5+6+7 - 1,
        //     0 => 5+6+7+6 - 2,
            
        //     _ => unreachable!()
        // };

        // shift + uy
    }

    

    #[inline]
    pub const fn from_code(code : u8) -> Tile{
        Tile(code)
        // let (ux,shift) = match code{
        //     0..5 =>  (5,0),
        //     5..11 => (4,5),
        //     11..18 => (3,11),
        //     18..24 => (2,17),
        //     24..29 => (1,22),
        //     _ => unreachable!()
        // };
        // let uy = code - shift;

        // Tile((uy << 4) | ux)
    }

    #[inline]
    pub const fn ux(&self) -> u8{
        // self.0 & 0xF
        self.0 / ROW_OFFSET
    }

    #[inline]
    pub const fn x(&self) -> i8{
        (self.ux() as i8) - Tile::OFF_X
    }

    #[inline]
    pub const fn uy(&self) -> u8{
        // self.0 >> 4
        self.0 % ROW_OFFSET
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
            Species::Flat => [
                Some(Delta::WH_FORWARD), 
                Some(Delta::WH_FRONTDOWN),
                Some(Delta::WH_FRONTUP),
                None, None, None
            ],
            
            Species::Lone(tall) | Species::Stack(tall)
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

    pub const fn mod3(&self) -> u8{
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

    pub fn draw_move_target(&self, color : Player,  flip_board : bool){
        let tex = get_pieceset_unchecked().tex;
        
        let (x,y) = self.to_world(flip_board);
        const R : f32 = 1.0;
        let src_off = match color{
            Player::White => 0.0,
            Player::Black => 1.0
        };

        tex.draw(
            x-R,
            y-R,
            WHITE, DrawTextureParams{
                dest_size : Some(vec2(2.0*R,2.0*R)),
                source : Some(Rect::new(0.0,128.0*(1.0 + 2.0*src_off),128.0,128.0)),
                ..Default::default()
            }
        )
        
    }

    fn tile_color(&self) -> Color{
        get_board_palette().sample(self.mod3())
    }

    const fn is_border(&self) -> bool{
        let (ux,uy) = (self.ux(),self.uy());

        match ux{
            0|4 => true,
            1  => (uy == 6) | (uy == 1),
            2 => (uy==6) | (uy == 0),
            3 => (uy==5) | (uy == 0),
            _ => unreachable!()
        }
    }

    pub fn draw_board(flip_board : bool){
        const DARK_TILE : Tile = Tile::from_xyz_unchecked(0, -1, 1);
        const LIGHT_TILE : Tile = Tile::from_xyz_unchecked(0, 1, -1);
        let dark_color = DARK_TILE.tile_color();
        let light_color = LIGHT_TILE.tile_color();
        

        
        Self::all_tiles().filter(|t|t.is_border())
        .for_each(|t|{
            let (x,y) = t.to_world(flip_board);
            draw_hexagon(x, y, 
                1.1, 
                0.0,//0.05, 
                true,
                Color::from_hex(0x111111),
                dark_color);
        });

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

        

        for player in [Player::Black,Player::White]{
            let (x,y) = Tile::corner(player).to_world(flip_board);

            
            let col = match player{
                Player::Black => dark_color,
                Player::White => light_color,
            };

            draw_hexagon(x, y, 
                0.6, 
                0.0,
                true,
                Color::from_hex(0x111111),
                col);
        }
    }

    pub fn draw_tile_numbers( flip_board : bool){
        let font = get_assets_unchecked().font;
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
        // let value = self.0.wrapping_add(delta.0) & 0b11110111;
        // Tile::new(value)
        Tile::from_xyz(self.x()+delta.dx(), self.y()+delta.dy(), self.z()+delta.dz())
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
#[derive(Debug)]
pub struct TileParseError;
impl FromStr for Tile{
    type Err = TileParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {return Err(TileParseError)};
        let mut chars = s.chars();
        let (letter,number) = (chars.next().unwrap(),chars.next().unwrap());

        let row = match letter{
            'a' => -2, 'b' => -1, 'c' => 0, 'd' => 1, 'e' => 2,
            _ => {return Err(TileParseError)}
        };
        let x : i8 = row;

        let tile_nr = number.to_digit(10).ok_or(TileParseError)? as i8;

        let y = match row{
            -2..=0 => 4,
            1 => 3,
            2 => 2,
            _ => unreachable!()
        } - tile_nr;

        Ok(Tile::from_xyz_unchecked(x, y, -x-y))
        
    }
}



#[derive(Clone, Copy,PartialEq, Eq,Hash, Debug)]
/// Identifies the player / color.
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

#[derive(Clone,Debug, PartialEq, Eq, Hash)]
pub struct PlayerMap<T>{
    white : T,
    black : T,
}
impl <T> Index<Player> for PlayerMap<T>{
    type Output = T;
    #[inline]
    fn index(&self, index: Player) -> &Self::Output {
        match index{
            Player::White => &self.white,
            Player::Black => &self.black
        }
    }
}
impl <T> IndexMut<Player> for PlayerMap<T>{
    #[inline]
    fn index_mut(&mut self, index: Player) -> &mut Self::Output {
        match index{
            Player::White => &mut self.white,
            Player::Black => &mut self.black,
        }
    }
}
impl<T> PlayerMap<T> where T : Clone{
    pub fn twin(value : T) -> PlayerMap<T>{
        PlayerMap { white: value.clone(), black: value }
    }
}
impl<T> PlayerMap<T>{
    pub fn new(white : T, black : T) -> PlayerMap<T>{
        PlayerMap{white,black}
    }
    pub fn new_on_player(first_player : Player, first_value : T, other_value : T) -> Self{
        match first_player{
            Player::White => Self::new(first_value,other_value),
            Player::Black => Self::new(other_value,first_value),
        }
    }
}

pub struct PlayerMapIntoIterator<'a,T>{
    map : &'a PlayerMap<T>,
    player : Option<Player>
}

impl<'a, T> Iterator for PlayerMapIntoIterator<'a, T>{
    type Item = (Player,&'a T);
    fn next(&mut self) -> Option<Self::Item> {
        match self.player{
            Some(Player::White) => {
                self.player = Some(Player::Black);
                Some((Player::White,&self.map[Player::White]))
            },
            Some(Player::Black) => {
                self.player = None;
                Some((Player::Black,&self.map[Player::Black]))
            },
            None => {
                None
            }
        }
    }
}   
impl<'a, T> IntoIterator for &'a PlayerMap<T>{
    type Item = (Player,&'a T);
    type IntoIter = PlayerMapIntoIterator<'a,T>;
    fn into_iter(self) -> Self::IntoIter {
        PlayerMapIntoIterator{
            map : self,
            player : Some(Player::White)
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


/// Piece species.
#[derive(Clone, Copy,PartialEq, Eq,Hash, Debug)]
pub enum Species{
    Flat,
    Lone(Tall),
    Stack(Tall),
}

impl Species{
    pub fn unstack(self) -> Box<dyn Iterator<Item=Species>>{
        match self{
            Species::Stack(tall) => Box::new([
                Species::Flat,
                Species::Lone(tall),
            ].into_iter()),
            _ => Box::new([self].into_iter())
        }
    }

    

    pub fn value(&self) -> f32{
        match self {
            Species::Flat => {
                2.0
            },
            Species::Lone(tall) => tall.value(),
            Species::Stack(tall) => (Species::Flat.value() + tall.value()) - 0.2,
        }
    }

    #[inline]
    pub const fn code(&self) -> u8{
        match self{
            Species::Flat => 0,
            Species::Lone(tall) => match tall{
                Tall::Hand => 1,
                Tall::Blind => 2,
                Tall::Star => 3
            },
            Species::Stack(tall) => match tall{
                Tall::Hand => 4,
                Tall::Blind => 5,
                Tall::Star => 6
            }
        }
    }

    pub const fn from_code(code : u8) -> Species{
        match code {
            0 => Species::Flat,
            1 => Species::Lone(Tall::Hand),
            2 => Species::Lone(Tall::Blind),
            3 => Species::Lone(Tall::Star),
            4 => Species::Stack(Tall::Hand),
            5 => Species::Stack(Tall::Blind),
            6 => Species::Stack(Tall::Star),
            _ => unreachable!()
        }
    }

    pub fn positional_weight(&self) -> f32{
        const FLAT_POS_WEIGHT : f32 = 1.0;
        const TALL_POS_WEIGHT : f32 = 0.1;

        match self{
            Species::Flat => FLAT_POS_WEIGHT,
            Species::Lone(..) => TALL_POS_WEIGHT,
            Species::Stack(..) => FLAT_POS_WEIGHT + TALL_POS_WEIGHT,
        }
    }

    pub fn to_lone(&self) -> Species{
        match self{
            Species::Flat | Species::Lone(..) => *self,
            Species::Stack(tall) => Species::Lone(*tall)
        }
    }

    pub const ALL : [Species;7] = [
        Species::Flat, 
        Species::Lone(Tall::Hand),
        Species::Lone(Tall::Blind),
        Species::Lone(Tall::Star),
        Species::Stack(Tall::Hand),
        Species::Stack(Tall::Blind),
        Species::Stack(Tall::Star),
    ];
}

/// Largely deprecated structure, piece with color + species.
#[derive(Clone, Copy,PartialEq, Eq,Hash, Debug)]
pub struct Piece{
    pub color : Player,
    pub species : Species
}

impl Piece{
    pub fn draw(&self, x : f32, y: f32,  scale: f32){
        
        let pieceset = get_pieceset_unchecked();

        match pieceset.composition_mode {
            CompositionMode::ComposeOnFlat => {
                match self.species{
                    Species::Stack(tall) => {
                        Piece{color:self.color, species : Species::Flat}.draw(x, y, scale);
                        Piece{color:self.color, species : Species::Lone(tall)}.draw(x, y, scale);
                        return;
                    },
                    _ => {},
                }

            }
            CompositionMode::Precomposed => {},
        }

        let sx_single = match self.species{
            Species::Flat => 0,
            Species::Lone(tall) | Species::Stack(tall) => match tall{
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
            Species::Stack(..) => 1,
            _ => 0
        };

        
        
        let tex = pieceset.tex;

        let tile_size = tex.width() * 0.25;
        let world_size = pieceset.base_scale * scale;

        let sx = sx as f32;
        let sy = sy as f32;
        
        
        tex.draw(
            x - world_size * 0.5, y - world_size * 0.5,
                WHITE, DrawTextureParams{
            dest_size : Some(vec2(1.0, 1.0) * world_size),
            source : Some(Rect{x:sx*tile_size,y: sy*tile_size,w:tile_size,h:tile_size}),
            ..Default::default()
            });
        

    }

    pub fn unstack(self) -> Box<dyn Iterator<Item=Piece>>{
        let color = self.color;
        match self.species{
            Species::Stack(tall) => Box::new([
                Piece{color, species: Species::Flat},
                Piece{color, species:Species::Lone(tall)},
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
                Color::new(1.0, 1.0, 0.0, 0.5), 
                0.3,
                0.7,
                0.8
            );
        
    }

}


impl Display for Ply{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}{}",self.from_tile,self.to_tile)
        
    }
}

#[derive(Debug)]
pub enum PlyParseError{
    Tile(TileParseError),
    Other,
}
impl From<TileParseError> for PlyParseError{
    fn from(value: TileParseError) -> Self {
        PlyParseError::Tile(value)
    }
}
impl FromStr for Ply{
    type Err = PlyParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {return Err(PlyParseError::Other)};
        let (from,to) = (&s[0..2],&s[2..4]);
        
        Ok(Ply{
            from_tile : Tile::from_str(from)?,
            to_tile : Tile::from_str(to)?
        })
    }
}




#[derive(PartialEq,Eq,Debug,Clone)]
pub struct Captured([u8;7]);
impl Captured{
    pub fn empty()->Captured{
        Captured([0;7])
    }
    #[inline]
    pub fn push(&mut self, pt : Species){
        let idx = pt.code();
        self.0[idx as usize] += 1;
    }

    pub fn iter_counts(&self) -> impl Iterator<Item = (Species,u8)> + '_{
        self.0.iter().enumerate()
        .filter(|(_,count)|**count > 0)
        .map(|(code , count)| 
            (Species::from_code(code as u8),*count)
        )
    }
    pub fn iter(&self) -> impl Iterator<Item = Species> + '_{
        self.0.iter().enumerate()
        .flat_map(|(code,count)| {
            let pt = Species::from_code(code as u8);
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

    pub fn extend(&mut self, iterator : impl IntoIterator<Item = Species>){
        iterator.into_iter().for_each(|pt| self.push(pt))
    }

    pub fn draw(&self, color : Player){
        let capts = self;
        let n_capt = 0.5*(capts.count().saturating_sub(1) as f32);
        let basey = match color {Player::White => 4.7, Player::Black => -4.7};
        
        capts.iter().enumerate().for_each(|(i,piece_type)|{
            let p = Piece{color : color.flip(), species : piece_type};
            let x = 0.6*(i as f32 - n_capt);
            let y = basey;
            p.draw(x,y, 0.5);
            
            
        });
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

        Tile::all_tiles().for_each(|t|{
            
            let parsed = Tile::from_str(&format!("{}",t))
            .expect("Tile parse error");
            assert_eq!(t,parsed);
            

        });
        
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


}