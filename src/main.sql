CREATE TABLE output (
    num FLOAT NOT NULL DEFAULT 0
);

CREATE PROCEDURE init()
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO output(num) VALUES (0.0);
END;
$$;

CREATE PROCEDURE update(delta_time FLOAT)
LANGUAGE plpgsql
AS $$
BEGIN
    UPDATE output SET num = num + delta_time;
END;
$$;