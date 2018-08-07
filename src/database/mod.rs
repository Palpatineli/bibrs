mod journal;

use std::str;
use rusqlite::{Connection, Result};
use rusqlite::types::Value::{Integer, Text};
use rusqlite::types::{ToSql, ToSqlOutput};
use model::{Entry, Person};
use entry_type::EntryType;
use config::{CONFIG, read_config, Config};

/// retrieve all authors and editors for one given item id
/// Returns:
///     (authors, editors)
fn get_people(conn: &Connection, search_term: &str) -> (Vec<Person>, Vec<Person>) {
    let mut query = conn.prepare_cached("SELECT is_editor, persons.id, last_name, first_name, search_term FROM item_persons JOIN persons ON item_persons.person_id=persons.id WHERE item_id=? ORDER BY is_editor, order_seq").unwrap();
    let people: Vec<(Person, bool)> = query.query_map(&[&search_term],
        |row| (Person {id:row.get(1), last_name: row.get(2), first_name: row.get(3), search_term: row.get::<_, String>(4).into_bytes()}, row.get(0))
    ).unwrap().collect::<Result<Vec<(Person, bool)>>>().unwrap();
    let (editors, authors): (Vec<(Person, bool)>, Vec<(Person, bool)>)
        = people.into_iter().partition(|(_, is_editor)| *is_editor);
    (authors.into_iter().map(|(person, _)| person).collect(), editors.into_iter().map(|(person, _)| person).collect())
}

fn get_keywords(conn: &Connection, search_term: &str) -> Vec<String> {
    let mut query = conn.prepare_cached(
        "SELECT text FROM item_keywords JOIN keywords ON item_keywords.keyword_id=keywords.id WHERE item_id=? ORDER BY text").unwrap();
    let result = query.query_map(&[&search_term], |row| row.get(0)).unwrap().collect::<Result<Vec<String>>>().unwrap();
    result
}

fn get_extra_fields(conn: &Connection, search_term: &str) -> Vec<(String, String)> {
    let mut query = conn.prepare_cached("SELECT field, value FROM extra_fields WHERE item_id=?").unwrap();
    let result = query.query_map(&[&search_term], |row| (row.get(0), row.get(1)))
        .unwrap().collect::<Result<Vec<(String, String)>>>().unwrap();
    result
}

macro_rules! get_col {
    ($row:ident, $no:expr, Integer) => {
        if let Integer(temp) = $row.get($no) {Some(temp as i32)} else {None}
    };
    ($row:ident, $no:expr, Text) => {
        if let Text(temp) = $row.get($no) {Some(temp as String)} else {None}
    }
}

const ITEM_QUERY: &str = "
    SELECT citation, entry_type, title, booktitle, year, month, chapter, edition, volume, \"number\", pages,
           journals.name
      FROM items
           LEFT JOIN journals ON items.journal_id=journals.id
     WHERE citation = ?
     LIMIT 1";

pub fn get_item(conn: &Connection, search_term: &str) -> Result<Entry> {
    let mut query = conn.prepare(ITEM_QUERY).unwrap();
    query.query_row(&[&search_term],
        |row| {
            let (authors, editors) = get_people(&conn, search_term);
            Entry{
                citation: row.get::<_, String>(0).into_bytes(),
                entry_type: EntryType::parse(&row.get::<_, String>(1)),
                title: row.get(2),
                booktitle: get_col!(row, 3, Text),
                year: row.get(4),
                month: get_col!(row, 5, Integer),
                chapter: get_col!(row, 6, Integer),
                edition: get_col!(row, 7, Integer),
                volume: get_col!(row, 8, Integer),
                number: get_col!(row, 9, Integer),
                pages: get_col!(row, 10, Text),
                journal: get_col!(row, 11, Text),
                authors,
                editors,
                keywords: get_keywords(&conn, search_term),
                files: Vec::new(),
                extra_fields: get_extra_fields(&conn, search_term),
            }
        }
    )
}

macro_rules! multi_param {
    ($no:expr) => {{
        let mut out: Vec<&str> = Vec::new();
        for _idx in 0..$no { out.push("?"); }
        out.join(", ")
    }}
}

