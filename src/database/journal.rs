use std::path::PathBuf;
use rusqlite::{Result, Connection, Row, ToSql};
use crate::config::CONFIG;

pub struct Journal {
    pub id: Option<i32>,
    pub name: String,
    pub abbr: String,
    pub abbr_no_dot: String,
}

impl Journal {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Journal {
            id: Some(row.get_unwrap(0)),
            name: row.get_unwrap(1),
            abbr: row.get_unwrap(2),
            abbr_no_dot: row.get_unwrap(3),
        })
    }
    pub fn from_list(str_list: Vec<String>) -> Result<Self> {
        if str_list.len() < 3 {
            Err("Journal needs full name, abbreviation and abbreviation without dot")
        } else {
            Ok(Journal{id: None, name: str_list[0], abbr: str_list[1], abbr_no_dot: str_list[2]})
        }
    }
}

pub struct JournalDB {
    conn: Connection,
}

impl JournalDB {
    const SEARCH_QUERY: &'static str = "SELECT rowid, name, abbr, abbr_no_dot FROM journal \
        WHERE journal MATCH ? ORDER BY LENGTH(name) LIMIT 1;";

    pub fn new(path: Option<PathBuf>) -> Self {
        let db_path = path.unwrap_or_else(|| CONFIG.database.clone());
        let conn = Connection::open(&db_path).unwrap_or_else(
            |_| panic!("Cannot open sqlite file at {}!", db_path.to_string_lossy()));
        conn.pragma_update(None, "foreign_keys", &"ON").unwrap();
        JournalDB{conn}
    }

    pub fn search<T: AsRef<str> + ToSql + ?Sized>(&self, name: &T) -> Result<Journal> {
        self.conn.query_row_and_then(JournalDB::SEARCH_QUERY, &[&name], Journal::from_row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_search() {
        let db = JournalDB::new(Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/data/journal.sqlite")));
        let result = db.search("nature neuroscience").unwrap();
        assert_eq!(result.name, "Nature Neuroscience");
        assert_eq!(result.id.unwrap(), 7172);
    }
}
