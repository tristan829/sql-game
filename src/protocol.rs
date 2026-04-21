use sqlx::prelude::{FromRow, Type};

#[derive(Debug)]
pub enum Request {
    Tick {
        delta_time: f64,
        input: Vec<String>
    },
}

#[derive(Debug)]
pub enum Response {
    Ready,
    Tick(Vec<DrawCommand>),
}

#[derive(Debug, Type)]
#[sqlx(type_name = "draw_command_kind", rename_all = "lowercase")]
pub enum DrawCommandKind {
    Text,
    Sprite,
}

/// Draw command representation the `draw_commands` SQL table stores
#[derive(Debug, FromRow)]
pub struct DrawCommandRow {
    x: f32,
    y: f32,

    kind: DrawCommandKind,

    // Text
    text_content: Option<String>,
    
    // Sprite
    width: Option<f32>,
    height: Option<f32>,
    sprite_path: Option<String>,
}

/// Draw command representation the render function uses
#[derive(Debug)]
pub enum DrawCommand {
    Text {
        x: f32,
        y: f32,
        text_content: String,
    },
    Sprite {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        sprite_path: String,
    }
}

impl From<DrawCommandRow> for DrawCommand {
    fn from(row: DrawCommandRow) -> Self {
        match row.kind {
            DrawCommandKind::Text => {
                DrawCommand::Text {
                    x: row.x,
                    y: row.y,
                    text_content: row.text_content.unwrap_or_default()
                }
            }
            DrawCommandKind::Sprite => {
                let sprite_path = row.sprite_path.unwrap_or_else(|| panic!("No filepath for a sprite was given"));
                DrawCommand::Sprite {
                    x: row.x,
                    y: row.y,
                    width: row.width.unwrap_or(100.0),
                    height: row.width.unwrap_or(100.0),
                    sprite_path: sprite_path
                }
            }
        }
    }
}