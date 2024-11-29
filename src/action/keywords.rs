use core::{fmt, panic};
use std::{collections::HashSet, fmt::Display};
use itertools::Itertools;
use termion::{color, style};
use crate::{database::{SqliteBibDB, BibDataBase}, model::Entry};

macro_rules! fg {
    ($col:ident, $content:expr) => {
        format!("{}{}{}", color::Fg(color::$col), $content, color::Fg(color::Reset))
    }
}

pub struct AlteredKeywords {
    kept: HashSet<String>,
    added: HashSet<String>,
    deleted: HashSet<String>
}

/// add or delete keywords
/// returns a tuple (PaperItem, Added Keywords, Deleted Keywords, Kept Keywords)
pub fn keywords(conn: &SqliteBibDB, citation: &str, add: HashSet<String>, del: HashSet<String>) 
    -> (Entry, AlteredKeywords) {
    let old_entry = conn.get_item(citation).unwrap_or_else(|_| panic!("Cannot find entry {}", &citation));
    if !add.is_empty() {
        let add_new: Vec<&String> = add.difference(&old_entry.keywords).collect();
        if !add_new.is_empty() { conn.add_keywords(&old_entry.citation, &add_new)
            .unwrap_or_else(|_| panic!("Failed to add keywords to {}", &old_entry.citation));
        }
    };
    if !del.is_empty() {
        let del_exist: Vec<&String> = del.intersection(&old_entry.keywords).collect();
        if !del_exist.is_empty() { conn.del_keywords(&old_entry.citation, &del_exist)
            .unwrap_or_else(|_| panic!("Failed to delete keywords from {}", &old_entry.citation));
        }
    };
    let new_entry = conn.get_item(citation).unwrap();
    let alteration = AlteredKeywords{
        kept: new_entry.keywords.intersection(&old_entry.keywords).map(|x| x.to_owned()).collect(),
        added: new_entry.keywords.difference(&old_entry.keywords).map(|x| x.to_owned()).collect(),
        deleted: old_entry.keywords.difference(&new_entry.keywords).map(|x| x.to_owned()).collect()
    };
    (new_entry, alteration)
}

impl Display for AlteredKeywords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output: Vec<String> = Vec::new();
        output.push(self.kept.iter().sorted().join(", "));
        output.push(self.added.iter().sorted().map(|x| fg!(Blue, x)).join(", "));
        output.push(self.deleted.iter().sorted().map(|x|
                fg!(Red, format!("{}{}{}", style::CrossedOut, x, style::Reset))).join(", "));
        write!(f, "Keywords: {}", output.join(" | "))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::formatter::ToString;
    use crate::str_hashset;
    use super::*;
    #[test]
    fn test_keywords() {
        let conn = SqliteBibDB::new(Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/data/library.sqlite")));
        let res = keywords(&conn, "casagrande1994",
            str_hashset!("circuit", "computation"),
            str_hashset!("visual cortex", "intrinsic"));
        let _ = res.1.to_string();
        let res = keywords(&conn, "casagrande1994",
            str_hashset!("visual cortex", "intrinsic"),
            str_hashset!("circuit", "computation"));
        let _ = res.1.to_string();
    }
}
