-- Add migration script here
CREATE TABLE user_pinpoints(
    user_id uuid NOT NULL,
	pinpoint_id uuid NOT NULL,
	PRIMARY KEY (user_id, pinpoint_id)
);
