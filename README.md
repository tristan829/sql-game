# Game in SQL
A simple fruit-catching video game written in SQL (Specifically, PL/pgSQL). This is a novelty project and is mostly intended to show that it is possible, because this is really inefficient.

## Hey, I see Rust! That's not SQL!
I have to make a window somehow! SQL has all the logic -- even saying what to render -- but the Rust code starts the Postgres server and runs the loop of input, update, render. It's like the engine for the game.
