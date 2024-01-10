pub mod journal;
pub mod add_item;

use std::str;
use std::path::PathBuf;
use std::collections::{HashSet, HashMap};
use std::convert::From;
use rusqlite::{params, Connection, Result, Row};
use rusqlite::types::ToSql;

use crate::model::{Entry, Person};
use crate::entry_type::EntryType;
use crate::config::CONFIG;
use journal::Journal;

impl From<&Row<'_>> for Person {
    fn from(row: &Row) -> Person {
        Person{
            id: row.get_unwrap::<_, Option<i32>>(1),
            last_name: row.get_unwrap(2),
            first_name: row.get_unwrap(3),
            search_term: row.get_unwrap::<_, String>(4)}
    }
}

impl From<&Row<'_>> for Entry {
    fn from(row: &Row) -> Entry {
        Entry{
            citation: row.get_unwrap::<_, String>(0),
            entry_type: EntryType::parse(&row.get_unwrap::<_, String>(1)),
            title: row.get_unwrap::<_, String>(2),
            booktitle: row.get_unwrap::<_, Option<String>>(3),
            year: row.get_unwrap::<_, i32>(4),
            month: row.get_unwrap::<_, Option<i32>>(5),
            chapter: row.get_unwrap::<_, Option<i32>>(6),
            edition: row.get_unwrap::<_, Option<i32>>(7),
            volume: row.get_unwrap::<_, Option<i32>>(8),
            number: row.get_unwrap::<_, Option<i32>>(9),
            pages: row.get_unwrap::<_, Option<String>>(10),
            journal: row.get_unwrap::<_, Option<String>>(11),
            authors: Vec::new(),
            editors: Vec::new(),
            keywords: HashSet::new(),
            files: Vec::new(),
            extra_fields: HashMap::new(),
        }
    }
}

pub struct SqliteBibDB {
    conn: Connection,
}

pub trait BibDataBase {
    fn add_item(&self, entry: &Entry, journal_id: Option<i32>) -> Result<()>;
    fn get_item(&self, id: &str) -> Result<Entry>;
    fn search(&self, authors: &[String], keywords: &[String]) -> Result<Vec<Entry>>;
    fn delete(&self, id: &str) -> Result<()>;
    fn add_keywords<T: AsRef<str>>(&self, citation: &str, terms: &[T]) -> Result<()>;
    fn del_keywords<T: AsRef<str>>(&self, citation: &str, terms: &[T]) -> Result<()>;
    fn get_files(&self, citation: &str) -> Result<Vec<(String, String)>>;
    fn add_file(&self, citation: &str, name: &str, file_type: &str) -> Result<()>;
    fn query_journal<T: AsRef<str>>(&self, name: T) -> Result<i32>;
    fn add_journal(&self, journal: Journal) -> Result<i32>;
}

/// insert a number of question marks
macro_rules! multi_param {
    ($no:expr) => {{
        let mut out: Vec<&str> = Vec::new();
        for _idx in 0..$no { out.push("?"); }
        out.join(", ")
    }}
}

/// params! for vectors
macro_rules! build_param {
    ($(($terms:ident, $no:expr)),+) => {{
        let mut out: Vec<&dyn ToSql> = Vec::new();
        $(
            out.append(&mut $terms.iter().map(|x| x as &dyn ToSql).collect());
            out.push($no);
        )+
        out
    }}
}

impl SqliteBibDB {
    pub fn new(inputs: Option<PathBuf>) -> Self {
        let db_path = inputs.unwrap_or(CONFIG.database.clone());
        let conn = Connection::open(&db_path).expect(
            &format!("Cannot open sqlite file at {}!", db_path.to_string_lossy()));
        conn.pragma_update(None, "foreign_keys", &"ON").unwrap();
        SqliteBibDB{conn}
    }

