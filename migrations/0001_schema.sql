-- DROP OWNED BY CURRENT_USER CASCADE;
CREATE TYPE STAT AS ENUM ('waiting', 'p1turn', 'p2turn');

CREATE TABLE IF NOT EXISTS players (
    id CHAR(16) PRIMARY KEY,
    board CHAR(10) [10],
    room_code CHAR(4) NOT NULL,
    abandoned BOOLEAN DEFAULT FALSE NOT NULL,
    time TIMESTAMP DEFAULT NOW() NOT NULL
);

CREATE TABLE IF NOT EXISTS rooms (
    code CHAR(4) PRIMARY KEY,
    player1_id CHAR(16),
    player2_id CHAR(16),
    stat STAT DEFAULT 'waiting' NOT NULL,
    CHECK (
        (
            player1_id IS DISTINCT
            FROM player2_id
        )
        OR (
            player1_id IS NULL
            AND player2_id IS NULL
        )
    )
);

ALTER TABLE players
ADD CONSTRAINT fk_room_code FOREIGN KEY (room_code) REFERENCES rooms (code) ON DELETE
SET NULL;

ALTER TABLE rooms
ADD CONSTRAINT fk_player1 FOREIGN KEY (player1_id) REFERENCES players (id) ON DELETE
SET NULL ON UPDATE CASCADE,
    ADD CONSTRAINT fk_player2 FOREIGN KEY (player2_id) REFERENCES players (id) ON DELETE
SET NULL ON UPDATE CASCADE;

CREATE INDEX idx_player_room_code ON players (room_code);
CREATE INDEX idx_room_status ON players (abandoned);