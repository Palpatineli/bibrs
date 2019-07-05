use std::path::Path;
use rusqlite::{Result, OpenFlags, Connection};

struct Journal {
    id: Option<i32>,
    name: String,
    abbr: String,
    abbr_no_dot: String,
}

const SEARCH_QUERY: &'static str = "SELECT rowid, name, abbr, abbr_no_dot FROM journal WHERE journal MATCH ? ORDER BY LENGTH(name) LIMIT 1;";

fn search_journal(name: &str) -> Result<Journal> {
    let conn = Connection::open_with_flags(Path::new("data/journal.sqlite"), OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    conn.query_row_and_then(SEARCH_QUERY, &[&name], |row| {
        Ok(Journal {
            id: Some(row.get_unwrap(0)),
            name: row.get_unwrap(1),
            abbr: row.get_unwrap(2),
            abbr_no_dot: row.get_unwrap(3),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_search() {
        let result = search_journal("nature neuroscience").unwrap();
        assert_eq!(result.name, "Nature Neuroscience");
        assert_eq!(result.id.unwrap(), 7172);
    }
}
