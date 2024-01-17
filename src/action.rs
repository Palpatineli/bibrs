#![allow(unused)]
use std::path::PathBuf;
use std::io;
use std::fs;

use termion::color;

use crate::formatter::{BibPrint, LabeledPrint};
use crate::model::Entry;
use crate::database::{SqliteBibDB, BibDataBase};
use crate::reader::{pandoc::read_pandoc, bibtex::read_entries};
use crate::config::{self, CONFIG};
use crate::file::{File, BibFile};

mod keywords;
mod add_item;
pub use add_item::add_item;
pub use self::keywords::keywords;

pub fn search(conn: &SqliteBibDB, mut author: Vec<String>, mut keywords: Vec<String>) -> String {
    author.retain(|x| x.len() > 0);
    keywords.retain(|x| x.len() > 0);
    if author.len() == 0 && keywords.len() == 0 {
        return "Search by author last names either/or keywords!".to_string();
    }
    let results = conn.search(&author, &keywords).expect("Search Fail!");    
    if results.len() == 0 {
        return format!("Entries not found for authors [{}] and keywords [{}]",
            author.join(", "), keywords.join(", "));
    } else {
        return format!("{}", results.iter().map(|x| x.labeled_to_str(&author)).collect::<Vec<String>>().join("\n"));
    }
}

pub fn open(conn: &SqliteBibDB, id: &str, comment: bool, pdf: bool) {
    let result = conn.get_item(&id).expect(&format!("Cannot find entry with id {}", &id));
    let files = conn.get_files(&result.citation).expect("Find file record in db fail!");
    let mut has_comment = false;
    for (file_name, file_type) in files.iter() {
        if (pdf && (file_type == "pdf")) {
            let pdf_file = File::new(&file_name, &file_type);
            pdf_file.open();
        }
        if (comment && (file_type == "comment")) {
            has_comment = true;
            let comment_file = File::new(&file_name, &file_type);
            comment_file.open();
        }
    }
    if comment && !has_comment {
        let comment_file = File::new(&result.citation, "comment");
        fs::write(&comment_file.path(), result.to_comment());
        conn.add_file(&result.citation, &result.citation, "commment");
        comment_file.open();
    }
}

pub fn delete(conn: &SqliteBibDB, id: &str) {
    if let Ok(results) = conn.get_item(id) {
        conn.delete(id).expect(&format!("Failed to delete existing entry {}!", id));
    } else {
        println!("Cannot find entry with citation = {}", id);
    }
}

pub fn output_str(conn: &SqliteBibDB, source: &str) -> String {
    if PathBuf::from(source).exists() {
        read_pandoc(&source.into())
            .expect(&format!("Failed to read file for citation: {}", source))
            .iter().map(move |x| match conn.get_item(x) {
                Ok(e) => format!("{}", e.to_str()),
                Err(_) => format!("Entry not found for {}!", x),
            }).collect::<Vec<String>>().join("\n")
    } else {
        let output = conn.get_item(source).expect(&format!("Cannot find entry {}", source))
            .to_str();
        format!("{}", output)
    }
}

pub fn output_bib(conn: &SqliteBibDB, source: &str) -> String {
    if PathBuf::from(source).exists() {
        read_pandoc(&source.into())
            .expect(&format!("Failed to read file for citation: {}", source))
            .iter().map(move |x| match conn.get_item(x) {
                Ok(e) => format!("{}", e.to_bib()),
                Err(_) => format!("Entry not found for {}!", x),
            }).collect::<Vec<String>>().join("\n")
    } else {
        let output = conn.get_item(source).expect(&format!("Cannot find entry {}", source))
            .to_bib();
        format!("{}", output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_search() {
        let conn = SqliteBibDB::new(Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/data/library.sqlite")));
        let res = search(&conn, vec!["sur".to_string()], vec!["review".to_string()]);
        assert_eq!(res.split("\n").next(), Some("\u{1b}[38;5;1mMriganka\u{1b}[38;5;4m Sur\u{1b}[39m & John L.R. Rubenstein. (2005) Patterning And Plasticity Of The Cerebral Cortex. Science"));
        let res = search(&conn, vec!["sur".to_string()], Vec::<String>::new());
        assert_eq!(res.matches("\n").count(), 12);
    }
}
