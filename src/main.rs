use macroquad::prelude::*;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

mod db;
use crate::db::db_worker;

mod protocol;
use crate::protocol::*;

mod render;
use crate::render::render;

mod texture_cache;
use crate::texture_cache::TextureCache;


#[macroquad::main("Convoluted SQL Abomination")]
async fn main() {
    // Create channels to talk between the threads
    let (request_sender, request_reciever) = mpsc::channel::<Request>();
    let (response_sender, response_receiver) = mpsc::channel::<Response>();
    
    // Start listening to database requests
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            if let Err(e) = db_worker(request_reciever, response_sender).await {
                eprintln!("DB worker error: {e:?}");
            }
        });
    });

    let mut loading = true;

    let mut draw_state = Vec::new();
    let mut texture_cache = TextureCache::new();

    // Request first tick
    let _ = request_sender.send(Request::Tick { delta_time: 0.0, input: Vec::new() });

    loop {
        let delta_time: f64 = get_frame_time().into();

        let input: Vec<String> = get_keys_down().iter().map(|key| format!("{:?}", key).to_lowercase()).collect();

        // Loading screen for database startup
        if loading {
            draw_text("Loading SQL...", 50.0, 50.0, 30.0, WHITE);
        }

        match response_receiver.try_recv() {
            Ok(Response::Ready) => loading = false,

            Ok(Response::Tick(draw_commands)) => {
                draw_state = draw_commands;
                
                let _ = request_sender.send(Request::Tick { delta_time, input });
            }

            Err(TryRecvError::Empty) => {} // No message this frame

            Err(TryRecvError::Disconnected) => {
                panic!("DB worker thread disconnected");
            }
        }

        render(&draw_state, &mut texture_cache).await;

        next_frame().await;
    }
}
