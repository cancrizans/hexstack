use std::iter::once;
use circular_buffer::CircularBuffer;

use crate::{assets::{get_assets_unchecked, mipmaps::set_cam_from_cam2d}, theme, tokonoma::PositionString, ui::{draw_text_centered, MqUi}, Piece, Player, Position, Species, Tile};
use macroquad::prelude::*;

use super::Button;

const UNDO_HISTORY_SIZE : usize = 30;

#[derive(Clone)]
pub struct PositionEditor{
    state : Position,
    selected_brush : Option<Piece>,
    undo_history : CircularBuffer::<UNDO_HISTORY_SIZE,Position>,

    btn_undo : Button,
}

impl PositionEditor{
    

    pub fn setup() -> PositionEditor{
        Self::from_state(Position::setup())
    }

    pub fn from_state(position : Position)->PositionEditor{
        PositionEditor{
            state : position,
            selected_brush : None,
            undo_history : CircularBuffer::new(),
            btn_undo : Button::new(
                get_assets_unchecked().btn_takeback, 
                Rect::new(3.0,4.0,1.0,1.0),
            "Undo".to_string())
        }
    }

    pub fn tabulation_hash(&self)->u64{
        self.state.tabulation_hash()
    }

    fn push_history(&mut self){
        self.undo_history.push_back(self.state.clone());
    }

    fn can_undo(&self) -> bool{
        !self.undo_history.is_empty()
    }

    fn undo(&mut self){
        if let Some(previous) = self.undo_history.pop_back(){
            self.state = previous;
            
        } 
    }

    pub fn set_position(&mut self, new_position : Position){
        self.push_history();
        self.state = new_position;
    }

    fn process_palette_button(&mut self, position : Vec2, brush : Option<Piece>, camera : &Camera2D){
        let mouse =  mouse_position();
        let mouse_world = camera.screen_to_world(mouse.into());

        let highlighted = mouse_world.distance_squared(position) < 0.18;

        if highlighted & is_mouse_button_pressed(MouseButton::Left){
            self.selected_brush = brush;
        }

        let outline = if brush == self.selected_brush{
            Some(Color::from_hex(0x111111))
        } else {
            if highlighted{
                Some(Color::from_hex(0x666666))
            } else {None}
        };
        
        if let Some(col) = outline {
            draw_hexagon(position.x, position.y, 0.8, 0.1, 
                true, col, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 });
        };

        if let Some(piece) = brush{
            piece.draw(position.x, position.y,  0.9)
        } else {
            draw_line(position.x-0.3, position.y-0.3,position.x+0.3,position.y+0.3, 0.1, RED);
            draw_circle_lines(position.x, position.y, 0.3, 0.1, RED);
        }


        
    }

    pub fn process(&mut self, camera : &Camera2D){
        set_cam_from_cam2d(camera);
        let assets = get_assets_unchecked();

        
        draw_rectangle(-7.0, -8.0, 14.0, 16.0, theme::BG_COLOR);

        Tile::draw_board(false);

        self.state.draw(
            
            false, false, false);

        let mouse = mouse_position();
        let mouse_world = camera.screen_to_world(mouse.into());

        if let Some(hover_tile) = Tile::from_world(mouse_world.x, mouse_world.y, false){
            hover_tile.draw_highlight_outline(0.1,Color::from_hex(0x111111), false);

            if is_mouse_button_pressed(MouseButton::Left){
                self.push_history();
                self.state.paint(&hover_tile, self.selected_brush);
            }
        }

        once(None).chain(
            (0..7).map(|i|Species::from_code(i))
            .flat_map(|species|[
                Piece{color:Player::White,species},
                Piece{color:Player::Black,species}
            ]).map(|piece|Some(piece))
        ).enumerate().for_each(|(i,brush)|{

            let y = -8.0 + ((i/5) as f32 ) * 1.2;
            let x = -2.0 + ((i%5) as f32) * 1.1;

            self.process_palette_button(vec2(x,y), brush, camera);
        });
        
        

        const TMR_WIDTH : f32 = 6.0;
        const TMR_HEIGHT : f32 = 1.0;

        let to_move_rect = Rect::new(
            -TMR_WIDTH*0.5,
            5.0-TMR_HEIGHT*0.5,
            TMR_WIDTH, TMR_HEIGHT
        );

        let is_highlighted_to_move = to_move_rect.contains(mouse_world);

        let text = &format!("{} to move.",match self.state.to_play(){
            Player::White => "White",
            Player::Black => "Black"
        });
        
        draw_text_centered(
            text, assets.font, 0.8, 
            to_move_rect.center(), 
            Color::from_rgba(0x11, 0x11, 0x11, if is_highlighted_to_move{255} else {160})
        );

        
        let pstring : PositionString = (&self.state).into();
        draw_text_centered(
            &format!("{}",pstring), assets.font, 0.4, vec2(0.0,-4.5), 
            
            Color::from_hex(0x555555));




        if is_highlighted_to_move & is_mouse_button_pressed(MouseButton::Left){
            self.push_history();
            self.state.flip_to_move();
        }

        let mqui = MqUi::new(camera);
        if self.can_undo(){
            if self.btn_undo.process(&mqui){
                self.undo();
            }
        }
    }

    pub fn get_state_clone(&self) -> Position{
        self.state.clone()
    }

}