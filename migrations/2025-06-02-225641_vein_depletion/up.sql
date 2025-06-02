-- Your SQL goes here
CREATE TABLE vein_depletion (
    id VARCHAR(36) PRIMARY KEY,
    vein_id VARCHAR(36) NOT NULL,
    depleted BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (vein_id) REFERENCES vein(id) ON DELETE CASCADE
);