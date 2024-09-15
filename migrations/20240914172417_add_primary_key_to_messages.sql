-- Add migration script here
ALTER TABLE messages
    ADD CONSTRAINT pk_messages_id
    PRIMARY KEY (id);