macro_rules! build_param {
    ($(($terms:ident, $no:expr)),+) => {{
        let mut out: Vec<&ToSql> = Vec::new();
        $(
            out.append(&mut $terms.iter().map(|x| x as &ToSql).collect());
            out.push($no);
        )+
        out
    }}
}

pub fn search(conn: &Connection, authors: &[String], keywords: &[String]) -> Vec<Entry> {
    let author_no = authors.len() as isize;
    let keyword_no = keywords.len() as isize;
    let (query_str, terms): (String, Vec<&ToSql>) = if author_no > 0 {
        if keyword_no > 0 {
            (format!("
                SELECT item_id
                  FROM item_persons
                       JOIN persons
                         ON person_id = persons.id
                 WHERE search_term IN ({})
                 GROUP BY item_id
                HAVING count(DISTINCT search_term) = ?
                INTERSECT
                SELECT item_id
                  FROM item_keywords
                       JOIN keywords
                         ON keyword_id=keywords.id
                 WHERE keywords.text IN ({})
                 GROUP BY item_id
                HAVING count(*) = ?",
                multi_param!(author_no), multi_param!(keyword_no)),
             build_param!((authors, &author_no), (keywords, &keyword_no)))
        } else {
            (format!(
                 "SELECT item_id
                    FROM item_persons
                         JOIN persons
                           ON person_id = persons.id
                   WHERE search_term IN ({})
                   GROUP BY item_id
                  HAVING count(DISTINCT search_term) = ?", multi_param!(author_no)),
             build_param!((authors, &author_no)))
        }
    } else if keyword_no > 0 {
        (format!(
            "SELECT item_id
               FROM item_keywords
                    JOIN keywords
                      ON keyword_id=keywords.id
              WHERE keywords.text IN ({})
              GROUP BY item_id
            HAVING count(*) = ?", multi_param!(keyword_no)),
         build_param!((keywords, &keyword_no)))
    } else { panic!("please search with authors and/or keywords!") };
    let mut query = conn.prepare(&query_str).unwrap();
    let results = query.query_map(&terms, |row| row.get::<_, String>(0)).unwrap()
        .map(|term| get_item(conn, &(term.unwrap()))).collect::<Result<Vec<Entry>>>().unwrap();
    results
}

pub fn connect(config: Option<Config>) -> Connection {
    let db_path = if let Some(temp) = config {temp.database} else {CONFIG.database.clone()};
    let conn = Connection::open(&db_path)
        .expect(&format!("{} is not a valid database!", db_path.to_string_lossy()));
    conn.execute("PRAGMA foreign_keys = ON", &[]).unwrap();
    conn
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    #[test]
    fn test_get() {
        // multi_param
        assert_eq!(multi_param!(3), "?, ?, ?");
        // test get people
        let conn = connect(Some(read_config(Some(PathBuf::from("test/data/bibrs-test.toml")))));
        let (authors, editors) = get_people(&conn, "stein2004");
        assert_eq!(editors[0].id.unwrap(), 1878);
        assert_eq!(authors[3].last_name, "vaughan");
        assert_eq!(authors[2].search_term, "wallace".as_bytes());
        // test keywords
        let keywords = get_keywords(&conn, "sholl1953");
        assert_eq!(keywords, vec!["cat".to_owned(), "morphology".to_owned()]);
        // test extra fields
        let fields = get_extra_fields(&conn, "walker1938");
        assert_eq!(fields, vec![("note".to_owned(), "pulvinar structure at 48-56, connection at 187-190".to_owned()),
                                ("publisher".to_owned(), "University of Chicago press".to_owned())]);
        // test get item
        let entry = get_item(&conn, "stein2004").unwrap();
        assert_eq!(str::from_utf8(&entry.citation).unwrap(), "stein2004");
        assert_eq!(entry.entry_type, EntryType::Incollection);
        assert_eq!(entry.authors[1].last_name, "stanford");
        assert_eq!(entry.keywords[0], "multisensory");
        // test search
        let entries = search(&conn, &vec!["casagrande".to_owned(), "rosa".to_owned()], &Vec::<String>::new());
        let ref entry = &entries[0];
        assert_eq!(entry.authors[0].id, Some(690));
        assert_eq!(entry.authors[1].last_name, "casagrande");
        assert_eq!(entry.journal, Some("Journal of Clinical Neurophysiology".to_owned()));
    }
}
