-- Your SQL goes here
CREATE TABLE user (
    id VARCHAR(36) PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255),
    password_hash VARCHAR(255) NOT NULL,
    is_admin BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    invited_by CHAR(36),
    INDEX idx_username (username),
    INDEX idx_email (email),
    FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE SET NULL
);