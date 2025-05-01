-- Your SQL goes here


CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    slug VARCHAR NOT NULL UNIQUE,
    parent_id INTEGER REFERENCES categories(id),
    description TEXT,
    cover_image VARCHAR,
    logo_image VARCHAR,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
