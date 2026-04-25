CREATE TABLE game_data (
    id INT PRIMARY KEY DEFAULT 1,
    player_speed FLOAT DEFAULT 300,
    fall_speed FLOAT DEFAULT 200,
    time_elapsed FLOAT DEFAULT 0,
    last_object_spawn_time FLOAT DEFAULT 0,
    points INT DEFAULT 0,
    dead BOOLEAN DEFAULT FALSE
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

    IF random() < 0.2 THEN
        INSERT INTO draw_commands (kind, tag, x, y, width, height, sprite_path)
        VALUES ('sprite', 'falling_object_bomb', random(0, window_width - 50), -50, 50, 50, 'src/bomb.png');
    ELSE
        INSERT INTO draw_commands (kind, tag, x, y, width, height, sprite_path)
        VALUES ('sprite', 'falling_object', random(0, window_width - 50), -50, 50, 50, 'src/apple.png');
    END IF;
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

    player_dead BOOLEAN;
BEGIN

    SELECT width INTO window_width FROM window_config;

    SELECT player_speed, time_elapsed, fall_speed, last_object_spawn_time, points, dead
    INTO player_move_speed, elapsed, object_fall_speed, last_spawn, player_points, player_dead
    FROM game_data
    WHERE id = 1;

    IF player_dead THEN
        -- Draw game over screen
        DELETE FROM draw_commands;

        INSERT INTO draw_commands (kind, tag, x, y, text_content)
        VALUES ('text', 'game_over_text', 25, 100, CONCAT('You died! You got a score of ', player_points));

        INSERT INTO draw_commands (kind, tag, x, y, text_content)
        VALUES ('text', 'game_over_restart_prompt', 25, 150, 'Press Space to restart');

        FOREACH pressed_key IN ARRAY input LOOP
            CASE pressed_key
                WHEN 'space' THEN
                    DELETE FROM draw_commands;
                    DELETE FROM game_data;
                    CALL init();
                    RETURN;
                ELSE
                    NULL;
            END CASE;
        END LOOP;
    END IF;

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
    WHERE tag = 'falling_object' OR tag = 'falling_object_bomb';


    -- Collision

    -- Bombs
    WITH collided AS (
        DELETE FROM draw_commands o
        USING draw_commands p
        WHERE p.tag = 'player'
        AND o.tag = 'falling_object_bomb'
        AND o.x < p.x + 100
        AND o.x + 50 > p.x
        AND o.y < p.y + 100
        AND o.y + 50 > p.y
        RETURNING 1
    )
    SELECT COUNT(*) INTO added_points
    FROM collided;
    IF added_points >= 1 THEN
        UPDATE game_data SET dead = TRUE WHERE id = 1;
    END IF;

    -- Points
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