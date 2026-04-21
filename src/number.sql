CREATE TABLE data (
    num FLOAT
);

CREATE PROCEDURE init()
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO data (num) VALUES (0.0);
    INSERT INTO draw_commands (kind, x, y, text_content) VALUES ('text', 50, 50, 'Number is 0');
END;
$$;

CREATE PROCEDURE update(delta_time FLOAT)
LANGUAGE plpgsql
AS $$
DECLARE
    current_num FLOAT;
BEGIN
    UPDATE data SET num = num + delta_time;

    -- Display
    SELECT num INTO current_num FROM data LIMIT 1;

    UPDATE draw_commands
    SET text_content = CONCAT('Number is ', current_num::TEXT)
    WHERE id = 1;
END;
$$;