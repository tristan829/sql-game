-- This file defines types for SQL to have an easier time talking to the Rust side

CREATE TYPE draw_command_kind AS ENUM ('text', 'sprite');

CREATE TABLE draw_commands (
    id SERIAL PRIMARY KEY,
    tag TEXT,

    x FLOAT4,
    y FLOAT4,

    kind draw_command_kind,

    -- Text
    text_content TEXT,

    -- Sprite
    width FLOAT4,
    height FLOAT4,
    sprite_path TEXT
);

CREATE TABLE window_config AS
SELECT
    480 AS width,
    360 AS height;