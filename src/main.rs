use anyhow::Result;
use macroquad::prelude::*;
use postgresql_embedded::PostgreSQL;
use sqlx::prelude::FromRow;
use sqlx::{PgPool};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

#[derive(Debug, FromRow)]
struct DrawCommand {
    x: f32,
    y: f32,
    text_content: String,
}

#[derive(Debug)]
enum Request {
    Tick(f64),
}

#[derive(Debug)]
enum Response {
    Ready,
    Tick(Vec<DrawCommand>),
}

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

    // Request first tick
    let _ = request_sender.send(Request::Tick(0.0));

    loop {
        let delta_time: f64 = get_frame_time().into();

        // Loading screen for database startup
        if loading {
            draw_text("Loading SQL...", 50.0, 50.0, 30.0, WHITE);
        }

        match response_receiver.try_recv() {
            Ok(Response::Ready) => loading = false,

            Ok(Response::Tick(draw_commands)) => {
                draw_state = draw_commands;

                let _ = request_sender.send(Request::Tick(delta_time));
            }

            Err(TryRecvError::Empty) => {} // No message this frame

            Err(TryRecvError::Disconnected) => {
                panic!("DB worker thread disconnected");
            }
        }

        render(&draw_state);

        next_frame().await;
    }
}

async fn db_worker(request_reciever: mpsc::Receiver<Request>, response_sender: mpsc::Sender<Response>) -> Result<()> {
    // Start the server
    let mut postgresql = PostgreSQL::default();

    postgresql.setup().await.unwrap();
    postgresql.start().await.unwrap();

    let db = "sql-game";

    // Delete the old database if it exists before creating the new one for this session
    if postgresql.database_exists(db).await.unwrap() {
        postgresql.drop_database(db).await.unwrap();
    }

    postgresql.create_database(db).await.unwrap();

    let url = postgresql.settings().url(db);

    let pool = PgPool::connect(&url).await.unwrap();

    // Initialize
    sqlx::raw_sql(include_str!("main.sql"))
        .execute(&pool)
        .await?;

    sqlx::query("CALL init()").execute(&pool).await?;

    let _ = response_sender.send(Response::Ready);

    // Handle requests
    while let Ok(req) = request_reciever.recv() {
        match req {
            Request::Tick(delta_time) => {
                sqlx::query("CALL update($1)")
                    .bind(delta_time)
                    .execute(&pool)
                    .await?;

                let draw_commands: Vec<DrawCommand> = sqlx::query_as("SELECT * FROM draw_commands")
                    .fetch_all(&pool)
                    .await?;
                
                let _ = response_sender.send(Response::Tick(draw_commands));
            }
        }
    }

    Ok(())
}

fn render(draw_commands: &Vec<DrawCommand>) {
    for command in draw_commands.iter() {
        draw_text(&command.text_content, command.x, command.y, 30.0, GREEN);
    }
}