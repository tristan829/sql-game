CREATE PROCEDURE init()
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO draw_commands (kind, x, y, width, height, sprite_path)
    VALUES ('sprite', 50, 50, 100, 100, 'src/placeholder.png');
END;
$$;

CREATE PROCEDURE update(delta_time FLOAT, input TEXT[])
LANGUAGE plpgsql
AS $$
DECLARE
    player_x FLOAT;
    pressed_key TEXT;
BEGIN
    SELECT x INTO player_x FROM draw_commands WHERE id = 1;
    FOREACH pressed_key IN ARRAY input LOOP
        CASE pressed_key
            WHEN 'left' THEN
                player_x = player_x - 1;
            WHEN 'right' THEN
                player_x = player_x + 1;
            ELSE -- No else
        END CASE;
    END LOOP;

    UPDATE draw_commands SET x = player_x;
END;
$$;