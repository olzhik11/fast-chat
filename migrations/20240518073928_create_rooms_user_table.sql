-- Add migration script here
CREATE TABLE room_users (
    room_id uuid REFERENCES rooms(id),
    user_id uuid REFERENCES users(id),
    joined_at timestamptz NOT NULL DEFAULT NOW(),
    PRIMARY KEY (room_id, user_id)
)