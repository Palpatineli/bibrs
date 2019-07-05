#![allow(unused)]
use std::path::PathBuf;
use std::fs;

use crate::formatter::{BibPrint, LabeledPrint};
use crate::model::Entry;
use crate::database::{SqliteBibDB, BibDataBase};
use crate::reader::pandoc::read_pandoc;
use crate::config;
use crate::file::{File, BibFile};

mod keywords;
pub use self::keywords::keywords;

pub fn search(mut author: Vec<String>, mut keywords: Vec<String>) {
    let conn = SqliteBibDB::new(None);
    author.retain(|x| x.len() > 0);
    keywords.retain(|x| x.len() > 0);
    if author.len() == 0 && keywords.len() == 0 {
        println!("Search by author last names either/or keywords!");
        return
    }
    let results = conn.search(&author, &keywords);    
    if results.len() == 0 {
        println!("Entries not found for authors [{}] and keywords [{}]",
                 author.join(", "), keywords.join(", "));
    } else {
        println!("{}", results.iter().map(|x| x.labeled_to_str(&author)).collect::<Vec<String>>().join("\n"));
    }
}

pub fn open(id: &str, comment: bool, pdf: bool) {
    let conn = SqliteBibDB::new(None);
    let result = conn.get_item(&id).expect(&format!("Cannot find entry with id {}", &id));
    let files = conn.get_files(&result.citation);
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
        fs::write(comment_file.path(), result.to
    }
}

pub fn add_paper(keywords: Vec<String>) { }

pub fn delete(id: &str) {
    let conn = SqliteBibDB::new(None);
    if let Ok(results) = conn.get_item(id) {
        conn.delete(id).expect(&format!("Failed to delete existing entry {}!", id));
    } else {
        println!("Cannot find entry with citation = {}", id);
    }
}

pub fn output_str(source: &str) {
    let conn = SqliteBibDB::new(None);
    if PathBuf::from(source).exists() {
        read_pandoc(&source.into())
            .expect(&format!("Failed to read file for citation: {}", source))
            .iter().for_each(move |x| match conn.get_item(x) {
                Ok(e) => println!("{}", e.to_str()),
                Err(_) => println!("Entry not found for {}!", x),
            })
    } else {
        let output = conn.get_item(source).expect(&format!("Cannot find entry {}", source))
            .to_str();
        println!("{}", output);
    }
}

pub fn output_bib(source: &str) {
    let conn = SqliteBibDB::new(None);
    if PathBuf::from(source).exists() {
        read_pandoc(&source.into())
            .expect(&format!("Failed to read file for citation: {}", source))
            .iter().for_each(move |x| match conn.get_item(x) {
                Ok(e) => println!("{}", e.to_bib()),
                Err(_) => println!("Entry not found for {}!", x),
            })
    } else {
        let output = conn.get_item(source).expect(&format!("Cannot find entry {}", source))
            .to_bib();
        println!("{}", output);
    }
}
