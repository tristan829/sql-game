use anyhow::Result;
use macroquad::prelude::*;
use postgresql_embedded::PostgreSQL;
use sqlx::{PgPool, Row};
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
enum Request {
    Update { delta_time: f64 },
    GetNum,
}

#[derive(Debug)]
enum Response {
    Ready,
    Ok,
    Num(f64),
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
    let mut num = 0f64;

    loop {
        let delta_time: f64 = get_frame_time().into();

        // Handle responses
        while let Ok(msg) = response_receiver.try_recv() {
            match msg {
                Response::Ready => loading = false,
                Response::Num(v) => num = v,
                Response::Ok => {}
            }
        }

        // Draw
        if loading {
            draw_text("Loading SQL...", 50.0, 50.0, 30.0, WHITE);
        } else {
            draw_text(&format!("num = {}", num), 50.0, 50.0, 30.0, GREEN);

            // Update for next frame
            let _ = request_sender.send(Request::Update { delta_time });
            let _ = request_sender.send(Request::GetNum);
        }
        

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
            Request::Update { delta_time } => {
                sqlx::query("CALL update($1)").bind(delta_time).execute(&pool).await?;
                let _ = response_sender.send(Response::Ok);
            }

            Request::GetNum => {
                let row = sqlx::query("SELECT num FROM output LIMIT 1")
                    .fetch_one(&pool)
                    .await?;

                let v: f64 = row.get("num");
                let _ = response_sender.send(Response::Num(v));
            }
        }
    }

    Ok(())
}