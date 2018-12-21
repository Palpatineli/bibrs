mod journal;

use std::str;
use std::path::PathBuf;
use std::iter::FromIterator;
use std::collections::HashSet;
use rusqlite::{Connection, Result};
use rusqlite::types::Value::{Integer, Text};
use rusqlite::types::ToSql;

use crate::model::{Entry, Person};
use crate::entry_type::EntryType;
use crate::config::{CONFIG, Config};

pub enum ConfigOrPath {
    Config(Config),
    Path(PathBuf),
}

pub struct SqliteBibDB {
    conn: Connection,
}

pub trait BibDataBase {
    fn get_item(&self, id: &str) -> Result<Entry>;
    fn search(&self, authors: &[String], keywords: &[String]) -> Vec<Entry>;
    fn delete(&self, id: &str) -> Result<()>;
    fn add_keywords(&self, citation: &str, terms: &Vec<String>) -> Result<()>;
    fn del_keywords(&self, citation: &str, terms: &Vec<String>) -> Result<()>;
    fn get_files(&self, citation: &str) -> Vec<(String, String)>;
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

impl SqliteBibDB {
    pub fn new(inputs: Option<ConfigOrPath>) -> Self {
        let db_path = match inputs {
            Some(ConfigOrPath::Config(temp)) => temp.database,
            Some(ConfigOrPath::Path(temp)) => temp,
            None => CONFIG.database.clone(),
        };
        let conn = Connection::open(&db_path).expect(
            &format!("Cannot open sqlite file at {}!", db_path.to_string_lossy()));
        conn.execute("PRAGMA foreign_keys = ON", &[]).unwrap();
        SqliteBibDB{conn}
    }

    /// retrieve all authors and editors for one given item id
    /// Returns:
    ///     (authors, editors)
    fn get_people(&self, id: &str) -> (Vec<Person>, Vec<Person>)  {
        let mut query = self.conn.prepare_cached("SELECT is_editor, persons.id, last_name, first_name, search_term FROM item_persons JOIN persons ON item_persons.person_id=persons.id WHERE item_id=? ORDER BY is_editor, order_seq").unwrap();
        let people: Vec<(Person, bool)> = query.query_map(&[&id],
            |row| (Person {id:row.get(1), last_name: row.get(2), first_name: row.get(3), search_term: row.get::<_, String>(4)}, row.get(0))
        ).unwrap().collect::<Result<Vec<(Person, bool)>>>().unwrap();
        let (editors, authors): (Vec<(Person, bool)>, Vec<(Person, bool)>)
            = people.into_iter().partition(|(_, is_editor)| *is_editor);
        (authors.into_iter().map(|(person, _)| person).collect(), editors.into_iter().map(|(person, _)| person).collect())
    }

    fn get_keywords(&self, id: &str) -> Vec<String> {
        let mut query = self.conn.prepare_cached(
            "SELECT text FROM item_keywords JOIN keywords ON item_keywords.keyword_id=keywords.id WHERE item_id=? ORDER BY text ASC").unwrap();
        let result = query.query_map(&[&id], |row| row.get(0)).unwrap().collect::<Result<Vec<String>>>().unwrap();
        result
    }

    fn get_extra_fields(&self, id: &str) -> Vec<(String, String)> {
        let mut query = self.conn.prepare_cached("SELECT field, value FROM extra_fields WHERE item_id=?").unwrap();
        let result = query.query_map(&[&id], |row| (row.get(0), row.get(1)))
            .unwrap().collect::<Result<Vec<(String, String)>>>().unwrap();
        result
    }

