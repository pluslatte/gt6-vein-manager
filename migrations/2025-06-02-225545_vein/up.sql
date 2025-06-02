-- Your SQL goes here
CREATE TABLE vein (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    x_coord INT NOT NULL,
    y_coord INT DEFAULT NULL,
    z_coord INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);