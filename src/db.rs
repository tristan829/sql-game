use std::sync::mpsc;
use anyhow::Result;
use postgresql_embedded::PostgreSQL;
use sqlx::PgPool;

use crate::protocol::*;

pub async fn db_worker(request_reciever: mpsc::Receiver<Request>, response_sender: mpsc::Sender<Response>) -> Result<()> {
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
    sqlx::raw_sql(include_str!("init.sql"))
        .execute(&pool)
        .await?;

    sqlx::raw_sql(include_str!("main.sql"))
        .execute(&pool)
        .await?;

    sqlx::query("CALL init()").execute(&pool).await?;

    let _ = response_sender.send(Response::Ready);

    // Handle requests
    while let Ok(req) = request_reciever.recv() {
        match req {
            Request::Tick { delta_time, input } => {
                sqlx::query("CALL update($1, $2)")
                    .bind(delta_time)
                    .bind(input)
                    .execute(&pool)
                    .await?;

                let draw_command_rows: Vec<DrawCommandRow> = sqlx::query_as("SELECT * FROM draw_commands")
                    .fetch_all(&pool)
                    .await?;

                let draw_commands = draw_command_rows.into_iter().map(|row| DrawCommand::from(row)).collect();
                
                let _ = response_sender.send(Response::Tick(draw_commands));
            }
        }
    }

    Ok(())
}