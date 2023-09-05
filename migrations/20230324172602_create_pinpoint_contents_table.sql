-- Add migration script here
CREATE TABLE pinpoint_contents(
	pinpoint_id uuid NOT NULL REFERENCES pinpoints(id) ON DELETE CASCADE,
	content_id uuid NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
	PRIMARY KEY (pinpoint_id, content_id)
);
