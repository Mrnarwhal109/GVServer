-- Add migration script here
CREATE TABLE contents(
	id uuid NOT NULL,
	PRIMARY KEY (id),
	description TEXT NULL,
	attachment bytea NULL,
	added_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
