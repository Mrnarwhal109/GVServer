-- Add migration script here
CREATE TABLE pinpoints(
	id uuid NOT NULL,
    PRIMARY KEY (id),
	coordinates POINT NOT NULL,
	added_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