    /// retrieve all authors and editors for one given item id
    /// Returns:
    ///     (authors, editors)
    fn get_people(&self, id: &str) -> (Vec<Person>, Vec<Person>)  {
        let mut query = self.conn.prepare_cached("SELECT is_editor, persons.id, last_name, first_name, search_term FROM item_persons JOIN persons ON item_persons.person_id=persons.id WHERE item_id=? ORDER BY is_editor, order_seq").unwrap();
        let people: Vec<(Person, bool)> = query.query_map(&[&id],
            |row| (Ok((Person {
                id:row.get_unwrap(1),
                last_name: row.get_unwrap(2),
                first_name: row.get_unwrap(3),
                search_term: row.get_unwrap::<_, String>(4)
            }, row.get_unwrap(0))))
        ).unwrap().collect::<Result<Vec<(Person, bool)>>>().unwrap();
        let (editors, authors): (Vec<(Person, bool)>, Vec<(Person, bool)>)
            = people.into_iter().partition(|(_, is_editor)| *is_editor);
        (authors.into_iter().map(|(person, _)| person).collect(), editors.into_iter().map(|(person, _)| person).collect())
    }

    /// get people with the same last name
    pub fn search_lastname(&self, search_term: &str) -> Result<Vec<Person>> {
        let mut query = self.conn.prepare_cached("SELECT id, last_name, first_name, search_term FROM persons WHERE search_term = ? ORDER BY first_name;")?;
        let persons = query.query_map(&[&search_term], |row| Ok(Person::from(row)))?.collect::<Result<Vec<Person>>>();
        persons
    }

    pub fn search_person(&self, person: &Person) -> Result<Person> {
        let mut query = self.conn.prepare_cached("SELECT id, last_name, first_name, search_term FROM persons WHERE search_term = ? AND first_name = ?;")?;
        query.query_row(&[&person.search_term, &person.first_name], |row| Ok(Person::from(row)))
    }

    pub fn add_person(&self, person: &Person) -> Result<i32> {
        let mut query = self.conn.prepare_cached("INSERT INTO persons (last_name, first_name, search_term) VALUES (?, ?, ?);")?;
        query.insert(&[&person.last_name, &person.first_name, &person.search_term]).map(|x| x as i32)
    }

    fn get_keywords(&self, id: &str) -> Vec<String> {
        let mut query = self.conn.prepare_cached(
            "SELECT text FROM item_keywords JOIN keywords ON item_keywords.keyword_id=keywords.id WHERE item_id=? ORDER BY text ASC").unwrap();
        let result = query.query_map(&[&id], |row| row.get(0)).unwrap().collect::<Result<Vec<String>>>().unwrap();
        result
    }

    fn get_extra_fields(&self, id: &str) -> Vec<(String, String)> {
        let mut query = self.conn.prepare_cached("SELECT field, value FROM extra_fields WHERE item_id=?").unwrap();
        let result = query.query_map(&[&id], |row| Ok((row.get_unwrap(0), row.get_unwrap(1))))
            .unwrap().collect::<Result<Vec<(String, String)>>>().unwrap();
        result
    }

