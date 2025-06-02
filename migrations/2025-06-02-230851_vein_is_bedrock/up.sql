-- Your SQL goes here
CREATE TABLE vein_is_bedrock (
    id VARCHAR(36) PRIMARY KEY,
    vein_id VARCHAR(36) NOT NULL,
    is_bedrock BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (vein_id) REFERENCES vein(id) ON DELETE CASCADE
);