-- Add migration script here
CREATE TABLE messages (
    id uuid NOT NULL UNIQUE,
    content VARCHAR NOT NULL,
    author uuid NOT NULL,
    room uuid NOT NULL,
    status SMALLINT NOT NULL,
    created_at timestamptz NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_users
        FOREIGN KEY(author)
            REFERENCES users(id)
)