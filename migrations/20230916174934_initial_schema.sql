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
INSERT INTO roles(id, title) VALUES(5, 'VIBE_GOD');

CREATE TABLE user_pinpoints(
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	pinpoint_id uuid NOT NULL REFERENCES pinpoints(id) ON DELETE CASCADE,
	PRIMARY KEY (user_id, pinpoint_id)
);

-- Add migration script here
CREATE TABLE pinpoint_contents(
	pinpoint_id uuid NOT NULL REFERENCES pinpoints(id) ON DELETE CASCADE,
	content_id uuid NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
	PRIMARY KEY (pinpoint_id, content_id)
);

CREATE TABLE user_roles(
	user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	role_id int NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
	PRIMARY KEY (user_id, role_id)
);
