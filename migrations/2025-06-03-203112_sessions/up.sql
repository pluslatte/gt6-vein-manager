-- Your SQL goes here
CREATE TABLE sessions (
    id VARCHAR(255) PRIMARY KEY,
    data JSON NOT NULL,
    expiry_date TIMESTAMP NOT NULL,
    INDEX idx_expiry_date (expiry_date)
);