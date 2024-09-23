-- delete room if both players are null
CREATE OR REPLACE FUNCTION delete_room() RETURNS TRIGGER AS $$ BEGIN IF (
        SELECT player1_id IS NULL
            AND player2_id IS NULL
        FROM rooms
        WHERE code = OLD.room_code
    ) THEN
DELETE FROM rooms
WHERE code = OLD.room_code;
END IF;
RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER delete_room_trigger
AFTER DELETE ON players FOR EACH ROW EXECUTE FUNCTION delete_room();


-- retain only 1000 recent abandoned players according to timestamp
CREATE OR REPLACE FUNCTION delete_player() RETURNS TRIGGER AS $$ BEGIN IF (
        SELECT COUNT(*)
        FROM players
        WHERE abandoned = TRUE
    ) > 10000 THEN
DELETE FROM players
WHERE id IN (
        SELECT id
        FROM players
        WHERE abandoned = TRUE
        ORDER BY time DESC OFFSET 10000
    );
END IF;
RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER delete_player_trigger
AFTER
INSERT ON players FOR EACH ROW EXECUTE FUNCTION delete_player();