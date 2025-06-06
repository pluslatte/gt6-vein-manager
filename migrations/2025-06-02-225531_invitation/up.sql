-- Your SQL goes here
CREATE TABLE invitation (
    id VARCHAR(36) PRIMARY KEY,
    email VARCHAR(255),
    token CHAR(36) UNIQUE NOT NULL,
    invited_by VARCHAR(255) NULL,
    expires_at TIMESTAMP NOT NULL,
    used_at TIMESTAMP NULL,
    used_by CHAR(36) NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_token (token),
    INDEX idx_email (email),
    FOREIGN KEY (used_by) REFERENCES user(id) ON DELETE SET NULL
);