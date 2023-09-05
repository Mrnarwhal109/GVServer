-- Add migration script here
CREATE TABLE users(
	id uuid NOT NULL,
	PRIMARY KEY (id),
	email TEXT NOT NULL UNIQUE,
	username TEXT NOT NULL UNIQUE,
	phash TEXT NOT NULL,
	salt TEXT NOT NULL,
	added_at timestamptz NOT NULL DEFAULT clock_timestamp()
);

-- Add migration script here
CREATE TABLE contents(
	id uuid NOT NULL,
	PRIMARY KEY (id),
	description TEXT NULL,
	attachment bytea NULL,
	added_at timestamptz NOT NULL DEFAULT clock_timestamp()
);

-- Add migration script here
CREATE TABLE pinpoints(
	id uuid NOT NULL,
    PRIMARY KEY (id),
    latitude DOUBLE PRECISION NULL,
    longitude DOUBLE PRECISION NULL,
	added_at timestamptz NOT NULL DEFAULT clock_timestamp()
);

-- Add migration script here
CREATE TABLE roles(
	id int NOT NULL,
    PRIMARY KEY (id),
    title TEXT NOT NULL,
	added_at timestamptz NOT NULL DEFAULT clock_timestamp()
);

INSERT INTO roles(id, title) VALUES(1, 'RESTRICTED');
INSERT INTO roles(id, title) VALUES(2, 'BASIC');
INSERT INTO roles(id, title) VALUES(3, 'ELEVATED');
INSERT INTO roles(id, title) VALUES(4, 'ADMIN');
INSERT INTO roles(id, title) VALUES(6, 'VIBE_GOD');