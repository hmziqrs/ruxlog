-- Your SQL goes here

CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    slug VARCHAR NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE OR REPLACE FUNCTION array_remove_duplicates(anyarray) RETURNS anyarray AS $$
  SELECT array_agg(DISTINCT x) FROM unnest($1) AS t(x);
$$ LANGUAGE SQL IMMUTABLE;


ALTER TABLE posts
ADD COLUMN tag_ids INTEGER[] DEFAULT '{}';

ALTER TABLE posts
ADD CONSTRAINT unique_tag_ids CHECK (array_length(tag_ids, 1) = cardinality(array_remove_duplicates(tag_ids)));
