CREATE TABLE jobs (
    id INTEGER NOT NULL PRIMARY KEY,
    repository VARCHAR NOT NULL,
    hash VARCHAR NOT NULL,
    comments_url VARCHAR NOT NULL,
    started_at TIMESTAMP,
    UNIQUE(repository, hash)
)
