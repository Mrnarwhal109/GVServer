-- Add migration script here
CREATE TABLE pinpoint_contents(
	pinpoint_id uuid NOT NULL,
	content_id uuid NOT NULL,
	PRIMARY KEY (pinpoint_id, content_id)
);
