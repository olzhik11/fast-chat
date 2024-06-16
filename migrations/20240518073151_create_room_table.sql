-- Add migration script here
CREATE TABLE rooms (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    name VARCHAR(255) NOT NULL,
    description VARCHAR(255) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    updated_at timestamptz NOT NULL DEFAULT NOW()
)