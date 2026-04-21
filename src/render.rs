use macroquad::prelude::*;

use crate::{
    protocol::DrawCommand,
    texture_cache::TextureCache
};

pub async fn render(commands: &[DrawCommand], cache: &mut TextureCache) {
    for cmd in commands {
        match cmd {
            DrawCommand::Text { x, y, text_content } => {
                draw_text(&text_content, *x, *y, 30.0, GREEN);
            }

            DrawCommand::Sprite { x, y, width, height, sprite_path } => {
                let texture = cache.get(sprite_path).await;

                draw_texture_ex(
                    &texture,
                    *x,
                    *y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(*width, *height)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}