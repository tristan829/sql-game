use macroquad::prelude::*;

use crate::protocol::DrawCommand;

pub fn render(draw_commands: &Vec<DrawCommand>) {
    for command in draw_commands.iter() {
        draw_text(&command.text_content, command.x, command.y, 30.0, GREEN);
    }
}

