-- Initial schema
CREATE TABLE IF NOT EXISTS users (
	id BIGINT PRIMARY KEY NOT NULL UNIQUE,
	username VARCHAR(32) UNIQUE NOT NULL,
	display_name VARCHAR(32),
	bio VARCHAR(2048),
	password_hash varchar NOT NULL,
	permissions BIGINT NOT NULL DEFAULT 43,
	flags INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS messages (
	id BIGINT PRIMARY KEY NOT NULL UNIQUE,
	author_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	content VARCHAR(4096) NOT NULL,
	thread_id BIGINT NOT NULL,
	referenced_message_id BIGINT REFERENCES messages(id) ON DELETE SET NULL,
	flags INTEGER NOT NULL DEFAULT 0,
	updated_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS categories (
	id BIGINT PRIMARY KEY NOT NULL UNIQUE,
	owner_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	title VARCHAR(128) NOT NULL,
	description VARCHAR(2048) NOT NULL,
	locked BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS threads (
	id BIGINT PRIMARY KEY NOT NULL UNIQUE,
	title VARCHAR(128) NOT NULL,
	author_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	category_id BIGINT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
	original_message_id BIGINT NOT NULL UNIQUE REFERENCES messages(id) ON DELETE CASCADE,
	flags INTEGER NOT NULL DEFAULT 0
);

ALTER TABLE messages
ADD CONSTRAINT fk_messages_thread
FOREIGN KEY (thread_id) REFERENCES threads(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED;

CREATE OR REPLACE FUNCTION updated_at_column() RETURNS TRIGGER AS $$
    BEGIN
        NEW.updated_at = CURRENT_TIMESTAMP;
        RETURN NEW;
    END;
$$ LANGUAGE 'plpgsql';


CREATE OR REPLACE TRIGGER updated_at_users BEFORE UPDATE ON messages FOR EACH ROW EXECUTE PROCEDURE updated_at_column();