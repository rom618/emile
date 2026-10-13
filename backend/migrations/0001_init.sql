CREATE TABLE items (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    place TEXT NOT NULL CHECK (place IN ('fridge', 'freezer', 'cupboard')),
    qty REAL NOT NULL CHECK (qty >= 0),
    unit TEXT NOT NULL,
    cook_unit TEXT,
    cook_per_unit REAL,
    expires_on TEXT,
    expiry_guessed INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE unit_factors (
    name TEXT NOT NULL,
    unit TEXT NOT NULL,
    cook_unit TEXT NOT NULL,
    cook_per_unit REAL NOT NULL,
    PRIMARY KEY (name, unit)
);

CREATE TABLE recipes (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    steps TEXT NOT NULL,
    minutes INTEGER,
    servings INTEGER,
    rating INTEGER CHECK (rating BETWEEN 1 AND 5),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE recipe_ingredients (
    recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    qty REAL NOT NULL,
    unit TEXT NOT NULL
);

CREATE TABLE cooks (
    id INTEGER PRIMARY KEY,
    recipe_id INTEGER NOT NULL REFERENCES recipes(id),
    cooked_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cancelled_at TEXT
);

CREATE TABLE movements (
    id INTEGER PRIMARY KEY,
    item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    delta REAL NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('receipt', 'text', 'photo', 'recipe', 'checkin', 'manual')),
    cook_id INTEGER REFERENCES cooks(id),
    at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE proposals (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    payload TEXT NOT NULL,
    recipe_id INTEGER REFERENCES recipes(id),
    shown_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    outcome TEXT CHECK (outcome IN ('accepted', 'refused')),
    reason TEXT
);

CREATE TABLE taste (
    id INTEGER PRIMARY KEY,
    line TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('rom', 'refusal')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE shopping (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    qty REAL,
    unit TEXT,
    done_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX movements_item ON movements(item_id);
CREATE INDEX items_expiry ON items(expires_on);
