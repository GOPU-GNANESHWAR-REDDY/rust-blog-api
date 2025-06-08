CREATE TABLE posts_tags (
    fk_post_id INTEGER NOT NULL REFERENCES posts(id),
    tag VARCHAR NOT NULL,
    PRIMARY KEY (fk_post_id, tag)
);