-- Add migration script here
CREATE TABLE pinpoints(
	id uuid NOT NULL,
    PRIMARY KEY (id),
    latitude DOUBLE PRECISION NULL,
    longitude DOUBLE PRECISION NULL,
	description TEXT, -- This needs to be deleted. It is a "cutting corners" field.
	added_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
