-- Your SQL goes here



CREATE TABLE "posts"(
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    author_id INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    published_at TIMESTAMP WITH TIME ZONE,
    is_published BOOLEAN DEFAULT FALSE,
    slug VARCHAR(255) UNIQUE,
    excerpt TEXT,
    featured_image_url VARCHAR(255),
    category_id INTEGER,
    view_count INTEGER DEFAULT 0,
    likes_count INTEGER DEFAULT 0,
	FOREIGN KEY ("author_id") REFERENCES "users"("id")
);

CREATE INDEX idx_posts_author_id ON posts(author_id);
CREATE INDEX idx_posts_category_id ON posts(category_id);
CREATE INDEX idx_posts_created_at ON posts(created_at);
CREATE INDEX idx_posts_published_at ON posts(published_at);