    /// Get the list of non-existing keywords (to add) and the ids of existing but not related
    /// keywords (to add relation)
    /// Returns:
    ///     (Vec<non_existing_keys>, Vec<unrelated_existing_keys>)
    fn exist_keywords<T: AsRef<str>>(&self, terms: &[T]) -> (Vec<String>, Vec<i64>) {
        let mut query = self.conn.prepare_cached(
            &format!("
                SELECT id, text
                  FROM keywords
                 WHERE text IN ({})", multi_param!(terms.len()))).unwrap();
        let search_terms = terms.iter().map(|x| x.as_ref()).collect::<Vec<&str>>();
        let mut ids: Vec<i64> = Vec::new();
        let mut texts: Vec<String> = Vec::new();
        query.query_map(&search_terms, |row| Ok((row.get_unwrap::<_, i64>(0), row.get_unwrap::<_, String>(1)))).unwrap()
            .for_each(|r| { let (x, y) = r.unwrap(); ids.push(x); texts.push(y)});
        let non_existing: Vec<String> = terms.iter().map(|x| x.as_ref().to_string()).filter(|x| !texts.contains(x)).collect();
        (non_existing, ids)
    }

    fn add_extra_fields(&self, citation: &str, extra_fields: &HashMap<String, String>) -> Result<()> {
        let mut insert_query = self.conn.prepare_cached("REPLACE INTO extra_fields (item_id, field, value) VALUES (?, ?, ?)")?;
        for (field, value) in extra_fields.iter() {
            insert_query.insert(&[citation, &field, &value])?;
        };
        Ok(())
    }
}

impl BibDataBase for SqliteBibDB {
    fn add_item(&self, entry: &Entry, journal_id: Option<i32>) -> Result<()> {
        let mut insert_query = self.conn.prepare_cached("
            INSERT INTO items (citation, entry_type, title, booktitle, year, month, chapter, edition,
                               volume, \"number\", pages, journal_id)
            VALUES (?,?,?,?,?,?,?,?,?,?,?,?);")?;
        insert_query.query(params![&entry.citation, &entry.entry_type.to_string(), &entry.title, &entry.booktitle, &entry.year,
            &entry.month, &entry.chapter, &entry.edition, &entry.volume, &entry.number, &entry.pages, &journal_id])?;
        let mut insert_relation = self.conn.prepare_cached("INSERT INTO item_persons (item_id, person_id, order_seq, is_editor) VALUES (?, ?, ?, ?);")?;
        for (people, is_editor) in [(&entry.authors, false), (&entry.editors, true)].iter() {
            for (order, person) in people.iter().enumerate() {
                let author_id = match self.search_person(&person) {
                    Ok(x) => x.id.expect("existing person row doesn't have rowid?"),
                    Err(_) => self.add_person(person)?,
                };
                insert_relation.insert(params![&entry.citation, &author_id, &(order as isize), &is_editor])?;
            }
        }
        self.add_keywords(&entry.citation, &entry.keywords.iter().cloned().collect::<Vec<String>>())?;
        self.add_extra_fields(&entry.citation, &entry.extra_fields)?;
        for (name, file_type) in entry.files.iter() { self.add_file(&entry.citation, name, file_type)?; }
        Ok(())
    }

    fn get_item(&self, id: &str) -> Result<Entry> {
        let mut query = self.conn.prepare_cached("
            SELECT citation, entry_type, title, booktitle, year, month, chapter, edition, volume, \"number\", pages,
                   journals.name
              FROM items
                   LEFT JOIN journals ON items.journal_id=journals.id
             WHERE citation = ?
             LIMIT 1")?;
        query.query_row(&[&id],
            |row| {
                let (authors, editors) = self.get_people(id);
                let mut entry = Entry::from(row);
                entry.authors.extend(authors.into_iter());
                entry.editors.extend(editors.into_iter());
                entry.keywords.extend(self.get_keywords(id).into_iter());
                entry.extra_fields.extend(self.get_extra_fields(id).into_iter());
                Ok(entry)
            }
        )
    }

    fn get_files(&self, citation: &str) -> Result<Vec<(String, String)>> {
        let mut file_query = self.conn.prepare_cached("
            SELECT name, object_type
              FROM files
             WHERE item_id=?")?;
        let files = file_query.query_map(&[&citation], |row| Ok((row.get_unwrap::<_, String>(0), row.get_unwrap::<_, String>(1)))).unwrap()
            .collect::<Result<Vec<(String, String)>>>()?;
        Ok(files)
    }

    fn add_file(&self, citation: &str, name: &str, file_type: &str) -> Result<()> {
        let mut insert_query = self.conn.prepare_cached("INSERT INTO files (item_id, name, object_type) VALUES (?, ?, ?)")?;
        insert_query.insert(params![citation, name, file_type])?;
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<()> {
        let mut query = self.conn.prepare_cached("
            DELETE i, ip, ik
              FROM items AS i
             INNER JOIN item_persons AS ip
             INNER JOIN item_keywords AS ik
                ON items.citation=item_persons.item_id
                ON items.citation=item_keywords.item_id
             WHERE items.citation=?")?;
        query.query(&[&id]).map(|_| ())
    }

    fn search(&self, authors: &[String], keywords: &[String]) -> Result<Vec<Entry>> {
        let author_no = authors.len() as isize;
        let keyword_no = keywords.len() as isize;
        let (query_str, terms): (String, Vec<&dyn ToSql>) = if author_no > 0 {
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
        let mut query = self.conn.prepare_cached(&query_str)?;
        let results = query.query_map(&terms, |row| row.get::<_, String>(0))?
            .map(|term| self.get_item(&(term?))).collect::<Result<Vec<Entry>>>()?;
        Ok(results)
    }

    fn add_keywords<T: AsRef<str>>(&self, citation: &str, terms: &[T]) -> Result<()> {
        let (unexist, unrelated_ids) = self.exist_keywords(terms);
        let mut query_insert_key = self.conn.prepare_cached("INSERT INTO keywords (text) VALUES (?)").unwrap();
        let row_ids: Vec<i64> = unexist.iter().map(|x| query_insert_key.insert(&[x]).unwrap()).collect();
        let mut query_insert_relation = self.conn.prepare_cached("INSERT INTO item_keywords (item_id, keyword_id) VALUES (?, ?)").unwrap();
        for id in unrelated_ids.iter().chain(row_ids.iter()) { query_insert_relation.execute(params![citation, id])?; }
        Ok(())
    }

    /// Delete keywords associations
    fn del_keywords<T: AsRef<str>>(&self, citation: &str, terms: &[T]) -> Result<()> {
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
        let mut params: Vec<&dyn ToSql> = vec![&citation];
        let sql_terms: Vec<String> = terms.iter().map(|x| x.as_ref().to_string()).collect();
        sql_terms.iter().for_each(|x| params.push(x));
        query_del_relation.execute(&params)?;
        Ok(())
    }

    fn add_journal(&self, journal: Journal) -> Result<i32> {
        let mut insert = self.conn.prepare_cached("INSERT INTO journals (name, abbr, abbr_no_dot) VALUES (?, ?, ?);")?;
        insert.insert(&[journal.name, journal.abbr, journal.abbr_no_dot]).map(|x| x as i32)
    }

    /// query for journal_id in the main database by a name as either full name or abbreviation.
    /// If the journal does not exist in the main data base, insert the journal library entry into
    /// the main database if it exists. If the journal does not exist in the journal library, raise
    /// an error.
    fn query_journal<T: AsRef<str>>(&self, name: T) -> Result<i32>{
        let mut query = self.conn.prepare_cached("
            SELECT * FROM journals AS journals_full WHERE journals_full.name = ?
            UNION SELECT * FROM journals AS journals_abbr WHERE journals_abbr.abbr = ?
            UNION SELECT * FROM journals AS journals_abbr_nd WHERE journals_abbr_nd.abbr_no_dot LIKE ?;")?;
        let journal = query.query_row(params![name.as_ref(), name.as_ref(), &format!("%{}%", name.as_ref())], Journal::from_row);
        match journal {
            Ok(journal) => Ok(journal.id.unwrap()),
            Err(_) => { self.add_journal(Journal::search(name.as_ref())?) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;
    use itertools::Itertools;
    use crate::config::Config;

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
        let conn = SqliteBibDB::new(Some(Config::new(Some("test/data/bibrs-test.toml".into())).database));
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
        assert!(entry.keywords.contains("multisensory"));
        // test search
        let entries = conn.search(&vec_str!["casagrande", "rosa"], &Vec::<String>::new()).expect("search fail at the db level!");
        let ref entry = &entries[0];
        println!("entry: {}", entry.to_str());
        assert_eq!(entry.authors[0].id, Some(690));
        assert_eq!(entry.authors[1].last_name, "casagrande");
        assert_eq!(entry.journal, Some("Journal of Clinical Neurophysiology".to_owned()));
    }
    #[test]
    fn test_keywords() {
        // test add_keywords
        let conn = SqliteBibDB::new(Some(Config::new(Some("test/data/bibrs-test.toml".into())).database));
        conn.add_keywords("walker1938", &vec_str!["pulvinar", "thalamus", "macaque", "atlas", "bullshit"]).expect("can't add keywrods");
        conn.del_keywords("walker1938", &vec_str!["bullshit", "atlas", "review"])
            .expect("can't delete keywords");
        let entry = conn.get_item("walker1938").unwrap();
        let keywords = entry.keywords.clone();
        assert_eq!(keywords, HashSet::from_iter(["macaque", "pulvinar", "thalamus"].iter().map(|x| x.to_string())));
        conn.del_keywords("walker1938", &vec_str!["pulvinar", "thalamus", "macaque"])
            .expect("can't delete additional keywords");
        let entry = conn.get_item("walker1938").unwrap();
        println!("Leftover keywords include: {}", entry.keywords.iter().join(", "));
    }
}
