use std::iter::once;

use crate::{assets::Assets, theme, Piece, PieceType, Player, Position, Tile};
use macroquad::prelude::*;

#[derive(Clone)]
pub struct PositionEditor{
    state : Position,
    selected_brush : Option<Piece>,
}

impl PositionEditor{
    pub fn setup() -> PositionEditor{
        PositionEditor{
            state : Position::setup(),
            selected_brush : None
        }
    }

    fn process_palette_button(&mut self, position : Vec2, brush : Option<Piece>, camera : &Camera2D, assets : &Assets){
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
            piece.draw(position.x, position.y,assets.pieces,  0.9)
        } else {
            draw_line(position.x-0.3, position.y-0.3,position.x+0.3,position.y+0.3, 0.1, RED);
            draw_circle_lines(position.x, position.y, 0.3, 0.1, RED);
        }


        
    }

    pub fn process(&mut self, camera : &Camera2D, assets : &Assets){
        set_camera(camera);

        
        draw_rectangle(-7.0, -8.0, 14.0, 16.0, theme::BG_COLOR);

        Tile::draw_board(false);

        self.state.draw(
            assets.pieces,
            assets.font, 
            false, false, false);

        let mouse = mouse_position();
        let mouse_world = camera.screen_to_world(mouse.into());

        if let Some(hover_tile) = Tile::from_world(mouse_world.x, mouse_world.y, false){
            hover_tile.draw_highlight_outline(0.1,Color::from_hex(0x111111), false);

            if is_mouse_button_pressed(MouseButton::Left){
                self.state.paint(&hover_tile, self.selected_brush);
            }
        }

        once(None).chain(
            (0..7).map(|i|PieceType::from_code(i))
            .flat_map(|species|[
                Piece{color:Player::White,species},
                Piece{color:Player::Black,species}
            ]).map(|piece|Some(piece))
        ).enumerate().for_each(|(i,brush)|{

            let y = -8.0 + ((i/5) as f32 ) * 1.2;
            let x = -2.0 + ((i%5) as f32) * 1.1;

            self.process_palette_button(vec2(x,y), brush, camera, assets);
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
        let (x,y) = to_move_rect.center().into();
        let (font_size, font_scale, font_scale_aspect) = camera_font_scale(0.8);
        let center = get_text_center(text, Some(assets.font), font_size, font_scale, 0.0);
        draw_text_ex(text,x-center.x,y-center.y, TextParams{
            font : assets.font,
            font_size, font_scale, font_scale_aspect,
            color : Color::from_rgba(0x11, 0x11, 0x11, if is_highlighted_to_move{255} else {160}),
            ..Default::default()
        });

        if is_highlighted_to_move & is_mouse_button_pressed(MouseButton::Left){
            self.state.flip_to_move();
        }
    }

    pub fn get_state_clone(&self) -> Position{
        self.state.clone()
    }

}