    /// Get the list of non-existing keywords (to add) and the ids of existing but not related
    /// keywords (to add relation)
    /// Returns:
    ///     (Vec<non_existing_keys>, Vec<unrelated_existing_keys>)
    fn exist_keywords(&self, citation: &str, terms: &Vec<String>) -> (Vec<String>, Vec<i64>) {
        let mut query = self.conn.prepare_cached(
            &format!("
                SELECT id, text
                  FROM keywords
                 WHERE text IN ({})", multi_param!(terms.len()))).unwrap();
        let rows = query.query_map(&(terms.iter().map(|x| x as &ToSql).collect::<Vec<&ToSql>>()),
            |row| (row.get::<_, i64>(0), row.get::<_, String>(1)))
            .unwrap().collect::<Result<Vec<(i64, String)>>>().unwrap();
        let mut ids: Vec<i64> = Vec::new();
        let mut texts: Vec<String> = Vec::new();
        for (id, text) in rows.into_iter() {ids.push(id); texts.push(text);}
        let non_existing: Vec<String> = terms.iter().filter(|x| !texts.contains(x)).map(|x| x.clone()).collect();
        (non_existing, ids)
    }
}

macro_rules! get_col {
    ($row:ident, $no:expr, Integer) => {
        if let Integer(temp) = $row.get($no) {Some(temp as i32)} else {None}
    };
    ($row:ident, $no:expr, Text) => {
        if let Text(temp) = $row.get($no) {Some(temp as String)} else {None}
    }
}

impl BibDataBase for SqliteBibDB {
    fn get_item(&self, id: &str) -> Result<Entry> {
        const ITEM_QUERY: &str = "
            SELECT citation, entry_type, title, booktitle, year, month, chapter, edition, volume, \"number\", pages,
                   journals.name
              FROM items
                   LEFT JOIN journals ON items.journal_id=journals.id
             WHERE citation = ?
             LIMIT 1";
        let mut query = self.conn.prepare_cached(ITEM_QUERY).unwrap();
        query.query_row(&[&id],
            |row| {
                let (authors, editors) = self.get_people(id);
                Entry{
                    citation: row.get::<_, String>(0),
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
                    keywords: self.get_keywords(id),
                    files: Vec::new(),
                    extra_fields: self.get_extra_fields(id),
                }
            }
        )
    }

    fn get_files(&self, citation: &str) -> Vec<(String, String)> {
        const FILE_QUERY: &str = "
            SELECT name, object_type
              FROM file
             WHERE item_id=?";
        let mut file_query = self.conn.prepare_cached(FILE_QUERY).unwrap();
        let files = file_query.query_map(&[&citation], |row| (row.get::<_, String>(0), row.get::<_, String>(1))).unwrap()
            .collect::<Result<Vec<(String, String)>>>().unwrap();
        files
    }

    fn delete(&self, id: &str) -> Result<()> {
        const DELETE_QUERY: &str = "
            DELETE i, ip 
              FROM items AS i
             INNER JOIN item_persons AS ip
                ON items.citation=item_persons.item_id
             WHERE items.citation=?";
        self.conn.execute(DELETE_QUERY, &[&id]).map(|_| ())
    }

    fn search(&self, authors: &[String], keywords: &[String]) -> Vec<Entry> {
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
        let mut query = self.conn.prepare_cached(&query_str).unwrap();
        let results = query.query_map(&terms, |row| row.get::<_, String>(0)).unwrap()
            .map(|term| self.get_item(&(term.unwrap()))).collect::<Result<Vec<Entry>>>().unwrap();
        results
    }

    fn add_keywords(&self, citation: &str, terms: &Vec<String>) -> Result<()> {
        let entry = self.get_item(&citation)?;
        let new_terms = terms.iter().filter(|x| !entry.keywords.contains(x))
            .map(|x| x.clone()).collect();
        let (unexist, unrelated_ids) = self.exist_keywords(&citation, &new_terms);
        let mut query_insert_key = self.conn.prepare_cached("INSERT INTO keywords (text) VALUES (?)").unwrap();
        let row_ids: Vec<i64> = unexist.iter().map(|x| query_insert_key.insert(&[x]).unwrap()).collect();
        let mut query_insert_relation = self.conn.prepare_cached("INSERT INTO item_keywords (item_id, keyword_id) VALUES (?, ?)").unwrap();
        for id in unrelated_ids.iter().chain(row_ids.iter()) {
            query_insert_relation.execute(&[&citation, id])?;
        }
        Ok(())
    }

    /// Delete keywords associations
    fn del_keywords(&self, citation: &str, terms: &Vec<String>) -> Result<()> {
        let mut query_del_relation = self.conn.prepare_cached(
            &format!("
                DELETE FROM item_keywords
                 WHERE rowid IN (
                     SELECT item_keywords.rowid
                       FROM item_keywords
                       JOIN keywords ON item_keywords.keyword_id=keywords.id
                      WHERE item_keywords.item_id=?
                        AND keywords.text IN ({})
                 )", multi_param!(terms.len()))).unwrap();
        let mut params: Vec<&ToSql> = vec![&citation];
        let sql_terms = terms.iter().map(|x| x as &ToSql).collect::<Vec<&ToSql>>();
        params.extend(sql_terms.into_iter());
        query_del_relation.execute(&params)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read_config;

    macro_rules! vec_str {
        ($($word:expr),+) => {{
                let mut output: Vec<String> = Vec::new();
                $(output.push($word.to_owned());)+
                output
        }}
    }

    #[test]
    fn test_get() {
        // multi_param
        assert_eq!(multi_param!(3), "?, ?, ?");
        // test get people
        let conn = SqliteBibDB::new(Some(
                ConfigOrPath::Config(read_config(Some("test/data/bibrs-test.toml".into())))));
        let (authors, editors) = conn.get_people("stein2004");
        assert_eq!(editors[0].id.unwrap(), 1878);
        assert_eq!(authors[3].last_name, "vaughan");
        assert_eq!(authors[2].search_term, "wallace");
        // test keywords
        let keywords = conn.get_keywords("sholl1953");
        assert_eq!(keywords, vec_str!["cat", "morphology"]);
        // test extra fields
        let fields = conn.get_extra_fields("walker1938");
        assert_eq!(fields, vec![("note".to_owned(), "pulvinar structure at 48-56, connection at 187-190".to_owned()),
                                ("publisher".to_owned(), "University of Chicago press".to_owned())]);
        // test get item
        let entry = conn.get_item("stein2004").unwrap();
        assert_eq!(entry.citation, "stein2004");
        assert_eq!(entry.entry_type, EntryType::Incollection);
        assert_eq!(entry.authors[1].last_name, "stanford");
        assert_eq!(entry.keywords[0], "multisensory");
        // test search
        let entries = conn.search(&vec_str!["casagrande", "rosa"], &Vec::<String>::new());
        let ref entry = &entries[0];
        println!("entry: {}", entry.to_str());
        assert_eq!(entry.authors[0].id, Some(690));
        assert_eq!(entry.authors[1].last_name, "casagrande");
        assert_eq!(entry.journal, Some("Journal of Clinical Neurophysiology".to_owned()));
    }
    #[test]
    fn test_keywords() {
        // test add_keywords
        let conn = SqliteBibDB::new(Some(
                ConfigOrPath::Config(read_config(Some("test/data/bibrs-test.toml".into())))));
        conn.add_keywords("walker1938", &vec_str!["pulvinar", "thalamus", "macaque", "atlas", "bullshit"]).expect("can't add keywrods");
        conn.del_keywords("walker1938", &vec_str!["bullshit", "atlas", "review"])
            .expect("can't delete keywords");
        let entry = conn.get_item("walker1938").unwrap();
        let mut keywords = entry.keywords.clone();
        keywords.sort();
        assert_eq!(keywords, vec_str!["macaque", "pulvinar", "thalamus"]);
        conn.del_keywords("walker1938", &vec_str!["pulvinar", "thalamus", "macaque"])
            .expect("can't delete additional keywords");
        let entry = conn.get_item("walker1938").unwrap();
        println!("Leftover keywords include: {}", entry.keywords.join(", "));
    }
}
