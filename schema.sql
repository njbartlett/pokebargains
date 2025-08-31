CREATE TABLE IF NOT EXISTS category (
    id serial PRIMARY KEY,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS item (
    id bigserial PRIMARY KEY,
    category int NOT NULL REFERENCES category,
    title text NOT NULL,
    price money NOT NULL,
    description text DEFAULT ''
);

CREATE TABLE IF NOT EXISTS image (
    id bigserial PRIMARY KEY,
    item bigint NOT NULL REFERENCES item,
    path text,
    url text NOT NULL,
    width int CHECK (width >= 0),
    height int CHECK (height >= 0),
    ordinal int NOT NULL CHECK (ordinal >= 0),
    CONSTRAINT image_ordinal_unique UNIQUE (item, ordinal)
);