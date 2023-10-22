CREATE TABLE user_contents(
	user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	contents_id uuid NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
	PRIMARY KEY (user_id, contents_id)
);
