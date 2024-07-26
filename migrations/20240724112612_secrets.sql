-- Add secrets table

CREATE TABLE IF NOT EXISTS secrets (
	id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	password_hash varchar NOT NULL,
	email varchar NOT NULL,

	secret1 BIGINT NOT NULL,
	secret2 BIGINT NOT NULL,
	secret3 BIGINT NOT NULL
)
