use macroquad::prelude::*;

pub fn draw_arrow(
    start : Vec2, 
    end : Vec2, 
    color : Color, 
    thickness : f32, 
    head_length : f32, 
    head_width : f32){
    let (x1,y1) = start.into();
    


    let direction = (end-start).normalize();
    let base_center = end - direction * head_length;

    let (x2,y2) = base_center.into();
    draw_line(x1, y1, x2, y2, thickness, color);

    let base_vec = vec2(-direction.y,direction.x);
    
    let base_delta = base_vec * head_width * 0.5;
    let v1 = base_center + base_delta;
    let v2 = base_center - base_delta;

    draw_triangle(v1, v2, end, color);

}


#[allow(dead_code)]
pub fn draw_arrow_bowed(
    start : Vec2, 
    end : Vec2, 
    color : Color, 
    thickness : f32, 
    head_length : f32, 
    head_width : f32,
    bowing : f32
){

    const N_SAMPLES : usize = 10;

    let displacement = end-start;
    let distance = displacement.length();
    let height = distance * bowing;

    let samples : Vec<Vec2> = (0..N_SAMPLES).map(|i|{
        let t = (i as f32) / (N_SAMPLES as f32 - 1.0);

        let floor = start.lerp(end, t);

        floor + vec2(0.0, -1.0) * height * (4.0 * t *(1.0-t))
    }).collect();

    samples.windows(2).map(|pair|(pair[0],pair[1]))
    .for_each(|(p1,p2)|
        draw_line(p1.x, p1.y, p2.x, p2.y, thickness, color)
    );
}

#[allow(dead_code)]
pub fn draw_arrow_outlined(start : Vec2, end : Vec2, color : Color, thickness : f32, head_length : f32, head_width : f32, outline_color : Color, outline_thickness : f32){

    draw_arrow(
        start, end+ (end-start).normalize()*outline_thickness*0.707, 
        outline_color, 
        thickness+ outline_thickness, 
        head_length+1.0*outline_thickness,
         head_width+2.0*outline_thickness);
    draw_arrow(start, end , color, thickness, head_length, head_width);
}