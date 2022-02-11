CREATE TABLE job (
    id INTEGER NOT NULL PRIMARY KEY,
    repository VARCHAR NOT NULL,
    hash VARCHAR,
    comments_url VARCHAR,
    started_at TIMESTAMP,
    UNIQUE(repository, hash)
);

CREATE TABLE master_build (
    id INTEGER NOT NULL PRIMARY KEY,
    hash VARCHAR NOT NULL
);
