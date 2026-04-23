CREATE TABLE game_data (
    id INT PRIMARY KEY DEFAULT 1,
    player_speed FLOAT DEFAULT 300,
    fall_speed FLOAT DEFAULT 200,
    time_elapsed FLOAT DEFAULT 0,
    last_object_spawn_time FLOAT DEFAULT 0,
    points INT DEFAULT 0
);

CREATE PROCEDURE init()
LANGUAGE plpgsql
AS $$
DECLARE
    window_height INT;
BEGIN
    SELECT height INTO window_height FROM window_config;

    INSERT INTO game_data (id)
    VALUES (1);

    INSERT INTO draw_commands (kind, tag, x, y, width, height, sprite_path)
    VALUES ('sprite', 'player', 50, window_height - 100, 100, 100, 'src/player.png');
    
    INSERT INTO draw_commands (kind, tag, x, y, text_content)
    VALUES ('text', 'points_label', 10, 25, 'Points: 0');
END;
$$;

CREATE PROCEDURE spawn_falling_object()
LANGUAGE plpgsql
AS $$
DECLARE
    window_width INT;
    window_height INT;
BEGIN
    SELECT width  INTO window_width  FROM window_config;
    SELECT height INTO window_height FROM window_config;

    INSERT INTO draw_commands (kind, tag, x, y, width, height, sprite_path)
    VALUES ('sprite', 'falling_object', random(0, window_width - 50), -50, 50, 50, 'src/apple.png');
END;
$$;

CREATE PROCEDURE update(delta_time FLOAT, input TEXT[])
LANGUAGE plpgsql
AS $$
DECLARE
    dx FLOAT := 0;
    pressed_key TEXT;

    window_width INT;
    player_move_speed FLOAT;

    elapsed FLOAT;
    object_fall_speed FLOAT;
    last_spawn FLOAT;

    player_points INT;
    added_points INT;
BEGIN
    SELECT width INTO window_width FROM window_config;

    SELECT player_speed, time_elapsed, fall_speed, last_object_spawn_time, points
    INTO player_move_speed, elapsed, object_fall_speed, last_spawn, player_points
    FROM game_data
    WHERE id = 1;

    elapsed := elapsed + delta_time;

    FOREACH pressed_key IN ARRAY input LOOP
        CASE pressed_key
            WHEN 'left' THEN
                dx := dx - player_move_speed * delta_time;
            WHEN 'right' THEN
                dx := dx + player_move_speed * delta_time;
            ELSE
                NULL;
        END CASE;
    END LOOP;

    UPDATE draw_commands
    SET x = GREATEST(0, LEAST(x + dx, window_width - 100))
    WHERE tag = 'player';

    UPDATE game_data
    SET time_elapsed = elapsed
    WHERE id = 1;

    IF elapsed - last_spawn >= 0.5 THEN
        CALL spawn_falling_object();

        UPDATE game_data
        SET last_object_spawn_time = elapsed
        WHERE id = 1;
    END IF;

    UPDATE draw_commands
    SET y = y + object_fall_speed * delta_time
    WHERE tag = 'falling_object';

    -- Collision
    WITH collided AS (
        DELETE FROM draw_commands o
        USING draw_commands p
        WHERE p.tag = 'player'
        AND o.tag = 'falling_object'
        AND o.x < p.x + 100
        AND o.x + 50 > p.x
        AND o.y < p.y + 100
        AND o.y + 50 > p.y
        RETURNING 1
    )
    SELECT COUNT(*) INTO added_points
    FROM collided;

    player_points := player_points + added_points;

    UPDATE draw_commands
    SET text_content = CONCAT('Points: ', player_points)
    WHERE tag = 'points_label';

    UPDATE game_data
    SET points = player_points
    WHERE id = 1;
END;
$$;