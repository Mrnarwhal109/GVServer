-- Add migration script here
CREATE TABLE user_pinpoints(
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	pinpoint_id uuid NOT NULL REFERENCES pinpoints(id) ON DELETE CASCADE,
	PRIMARY KEY (user_id, pinpoint_id)
);
