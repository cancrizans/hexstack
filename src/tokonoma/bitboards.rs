use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use crate::Tall;

use super::{Player, Species, Tile, BOARD_SIZE, ROW_OFFSET};


pub const BOARD_BITS : [u8;BOARD_SIZE] = {
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

#[inline]
pub const fn tile_to_bit(tile : &Tile) -> u8{
    tile.to_bit()
}
#[inline]
pub const fn bit_to_tile(bit : u8) -> Tile{
    Tile::from_bit_unchecked(bit)
}

#[derive(Copy,Clone,PartialEq, Eq,Hash, Debug)]
pub struct BitSet(u64);

impl BitSet{
    const EMPTY : BitSet = BitSet(0);
    pub const fn empty()->BitSet{
        Self::EMPTY
    }

    #[inline]
    pub const fn count(&self) -> u32{
        self.0.count_ones()
    }

    

    #[inline]
    const fn bit_mask(bit : u8) -> BitSet{
        BitSet(1<<bit)
    }

    #[inline]
    pub const fn tile_mask(tile : &Tile) -> BitSet{
        Self::bit_mask(tile_to_bit(tile))
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

    

    pub const BOARD_MASK : BitSet = {
        let mut mask = BitSet::empty();

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
                Some(bit_to_tile(bit))
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

        let mut buffer = BitSet::empty();
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
        Self{bitplane0:BitSet::empty(),bitplane1:BitSet::empty()}
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
        flats : BitSet::empty(),
        talls : [BitSet::empty(), BitSet::empty()]
    };

    /// Flip 180°
    /// Not efficient, not meant for runtime
    pub fn flip(self) -> PieceMap{
        let mut flipped = PieceMap::EMPTY;
        for (t,p) in self.into_iter(){
            flipped.set(t.antipode(), p);
        };
        flipped
    }

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
            .map(|sp|(bit_to_tile(bit),sp))
            
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

    /// Value at bit position guaranteed in 0..=7 range.
    #[inline]
    pub fn get_3bit(&self, bit : u8) -> u8{
        let mask = BitSet::bit_mask(bit);
        let (flat,tall0,tall1) = (
            (self.flats & mask).is_not_empty(),
            (self.talls[0] & mask).is_not_empty(),
            (self.talls[1] & mask).is_not_empty()
        );


        (if flat {1} else {0})
        | (if tall0 {2} else {0})
        | (if tall1 {4} else {0})

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
    use itertools::Itertools;

    use super::*;
    use super::super::Tile;
    #[test]
    fn test_bitsets(){
        (-2..=2).cartesian_product(-3..=3)
        .for_each(|(x,y)|{
            let tile_xyz = Tile::from_xyz(x, y, -x-y);

            if let Some(tile_xyz) = tile_xyz{
                let bit = tile_to_bit(&tile_xyz);
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
            let nasty = bit_to_tile(*b);

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