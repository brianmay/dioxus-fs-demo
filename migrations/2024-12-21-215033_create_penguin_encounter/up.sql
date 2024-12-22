CREATE TYPE penalty_enum AS ENUM ('pat_penguin', 'become_penguin_food',  'jail', 'sacrifice');

CREATE TABLE penguin_encounter (
  id SErIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  location VARCHAR NOT NULL,
  penalty penalty_enum NOT NULL,
  date_time timestamp with time zone NOT NULL
);
