use std::path::Path;
use rusqlite::{Result, OpenFlags, Connection, Row};

pub struct Journal {
    pub id: Option<i32>,
    pub name: String,
    pub abbr: String,
    pub abbr_no_dot: String,
}

impl Journal {
    const SEARCH_QUERY: &'static str = "SELECT rowid, name, abbr, abbr_no_dot FROM journal WHERE journal MATCH ? ORDER BY LENGTH(name) LIMIT 1;";

    pub fn search(name: &str) -> Result<Self> {
        let conn = Connection::open_with_flags(Path::new("data/journal.sqlite"), OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        conn.query_row_and_then(Journal::SEARCH_QUERY, &[&name], Journal::from_row)
    }

    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Journal {
            id: Some(row.get_unwrap(0)),
            name: row.get_unwrap(1),
            abbr: row.get_unwrap(2),
            abbr_no_dot: row.get_unwrap(3),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_search() {
        let result = Journal::search("nature neuroscience").unwrap();
        assert_eq!(result.name, "Nature Neuroscience");
        assert_eq!(result.id.unwrap(), 7172);
    }
}
