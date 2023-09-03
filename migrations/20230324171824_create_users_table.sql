-- Add migration script here
CREATE TABLE users(
	id uuid NOT NULL,
	PRIMARY KEY (id),
	email TEXT NOT NULL UNIQUE,
	username TEXT NOT NULL,
	phash TEXT NOT NULL,
	salt TEXT NOT NULL,
	added_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
