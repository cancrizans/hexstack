use std::fmt::Display;
use std::{env, fs::File, io::Write, path::Path};
use std::collections::{HashMap,HashSet};
use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tile{
    x : i8,
    y : i8
}

impl Tile{
    pub const fn from_xyz(x:i8,y:i8,z:i8)->Tile{
        if x+y+z != 0 {
            panic!("Incorrect axial coords.")
        }
        Tile{x,y}
    }

    pub const ORIGIN : Tile = Self::from_xyz(0, 0,0);

    fn z(&self) -> i8{
        -self.x-self.y
    }

    pub fn in_board(&self) -> bool{
        let sbr = BOARD_RADIUS as i8;
        let range = -sbr..=sbr;
        range.contains(&self.x) 
        & range.contains(&self.y) 
        & range.contains(&self.z())
    }

    fn antipode(&self) -> Tile{
        Tile{x:-self.x,y:-self.y}
    }

    fn all_neighbours(&self, kind : EdgeType) -> Vec<Tile>{
        let (x,y,z) = (self.x, self.y, self.z());

        match kind{
            EdgeType::WhiteFlat => [
                Tile::from_xyz(x+1, y-1, z),
                Tile::from_xyz(x, y+1, z-1),
                Tile::from_xyz(x-1, y, z+1),
            ].into(),

            EdgeType::BlackFlat => [
                Tile::from_xyz(x-1, y+1, z),
                Tile::from_xyz(x, y-1, z+1),
                Tile::from_xyz(x+1, y, z-1),
            ].into(),

            EdgeType::Diagonal => [
                (x-2, y+1, z+1),
                (x+1, y-2, z+1),
                (x+1, y+1, z-2),
                (x+2, y-1, z-1),
                (x-1, y+2, z-1),
                (x-1, y-1, z+2),
                
            ].map(|(x,y,z)|Tile::from_xyz(x, y, z)).into()
        }
    }

    pub fn neighbours(&self, kind : EdgeType) -> impl Iterator<Item = Tile>{
        self.all_neighbours(kind).into_iter().filter(|n|n.in_board())
    }

    pub fn to_world(&self) -> (f32,f32){
        const SQRT3 : f32 = 1.73205080757;
        const SQRT3_2 : f32 = 0.86602540378;
        (1.5* (self.x as f32) ,
                  SQRT3_2 * ( self.x as f32) +               SQRT3 * (self.y as f32))
    }

    pub fn mod3(&self) -> u8{
        (self.x-self.y).rem_euclid(3) as u8
    }
}

impl Display for Tile{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"({},{})",self.x,self.y)
    }
}

const BOARD_RADIUS : u8 = 4;
#[derive(Clone, Copy,PartialEq, Eq, Hash)]
pub enum EdgeType{
    WhiteFlat,
    BlackFlat,
    Diagonal
}

impl EdgeType{
    // pub fn to_color(&self) -> Color{
    //     match self{
    //         Self::BlackFlat => BLACK,
    //         Self::WhiteFlat => WHITE,
    //         Self::Diagonal => RED
    //     }
    // }
}

pub struct Board{
    tiles : HashSet<Tile>,
    edges : HashSet<(EdgeType, Tile, Tile)>,
    neighbours : HashMap<(Tile,EdgeType), Vec<Tile>>
}

impl Board{
    pub fn build() -> Board{
        let range = -(BOARD_RADIUS as i8)..=BOARD_RADIUS as i8;
        let tiles = HashSet::from_iter(
            range.clone().cartesian_product(range)
            .map(|(x,y)|Tile::from_xyz(x, y, -x-y))
            .filter(|t|t.in_board())
            
        );

        let mut edges = HashSet::new();
        let mut neighbours = HashMap::new();

        tiles.iter().for_each(|&t|{
            [EdgeType::Diagonal,EdgeType::BlackFlat,EdgeType::WhiteFlat].into_iter().for_each(|et|{
                t.neighbours(et).for_each(|n|{
                    edges.insert((et,t,n));

                });
                
            })
        });

        Board{tiles,edges,neighbours}
    }
    
    // pub fn draw(&self){

    //     // self.edges.iter().for_each(|(et,t,n)|{
    //     //     let (x1,y1) = t.to_world();
    //     //     let (x2,y2) = n.to_world();
    //     //     let (xm,ym) = (0.5*(x1+x2),0.5*(y1+y2));
    //     //     draw_line(x1, y1, xm, ym, 0.1, et.to_color());
    //     // });
    //     self.tiles.iter().for_each(|t|{
    //         let (x,y) = t.to_world();

    //         let tile_color = match t.mod3(){
    //             0 => Color::from_hex(0x888888),
    //             1 => Color::from_hex(0xaaaaaa),
    //             2 => Color::from_hex(0xcccccc),
    //             _ => unreachable!()
    //         };

    //         draw_hexagon(x, y, 
    //             1.0, 
    //             0.05, 
    //             false,
    //             BLACK,
    //             tile_color);
    //     });

        
    // }

    pub fn all_tiles(&self) -> impl Iterator<Item = &Tile>{
        self.tiles.iter()
    }
}


fn build_tile(){
    

    let board = Board::build();
    let count = board.tiles.len();

    let mut impl_code = String::new();

    impl_code.push_str(&format!(
        "const WORLD_POS : [(f32,f32);{}] = [{}];",
        count,
        board.tiles.iter().map(|t|t.to_world())
        .map(|(x,y)|format!("({:.},{:.})",x,y))
        .join(",")
    ));

    impl_code.push_str(&"
        pub fn to_world(&self) -> (f32,f32){
            Self::WORLD_POS[self.0]
        }
    ");

    impl_code.push_str(&format!(
        "pub fn from_xyz(x:i8,y:i8,z:i8)->Tile{{
            if x+y+z != 0 {{
                panic!(\"Incorrect axial coords.\")
            }}

            match (x,y){{
                {}
                _ => unreachable!()
            }}
        }}",

        board.tiles.iter().enumerate().map(|(i,t)|
            format!("({},{}) => Tile({}),",t.x,t.y,i as u8)
        ).join("\n")
    ));

    let codestring = format!("
        #[derive(PartialEq,Eq,Hash,Copy,Clone,Debug,Display)]
        pub struct Tile(u8);
        impl Tile{{
            {}
        }}

    ",impl_code);
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("hexboard.rs");
    let mut f = File::create(dest_path).unwrap();
    f.write_all(codestring.as_bytes()).unwrap()
}

fn main(){
    build_tile();
}