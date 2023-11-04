CREATE TABLE servers (
	id bigserial NOT NULL,
	actual_id varchar NOT NULL,
	CONSTRAINT servers_pkey PRIMARY KEY (id)
);

CREATE TABLE users (
	id bigserial NOT NULL,
	discord_id varchar NOT NULL,
	server_id int8 NOT NULL,
	CONSTRAINT users_pkey PRIMARY KEY (id),
	CONSTRAINT users_server_id_fkey FOREIGN KEY (server_id) REFERENCES servers(id)
);

CREATE TABLE femboys (
	user_id int8 NOT NULL,
	balance int8 NOT NULL DEFAULT 0,
	wins_num int8 NOT NULL DEFAULT 0,
	CONSTRAINT femboys_pkey PRIMARY KEY (user_id),
	CONSTRAINT femboys_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id)
);