use sqlx::prelude::FromRow;

#[derive(Debug, FromRow)]
pub struct DrawCommand {
    pub x: f32,
    pub y: f32,
    pub text_content: String,
}

#[derive(Debug)]
pub enum Request {
    Tick(f64),
}

#[derive(Debug)]
pub enum Response {
    Ready,
    Tick(Vec<DrawCommand>),
}