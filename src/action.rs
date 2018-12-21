#![allow(unused)]
use std::path::PathBuf;
use std::fs;
use termion::{color, style};

use crate::formatter::{BibPrint, LabeledPrint};
use crate::model::Entry;
use crate::database::{SqliteBibDB, BibDataBase};
use crate::reader::pandoc::read_pandoc;
use crate::config;
use crate::util::BibFileExt;

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
    let conn = SqliteBibDB::new(None);
    let result = conn.get_item(&id).expect(&format!("Cannot find entry with id {}", &id));
    let files = conn.get_files(&id);
    for (file_type, file_name) in files.iter() {
        if (comment && (file_type == "comment")) || (pdf && (file_type = "pdf")) { }
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

/// add or delete keywords
/// print the resulting entry, with new keywords in red and deleted keywords in invert color
pub fn keywords(citation: &str, mut add: Vec<String>, mut del: Vec<String>) {
    let conn = SqliteBibDB::new(None);
    let old_entry = conn.get_item(citation).expect(&format!("Cannot find entry {}", &citation));
    if add.len() > 0 {  // non-exsitng keywords to add
        add = add.into_iter().filter(
            |x| !old_entry.keywords.contains(x)
        ).collect::<Vec<String>>();
    }
    let has_new = (add.len() > 0);
    if del.len() > 0 {  // existing keywords to delete
        del = del.into_iter().filter(
            |x| old_entry.keywords.contains(x)
        ).collect::<Vec<String>>();
    }
    let has_del = (del.len() > 0);
    if !(has_new || has_del) {
        println!("{}\n\tKeywords: {}", old_entry.to_str(), old_entry.keywords.join(", "));
        return
    }
    if has_new {conn.add_keywords(&old_entry.citation, &add);}
    if has_del {conn.del_keywords(&old_entry.citation, &del);}
    let new_entry = conn.get_item(citation).unwrap();
    let (new_terms, retained_terms): (Vec<String>, Vec<String>) = new_entry.keywords.clone().into_iter()
        .partition(|x| old_entry.keywords.contains(&x));
    let deleted_terms = old_entry.keywords.into_iter().filter(|x| !new_entry.keywords.contains(x)).collect::<Vec<String>>();
    let mut keywords: Vec<(String, String)> = new_terms.into_iter().map(
        move |x| (x.clone(), format!("{}{}{}", color::Fg(color::Red), x, color::Fg(color::Reset)))
    ).collect();
    keywords.extend(deleted_terms.into_iter().map(
        move |x| (x.clone(), format!("{}{}{}{}{}", color::Fg(color::Blue), style::Invert, x, style::Reset, color::Fg(color::Reset)))
    ));
    keywords.extend(retained_terms.into_iter().map(move |x| (x.clone(), x)));
    keywords.sort();
    let keywords_str = keywords.into_iter().map(|x| x.1).collect::<Vec<String>>().join(", ");
    println!("{}\n\tKeywords: {}", new_entry.to_str(), keywords_str);
}
