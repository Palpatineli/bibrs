use core::panic;
use std::path::PathBuf;
use std::fs;

use crate::formatter::{ToString, LabeledPrint, bibtex::BibPrint};
use crate::database::{SqliteBibDB, BibDataBase};
use crate::reader::pandoc::read_pandoc;
use crate::file::{File, BibFile};

mod keywords;
mod add_item;
pub use add_item::add_item;
pub use self::keywords::keywords;

pub fn search(conn: &SqliteBibDB, mut author: Vec<String>, mut keywords: Vec<String>) -> String {
    author.retain(|x| !x.is_empty());
    keywords.retain(|x| !x.is_empty());
    if author.is_empty() && keywords.is_empty() {
        return "Search by author last names either/or keywords!".to_string();
    }
    let results = conn.search(&author, &keywords).expect("Search Fail!");    
    if results.is_empty() {
        format!("Entries not found for authors [{}] and keywords [{}]", author.join(", "), keywords.join(", "))
    } else {
        results.iter().map(|x| x.labeled_to_str(&author)).collect::<Vec<String>>().join("\n")
    }
}

pub fn open(conn: &SqliteBibDB, id: &str, comment: bool, pdf: bool) {
    let result = conn.get_item(id).unwrap_or_else(|_| panic!("Cannot find entry with id {}", &id));
    let files = conn.get_files(&result.citation).expect("Find file record in db fail!");
    let mut has_comment = false;
    for (file_name, file_type) in files.iter() {
        if pdf && (file_type == "pdf") {
            let pdf_file = File::new(file_name, file_type);
            pdf_file.open().unwrap();
        }
        if comment && (file_type == "comment") {
            has_comment = true;
            let comment_file = File::new(file_name, file_type);
            comment_file.open().unwrap();
        }
    }
    if comment && !has_comment {
        let comment_file = File::new(&result.citation, "comment");
        fs::write(comment_file.path(), result.to_comment()).unwrap();
        conn.add_file(&result.citation, &result.citation, "commment").unwrap();
        comment_file.open().unwrap();
    }
}

pub fn delete(conn: &SqliteBibDB, id: &str) {
    if let Ok(_) = conn.get_item(id) {
        let files = conn.get_files(id).unwrap();
        conn.delete(id).unwrap_or_else(|_| panic!("Failed to delete existing entry {}!", id));
        for (ref file_name, ref file_type) in files {
            File::new(file_name, file_type).remove()
                .unwrap_or_else(|_| panic!("Failed to remove file with name = '{}' and type = '{}'",
                        file_name, file_type));
        }
    } else {
        println!("Cannot find entry with citation = {}", id);
    }
}

pub fn output_str(conn: &SqliteBibDB, source: &str) -> String {
    if PathBuf::from(source).exists() {
        read_pandoc(&source.into())
            .unwrap_or_else(|_| panic!("Failed to read file for citation: {}", source))
            .iter().map(move |x| match conn.get_item(x) {
                Ok(e) => e.to_str(),
                Err(_) => format!("Entry not found for {}!", x),
            }).collect::<Vec<String>>().join("\n")
    } else {
        conn.get_item(source).unwrap_or_else(|_| panic!("Cannot find entry {}", source)).to_str()
    }
}

pub fn output_bib(conn: &SqliteBibDB, source: &str) -> String {
    if PathBuf::from(source).exists() {
        read_pandoc(&source.into())
            .unwrap_or_else(|_| panic!("Failed to read file for citation: {}", source))
            .iter().map(move |x| match conn.get_item(x) {
                Ok(e) => e.to_bib(),
                Err(_) => format!("Entry not found for {}!", x),
            }).collect::<Vec<String>>().join("\n")
    } else {
        conn.get_item(source).unwrap_or_else(|_| panic!("Cannot find entry {}", source)).to_bib()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search() {
        let conn = SqliteBibDB::new(Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/data/library.sqlite")));
        let res = search(&conn, vec!["sur".to_string()], vec!["review".to_string()]);
        assert_eq!(res.split('\n').next(), Some("\u{1b}[38;5;1mMriganka\u{1b}[38;5;4m Sur\u{1b}[39m & John L.R. \
                Rubenstein. (2005) Patterning And Plasticity Of The Cerebral Cortex. Science"));
        let res = search(&conn, vec!["sur".to_string()], Vec::<String>::new());
        assert_eq!(res.matches('\n').count(), 12);
        let res = search(&conn, Vec::<String>::new(), vec!["review".to_string()]);
        assert_eq!(res.matches('\n').count(), 76);
    }

    #[test]
    fn test_output_bib() {
        let conn = SqliteBibDB::new(Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/data/library.sqlite")));
        let res = output_bib(&conn, "casagrande1994");
        assert_eq!(res.split('\n').map(|x| x.trim()).collect::<Vec<&str>>().join(""),
        "@article{casagrande1994,\
        year = {1994},\
        volume = {10},\
        number = {8},\
        pages = {201-259},\
        journal = {Cerebral Cortex},\
        author = {Casagrande, Vivien A.}\
        }");
    }

    #[test]
    fn test_output_from_text() {
        let conn = SqliteBibDB::new(Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/data/library.sqlite")));
        let test_text = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/data/extract_test.txt");
        let _bib_res = output_bib(&conn, test_text.to_str().unwrap());
        let _str_res = output_str(&conn, test_text.to_str().unwrap());
    }

    #[test]
    fn test_output_str() {
        let conn = SqliteBibDB::new(Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/data/library.sqlite")));
        let res = output_str(&conn, "casagrande1994");
        assert_eq!(res, "Vivien A. Casagrande. (1994).The Afferent, Intrinsic, And Efferent Connections Of Primary Visual Cortex In Primates. Cerebral Cortex");
    }
}
