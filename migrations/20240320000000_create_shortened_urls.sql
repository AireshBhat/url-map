-- Create shortened_urls table
CREATE TABLE IF NOT EXISTS shortened_urls (
    id BIGSERIAL PRIMARY KEY,
    original_url TEXT NOT NULL,
    short_url VARCHAR(10) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    visits BIGINT NOT NULL DEFAULT 0
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_shortened_urls_short_url ON shortened_urls(short_url);
CREATE INDEX IF NOT EXISTS idx_shortened_urls_created_at ON shortened_urls(created_at); 