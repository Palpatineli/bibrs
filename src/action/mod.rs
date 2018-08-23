#![allow(unused)]
use std::path::PathBuf;
use std::fs;
use config;
use formatter::BibPrint;
use database::{SqliteBibDB, BibDataBase};
use reader::pandoc::read_pandoc;

pub fn search(author: Vec<String>, keywords: Vec<String>) {
    let conn = SqliteBibDB::new(None);
    let results = conn.search(&author, &keywords);    
    if results.len() == 0 {
        println!("Entries not found for authors [{}] and keywords [{}]",
                 author.join(", "), keywords.join(", "));
    } else {
        println!("{}", results.iter().map(|x| x.labeled_to_str(&author)).collect::<Vec<String>>().join("\n"));
    }
}

pub fn open(id: &str, comment: bool, pdf: bool) {
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
            .to_str();
        println!("{}", output);
    }
}

pub fn keywords(source: &str, add: Vec<String>, del: Vec<String>) {
    let conn = SqliteBibDB::new(None);
    let entry = conn.get_item(source).expect(&format!("Cannot find entry {}", &source));
    println!("{}\n\tKeywords: {}", entry.to_str(), entry.keywords.to_str());
}
