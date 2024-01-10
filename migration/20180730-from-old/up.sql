CREATE TABLE items (
    citation VARCHAR(50) PRIMARY KEY,
    entry_type VARCHAR(15) NOT NULL,
    title VARCHAR(150) NOT NULL,
    booktitle VARCHAR(200),
    year INTEGER NOT NULL,
    month INTEGER,
    chapter INTEGER,
    edition INTEGER,
    volume INTEGER,
    "number" INTEGER,
    pages VARCHAR(50),
    journal_id INTEGER,
    UNIQUE (title),
    FOREIGN KEY(journal_id) REFERENCES journals (id)
);

CREATE TABLE extra_fields (
    item_id VARCHAR(50) NOT NULL,
    field VARCHAR(50) NOT NULL,
    value VARCHAR(200) NOT NULL,
    PRIMARY KEY (item_id, field),
    FOREIGN KEY (item_id) REFERENCES items (citation)
);

INSERT INTO items (citation, entry_type, title, year, booktitle, chapter,
                   edition, month, "number", volume, pages, journal_id)
SELECT id, object_type, title, year, booktitle, chapter, edition, month,
       "number", volume, pages, journal_id
  FROM item;

INSERT INTO extra_fields (item_id, field, value)
SELECT id, 'doi', doi 
  FROM item WHERE doi IS NOT NULL AND doi != "";

INSERT INTO extra_fields (item_id, field, value)
SELECT id, 'eprint', eprint
  FROM item
 WHERE eprint IS NOT NULL AND eprint != "";

INSERT INTO extra_fields (item_id, field, value)
SELECT id, 'organization', organization
  FROM item
 WHERE organization IS NOT NULL AND organization != "";

INSERT INTO extra_fields (item_id, field, value)
SELECT id, 'publisher', publisher
  FROM item
 WHERE publisher IS NOT NULL AND publisher != "";

INSERT INTO extra_fields (item_id, field, value)
SELECT id, 'institution', institution
  FROM item
 WHERE institution IS NOT NULL AND institution != "";

INSERT INTO extra_fields (item_id, field, value) SELECT id, 'series', series FROM item WHERE series IS NOT NULL AND series != "";
INSERT INTO extra_fields (item_id, field, value) SELECT id, 'address', address FROM item WHERE address IS NOT NULL AND address != "";
INSERT INTO extra_fields (item_id, field, value) SELECT id, 'note', "note" FROM item WHERE "note" IS NOT NULL AND "note" != "";
INSERT INTO extra_fields (item_id, field, value) SELECT id, 'howpublished', howpublished FROM item WHERE howpublished IS NOT NULL AND howpublished != "";
INSERT INTO extra_fields (item_id, field, value) SELECT id, 'school', school FROM item WHERE school IS NOT NULL AND school != "";

DROP TABLE IF EXISTS item;

CREATE TABLE "files" (
    item_id VARCHAR(50) NOT NULL,
    name VARCHAR(150) NOT NULL,
    "note" VARCHAR(50),
    object_type VARCHAR(50),
    FOREIGN KEY(item_id) REFERENCES items (citation)
);

CREATE INDEX x_files_item_id ON "files" (item_id);

INSERT INTO "files" (item_id, name, "note", object_type) SELECT item_id, name, "note", object_type FROM "file";

DROP TABLE "file";

CREATE TABLE persons (
    id INTEGER NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    first_name VARCHAR(50),
    search_term VARCHAR(50) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (last_name, first_name)
);

CREATE INDEX x_persons_search_term ON persons (search_term);

INSERT INTO persons (id, last_name, first_name, search_term) SELECT id, last_name, first_name, last_name FROM person;

DROP TABLE IF EXISTS person;

CREATE TABLE keywords (
    id INTEGER PRIMARY KEY,
    text VARCHAR(50),
    UNIQUE (text)
);

CREATE UNIQUE INDEX x_keywords_text ON keywords (text);

INSERT INTO keywords (id, text) SELECT id, text FROM keyword;

DROP TABLE IF EXISTS keyword;

CREATE TABLE journals (
    id INTEGER PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    abbr VARCHAR UNIQUE NOT NULL,
    abbr_no_dot VARCHAR UNIQUE NOT NULL
);

CREATE UNIQUE INDEX x_journal_name ON journals (name);

INSERT INTO journals (id, name, abbr, abbr_no_dot)
SELECT id, name, abbr, abbr_no_dot
  FROM journal;

DROP TABLE IF EXISTS journal;

CREATE TABLE item_keywords (
    item_id VARCHAR(50),
    keyword_id INTEGER,
    PRIMARY KEY (item_id, keyword_id),
    FOREIGN KEY(item_id) REFERENCES items (citation),
    FOREIGN KEY(keyword_id) REFERENCES keywords (id)
);

INSERT INTO item_keywords (item_id, keyword_id)
SELECT item_id, keyword_id
  FROM association;

DROP TABLE IF EXISTS association;

CREATE TABLE item_persons (
    item_id VARCHAR(50) NOT NULL,
    person_id INTEGER NOT NULL,
    order_seq INTEGER NOT NULL,
    is_editor BOOLEAN NOT NULL CHECK (is_editor IN (0, 1)) DEFAULT 0,
    PRIMARY KEY (item_id, person_id),
    UNIQUE (item_id, order_seq, is_editor),
    FOREIGN KEY(item_id) REFERENCES items (citation),
    FOREIGN KEY(person_id) REFERENCES persons (id)
);

INSERT INTO item_persons (item_id, person_id, order_seq, is_editor)
SELECT item_id, person_id, "order", false
  FROM authorship;

INSERT INTO item_persons (item_id, person_id, order_seq, is_editor)
SELECT item_id, person_id, "order", true
  FROM editorship;

DROP TABLE IF EXISTS authorship;
DROP TABLE IF EXISTS editorship;

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
