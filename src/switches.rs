use std::collections::HashMap;

use indexmap::IndexMap;
use sdl2::{pixels::Color, rect::Rect};

const SWITCH_WIDTH: u32 = 140;
const SWITCH_HEIGHT: u32 = 20;
const MARGIN: u32 = 2;

pub fn render_switches(
    at_x: u32,
    at_y: u32,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    font: &sdl2::ttf::Font<'_, '_>,
    switches: &IndexMap<u32, &str>,
    switch_states: &HashMap<u32, bool>,
) -> Result<(), String> {
    for (index, (code, label)) in switches.iter().enumerate() {
        let x = (index % 5) as u32;
        let y = (index / 5) as u32;
        let rect = Rect::new(
            (at_x + x * (SWITCH_WIDTH + MARGIN)) as i32,
            (at_y + y * (SWITCH_HEIGHT + MARGIN)) as i32,
            SWITCH_WIDTH,
            SWITCH_HEIGHT,
        );
        let button_color = match switch_states.get(code) {
            Some(true) => Color::RGB(100, 40, 40),
            Some(false) => Color::RGB(40, 10, 10),
            None => Color::RGB(20, 20, 20),
        };
        canvas.set_draw_color(button_color);
        canvas.fill_rect(rect)?;

        let font_color = Color::RGBA(200, 200, 200, 255);

        let texture_creator = canvas.texture_creator();
        // render a surface, and convert it to a texture bound to the canvas
        // TODO we need to cache these textures
        let label = format!("{}: {}", code, label);
        let surface = font
            .render(&label)
            .blended(font_color)
            .map_err(|e| e.to_string())?;
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        // min of font width or switch width (scaled)
        let min_width = std::cmp::min(
            surface.width(),
            (SWITCH_WIDTH as f32 * canvas.scale().0) as u32,
        );
        let min_height = std::cmp::min(
            surface.height(),
            (SWITCH_HEIGHT as f32 * canvas.scale().1) as u32,
        );
        let src = Rect::new(0, 0, min_width, min_height);

        let target = Rect::new(
            (at_x + x * (SWITCH_WIDTH + MARGIN)) as i32,
            (at_y + y * (SWITCH_HEIGHT + MARGIN)) as i32,
            (min_width as f32 / canvas.scale().1) as u32,
            (min_height as f32 / canvas.scale().1) as u32,
        );

        canvas.copy(&texture, Some(src), Some(target))?;
    }
    Ok(())
}

pub fn switch_id_for_mouse(x: i32, y: i32, switches: &IndexMap<u32, &str>) -> Option<u32> {
    let x = x / (SWITCH_WIDTH + MARGIN) as i32;
    let y = y / (SWITCH_HEIGHT + MARGIN) as i32;

    let index = x + y * 5;

    if index < switches.len() as i32 {
        switches.keys().nth(index as usize).cloned()
    } else {
        None
    }
}
