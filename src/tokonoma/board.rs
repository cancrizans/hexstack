
use itertools::Itertools;
use macroquad::prelude::*;
use std::{fmt::Display, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not}};
use memoize::memoize;
use crate::{arrows::draw_arrow, assets::{get_assets_unchecked, get_pieceset_unchecked, Assets, CompositionMode, ASSETS}};



pub const BOARD_RADIUS : i8 = 3;
const BOARD_SHORT_RADIUS : i8 = 2;

const BOARD_SIZE : usize = 29;

const ROW_OFFSET : u8 = 12;
const BOARD_BITS : [u8;BOARD_SIZE] = {
    let mut bits = [0;29];

    let mut i = 0;
    let mut x = 0;
    while x < 5 {
        let mut y = 0;
        while y < 7 {
            let z = -5+(x as i32)+(y as i32);
            if (z >= -3) & (z<=3) {
                bits[i] = ROW_OFFSET * x + y;
                i+=1;
            
            }
            y+=1;
        }
        x+=1;
    }
    
    bits
};
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
    const fn ux(&self) -> u8{
        // self.0 & 0xF
        self.0 / ROW_OFFSET
    }

    #[inline]
    pub const fn x(&self) -> i8{
        (self.ux() as i8) - Tile::OFF_X
    }

    #[inline]
    const fn uy(&self) -> u8{
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

        draw_texture_ex(
            tex,
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
        match self.mod3(){
            0 => Color::from_hex(0xbbbbbb),
            1 => Color::from_hex(0x999999),
            2 => Color::from_hex(0xdddddd),
            _ => unreachable!()
        }
    }

    pub fn draw_board(flip_board : bool){

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

            const DARK_TILE : Tile = Tile::from_xyz_unchecked(0, -1, 1);
            const LIGHT_TILE : Tile = Tile::from_xyz_unchecked(0, 1, -1);
            let col = match player{
                Player::Black => DARK_TILE.tile_color(),
                Player::White => LIGHT_TILE.tile_color(),
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
        let font = ASSETS.get().unwrap().font;
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
        
        draw_texture_ex(tex, 
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

#[derive(Copy,Clone,PartialEq, Eq,Hash, Debug)]
pub struct BitSet(u64);

impl BitSet{
    pub const fn new()->BitSet{
        BitSet(0)
    }

    #[inline]
    pub const fn count(&self) -> u32{
        self.0.count_ones()
    }

    #[inline]
    const fn tile_to_bit(tile : &Tile) -> u8{
        tile.0
    }
    #[inline]
    const fn bit_to_tile(bit : u8) -> Tile{
        Tile(bit)
    }

    #[inline]
    pub const fn tile_mask(tile : &Tile) -> BitSet{
        BitSet(1<<Self::tile_to_bit(tile))
    }

    #[inline]
    pub const fn intersection(self, other : BitSet) -> BitSet{
        BitSet(self.0 & other.0)
    }

    #[inline]
    pub const fn union(self, other : BitSet) -> BitSet{
        BitSet(self.0 | other.0)
    }

    pub fn set(&mut self, location : &Tile){
        *self |= Self::tile_mask(location)
    }

    pub fn get(&self, location : &Tile) -> bool{
        (*self & Self::tile_mask(location)).is_not_empty()
    }

    pub const fn get_at_bit(&self, bit : u8) -> bool{
        match bit{
            0..64 => (self.0 & (1<<bit))>0,
            _ => false
        }
    }

    pub fn unset(&mut self, location : &Tile){
        *self &= !Self::tile_mask(location)
    }

    pub fn set_mask_bool(&mut self, flag : bool, mask : BitSet){
        if flag{
            *self |= mask
        } else {
            *self &= !mask
        }
    }

    pub fn remove(&mut self, location : &Tile) -> bool{
        let mask = Self::tile_mask(location);
        let removed = self.0 & mask.0 > 0;
        *self &= !mask;
        removed
    }

    

    const BOARD_MASK : BitSet = {
        let mut mask = BitSet::new();

        let mut i = 0;
        while i < 29{
            let bit = BOARD_BITS[i];
            mask.0 |= 1<<bit;
            i+=1;
        }

        mask
    };

    pub fn into_iter(self) -> impl Iterator<Item = Tile>{
        BOARD_BITS.into_iter().flat_map(move |bit|{
            if self.0 & (1<<bit) > 0{
                Some(Self::bit_to_tile(bit))
            } else {None}
        })
    }

    pub fn is_not_empty(&self) -> bool{
        self.0 > 0
    }

    pub fn generate_move_destinations(&self, color : Player, species : Species) -> BitSet{
        const M : i32 = ROW_OFFSET as i32;

        let shifts = match species {
            Species::Flat => [1,M,-M+1].iter(),
            Species::Lone(tall) 
            | Species::Stack(tall)
            => match tall{
                Tall::Hand => [2,-M,M-1,-1].iter(),
                Tall::Blind => [2*M,-2*M+2,-M,M-1,-2].iter(),
                Tall::Star => [-2*M+1,2*M-1,M+1,-M-1, M-2,-M+2].iter()
            }
        };

        let mut buffer = BitSet::new();
        for &shift in shifts{
            let signed_shift = match color{
                Player::White => -shift,
                Player::Black => shift
            };

            let shifted = if signed_shift > 0{
                self.0 >> signed_shift
            } else {
                self.0 << -signed_shift
            };

            buffer.0 |= shifted;
        };

        buffer & Self::BOARD_MASK
    }
    pub fn move_destinations_from_tile(from_tile : Tile,color : Player, species : Species) -> BitSet{
        Self::tile_mask(&from_tile).generate_move_destinations(color, species)
    }

}

impl BitOr for BitSet{
    type Output = BitSet;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}
impl BitAnd for BitSet{
    type Output = BitSet;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}
impl Not for BitSet{
    type Output = BitSet;
    fn not(self) -> Self::Output {
        BitSet(!self.0)
    }
}
impl BitAndAssign for BitSet{
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}
impl BitOrAssign for BitSet{
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

pub struct DoubleCounterBitset{
    bitplane1 : BitSet,
    bitplane0 : BitSet
}

impl DoubleCounterBitset{
    pub fn new()->Self{
        Self{bitplane0:BitSet::new(),bitplane1:BitSet::new()}
    }
    pub fn add(&mut self, mask : BitSet){
        let carry = self.bitplane0 & mask;
        self.bitplane1 |= carry;
        self.bitplane0 |= mask;
    }
    pub fn get_doubles(&self)->BitSet{
        self.bitplane1
    }
}


#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct PieceMap{
    flats : BitSet,
    talls : [BitSet; 2]
}

impl PieceMap{
    pub const EMPTY : PieceMap = PieceMap{
        flats : BitSet::new(),
        talls : [BitSet::new(), BitSet::new()]
    };

    pub fn occupied(&self) -> BitSet{
        self.flats | self.talls[0] | self.talls[1]
    }

    pub fn count(&self) -> u32{
        self.occupied().count()
    }

    pub fn locate_lone_flats(&self) -> BitSet{
        self.flats & !(self.talls[0] | self.talls[1])
    }

    pub fn viable_tall_destinations(&self) -> BitSet{
        !(self.talls[0] | self.talls[1])
    }


    pub fn into_iter(self) -> impl Iterator<Item = (Tile,Species)>{
        
        BOARD_BITS.into_iter().flat_map(move |bit|{
            let (flat,tall0,tall1) = (
                self.flats.get_at_bit(bit),
                self.talls[0].get_at_bit(bit),
                self.talls[1].get_at_bit(bit)
            );
            Self::decode_species(flat, tall0, tall1)
            .map(|sp|(BitSet::bit_to_tile(bit),sp))
            
        })
    }

    #[inline]
    const fn encode_species(species : Species) -> (bool,bool,bool){
        let flat = match species{
            Species::Flat | Species::Stack(..) => true,
            Species::Lone(..) => false
        };

        let (t0,t1) = match species{
            Species::Flat => (false, false),
            Species::Lone(tall) | Species::Stack(tall) => 
            Self::encode_tall(tall)
        };

        (flat,t0,t1)
    }

    #[inline]
    const fn decode_tall(tall0 : bool, tall1 : bool) -> Option<Tall>{
        match (tall0,tall1){
            (false,false) => None,
            (false,true) => Some(Tall::Hand),
            (true,false) => Some(Tall::Blind),
            (true, true ) =>Some( Tall::Star)
        }
    }

    #[inline]
    const fn encode_tall(tall : Tall) -> (bool, bool){
        match tall{
            Tall::Hand => (false,true),
            Tall::Blind => (true, false),
            Tall::Star => (true, true)
        }
    }

    #[inline]
    const fn decode_species(flat : bool, tall0 : bool, tall1 : bool) -> Option<Species>{
        let tall = Self::decode_tall(tall0, tall1);
        if flat{
            if let Some(tall) = tall{
                Some(Species::Stack(tall))
            } else {
                Some(Species::Flat)
            }
        } else {
            if let Some(tall) = tall{
                Some(Species::Lone(tall))
            } else {
                None
            }
        }
    }

    pub fn set(&mut self, location : Tile, species : Species){
        let (sf, st0, st1) = Self::encode_species(species);

        let mask = BitSet::tile_mask(&location);

        self.flats.set_mask_bool(sf, mask);
        self.talls[0].set_mask_bool(st0, mask);
        self.talls[1].set_mask_bool(st1, mask);
    }

    pub fn contains_key(&self, key : &Tile) -> bool{
        let mask = BitSet::tile_mask(key);
        (self.occupied() & mask).is_not_empty()
    }

    #[inline]
    pub fn locate_talls(&self, tall : Tall) -> BitSet{
        let (tall0,tall1) = Self::encode_tall(tall);

        match (tall0,tall1){
            (true,true) => self.talls[0] & self.talls[1],
            (true,false) => self.talls[0] & !self.talls[1],
            (false,true) => !self.talls[0] & self.talls[1],

            (false,false) => unreachable!()
        }
    }


    pub fn locate_species(&self, species : Species) -> BitSet{
        // Note: this is maybe inefficient because
        // it is called twice for lones and stacks.

        match species{
            Species::Flat => self.locate_lone_flats(),
            Species::Lone(tall) => {
                self.locate_talls(tall) & !self.flats
            },
            Species::Stack(tall) => {
                self.locate_talls(tall) & self.flats
            }
        }

    }
    

    pub fn pull_moving_piece(&mut self, location : Tile) -> Species{
        let mask = BitSet::tile_mask(&location);
        let orig_flat = (self.flats & mask).is_not_empty();

        let orig_tall_bits = self.talls.map(|bf|(bf & mask).is_not_empty());

        let orig_tall = Self::decode_tall(orig_tall_bits[0], orig_tall_bits[1]);

        if orig_flat{
            if let Some(tall) = orig_tall{
                self.talls[0].set_mask_bool(false,mask);
                self.talls[1].set_mask_bool(false,mask);
                Species::Lone(tall) 
            } else {
                self.flats.set_mask_bool(false, mask);
                Species::Flat
            }
        } else {
            if let Some(tall) = orig_tall{
                self.talls[0].set_mask_bool(false,mask);
                self.talls[1].set_mask_bool(false,mask);
                Species::Lone(tall)
            } else {
                unreachable!()
            }
        }
    }

    pub fn mask(&self, mask : BitSet) -> Self{
        PieceMap {
            flats : self.flats & mask,
            talls : [
                self.talls[0] & mask,
                self.talls[1] & mask
            ]
        }
    }

    pub fn kill(&mut self, mask : BitSet) -> PieceMap{
        let kills_masked = self.mask(mask);

        *self = self.mask(!mask);

        kills_masked
    }

    pub fn is_not_empty(&self) -> bool{
        self.flats.is_not_empty() | self.talls[0].is_not_empty() | self.talls[1].is_not_empty()
    }

    pub fn toss(&mut self, location : Tile, piece : Species){
        let mask = BitSet::tile_mask(&location);

        let flat = (self.flats & mask).is_not_empty();

        match piece{
            Species::Flat => {
                assert!(!flat);
                self.flats.set_mask_bool(true, mask);
                
            },
            Species::Lone(tall) => {
                let (tall0, tall1) = Self::encode_tall(tall);
                self.talls[0].set_mask_bool(tall0, mask);
                self.talls[1].set_mask_bool(tall1, mask);
                
            },
            Species::Stack(..) => unreachable!()
        
        };

    }

    #[inline]
    pub fn get(&self, location : Tile) -> Option<Species>{
        let mask = BitSet::tile_mask(&location);
        let (flat,tall0,tall1) = (
            (self.flats & mask).is_not_empty(),
            (self.talls[0] & mask).is_not_empty(),
            (self.talls[1] & mask).is_not_empty()
        );

        Self::decode_species(flat, tall0, tall1)

    }

    pub fn clear_tile(&mut self, tile : Tile){
        let mask = !BitSet::tile_mask(&tile);

        self.flats &= mask;
        self.talls[0] &= mask;
        self.talls[1] &= mask;
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
                if !BOARD_BITS.contains(&bit){
                    panic!("tile {} ({}/{}) maps to bit {} which is invalid.", 
                    tile_xyz,
                    tile_xyz.ux(),
                    tile_xyz.uy(),
                    bit)
                };
            }
        });

        BOARD_BITS.iter().for_each(|b|{
            let nasty = BitSet::bit_to_tile(*b);

            assert!(Tile::from_xyz(nasty.x(), nasty.y(), nasty.z()).is_some())
        });
    }

    #[test]
    fn test_piecemaps(){
        // let pmap = PieceMap::EMPTY;

        [
            Tall::Blind, Tall::Star, Tall::Hand
        ].into_iter().for_each(|t|{
            let (a,b) = PieceMap::encode_tall(t);
            let t2 = PieceMap::decode_tall(a,b).unwrap();
            assert_eq!(t,t2);
        });
    }
}