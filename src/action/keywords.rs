use std::collections::HashSet;
use itertools::Itertools;
use termion::{color, style};
use crate::{database::{SqliteBibDB, BibDataBase}, model::Entry};

macro_rules! fg {
    ($col:ident, $content:expr) => {
        format!("{}{}{}", color::Fg(color::$col), $content, color::Fg(color::Reset))
    }
}

/// add or delete keywords
/// returns a tuple (PaperItem, Added Keywords, Deleted Keywords, Kept Keywords)
pub fn keywords(citation: &str, mut add: HashSet<String>, mut del: HashSet<String>) {
    let conn = SqliteBibDB::new(None);
    let old_entry = conn.get_item(citation).expect(&format!("Cannot find entry {}", &citation));
    let has_new = if add.len() > 0 {
        let add_new: Vec<&String> = add.difference(&old_entry.keywords).collect();
        if add_new.len() > 0 { conn.add_keywords(&old_entry.citation, &add_new); true } else { false }
    } else { false };
    let has_del = if del.len() > 0 {
        let del_exist: Vec<&String> = del.intersection(&old_entry.keywords).collect();
        if del_exist.len() > 0 { conn.del_keywords(&old_entry.citation, &del_exist); true } else { false }
    } else { false };
    if !(has_new || has_del) {
        println!("{}\n\tKeywords: {}", old_entry.to_str(), old_entry.keywords.iter().join(", "));
        return
    }
    let new_entry = conn.get_item(citation).unwrap();
    let mut all_keywords: Vec<String> = new_entry.keywords.union(&old_entry.keywords).map(|x| x.to_string()).collect();
    all_keywords.sort();
    // TODO: change the return value and add the formating for this
    let keyword_str = all_keywords.iter().map(
        |x| if new_entry.keywords.contains(x) {
            if old_entry.keywords.contains(x) { x.to_string() }
            else { fg!(Red, x) }
        } else { fg!(Blue, format!("{}{}{}", style::Invert, x, style::Reset)) }).join(", ");
    println!("{}\n\tKeywords: {}", new_entry.to_str(), keyword_str);
}
