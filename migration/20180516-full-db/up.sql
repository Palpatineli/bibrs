PRAGMA foreign_keys = ON;
CREATE TABLE items (
    citation VARCHAR(50) PRIMARY KEY,
    entry_type VARCHAR(15) NOT NULL,
    title VARCHAR(150) NOT NULL,
    year INTEGER NOT NULL,
    month INTEGER,
    pages VARCHAR(50),
    doi VARCHAR(50),
    volume INTEGER,
    "number" INTEGER,
    edition INTEGER,
    booktitle VARCHAR(200),
    chapter INTEGER,
    url VARCHAR(200),
    journal_id INTEGER,
    UNIQUE (title),
    UNIQUE (doi),
    UNIQUE (url),
    FOREIGN KEY(journal_id) REFERENCES journals (id)
);

CREATE TABLE extra_fields (
    item_id VARCHAR(50) NOT NULL,
    field VARCHAR(50) NOT NULL,
    value VARCHAR(200) NOT NULL,
    PRIMARY KEY (item_id, field),
    FOREIGN KEY (item_id) REFERENCES items (id)
);

CREATE TABLE "files" (
    item_id VARCHAR(50) NOT NULL,
    name VARCHAR(150) NOT NULL,
    "note" VARCHAR(50),
    object_type VARCHAR(50),
    FOREIGN KEY(item_id) REFERENCES items (id)
);

CREATE INDEX x_files_item_id ON "files" (item_id);

CREATE TABLE persons (
    id INTEGER NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    first_name VARCHAR(50),
    search_term VARCHAR(50) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (last_name, first_name)
);

CREATE INDEX x_persons_search_term ON persons (search_term);

CREATE TABLE keywords (
    id INTEGER PRIMARY KEY,
    text VARCHAR(50),
    UNIQUE (text)
);

CREATE UNIQUE INDEX x_keywords_text ON keywords (text);

CREATE TABLE journals (
    id INTEGER PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    abbr VARCHAR UNIQUE NOT NULL,
    abbr_no_dot VARCHAR UNIQUE NOT NULL
);

CREATE TABLE item_keywords (
    item_id VARCHAR(50),
    keyword_id INTEGER,
    PRIMARY KEY (item_id, keyword_id),
    FOREIGN KEY(item_id) REFERENCES items (id),
    FOREIGN KEY(keyword_id) REFERENCES keywords (id)
);

CREATE TABLE item_persons (
    item_id VARCHAR(50),
    person_id INTEGER,
    order_seq INTEGER,
    is_editor BOOLEAN NOT NULL CHECK (is_editor IN (0, 1)) DEFAULT 0,
    PRIMARY KEY (item_id, person_id),
    UNIQUE (item_id, order_seq, is_editor),
    FOREIGN KEY(item_id) REFERENCES items (id),
    FOREIGN KEY(person_id) REFERENCES persons (id)
);

CREATE TRIGGER lose_authorship
    AFTER DELETE ON item_persons WHEN (
        NOT EXISTS (
            SELECT *
              FROM item_persons
             WHERE person_id=OLD.person_id
        )
    ) 
BEGIN
    DELETE FROM persons 
     WHERE persons.id=OLD.author_id;
END;

CREATE TRIGGER lose_keyword
    AFTER DELETE ON item_keywords WHEN (
        NOT EXISTS (
            SELECT *
              FROM item_keywords 
             WHERE keyword_id=OLD.keyword_id
        )
    ) 
BEGIN
    DELETE FROM keywords
     WHERE keywords.id=OLD.keyword_id;
END;
