#![allow(unused)]
use std::path::PathBuf;
use std::io;
use std::fs;

use termion::color;

use crate::formatter::{BibPrint, LabeledPrint};
use crate::model::Entry;
use crate::database::{SqliteBibDB, BibDataBase};
use crate::database::add_item::{InsertionStart, InsertionWithName, InsertionWithJournal, InsertionWithPeople, CitationError};
use crate::reader::{pandoc::read_pandoc, bibtex::read_entries};
use crate::config::{self, CONFIG};
use crate::file::{File, BibFile};

mod keywords;
pub use self::keywords::keywords;

macro_rules! fg {
    ($col:ident, $content:expr) => {
        format!("{}{}{}", color::Fg(color::$col), $content, color::Fg(color::Reset))
    }
}

macro_rules! bg {
    ($col:ident, $content:expr) => {
        format!("{}{}{}", color::Bg(color::$col), $content, color::Bg(color::Reset))
    }
}

pub fn search(mut author: Vec<String>, mut keywords: Vec<String>) {
    let conn = SqliteBibDB::new(None);
    author.retain(|x| x.len() > 0);
    keywords.retain(|x| x.len() > 0);
    if author.len() == 0 && keywords.len() == 0 {
        println!("Search by author last names either/or keywords!");
        return
    }
    let results = conn.search(&author, &keywords).expect("Search Fail!");    
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

pub fn add_item(keywords: Vec<String>) {
    let bib_file = File::temp("temp_bib").expect(&format!("Cannot find bibtex file in {:?}", CONFIG.temp_bib.folder));
    let pdf_file: Option<File> = File::temp("temp_pdf").ok();
    let pdf = File::temp("temp_pdf").ok();
    let mut entries = read_entries(&bib_file.path());
    let mut entry = entries.pop().expect("empty bibtext file in download folder, or error in the bibtex file");
    entry.keywords.extend(keywords.into_iter());
    let mut insert = InsertionStart::new(entry, None);
    // start interactive addtion
    println!("New Item: {}\n", insert.entry.to_str());
    if let Some(file) = pdf_file { println!("\thas file: {}\n", file.path().to_string_lossy()) }
    println!("{}bort, {}ontinue?\n", fg!(Red, "(a)"), fg!(Blue, "(c)")); 
    let stdin = io::stdin();
    let mut input_string = String::new();
    match stdin.read_line(&mut input_string) { Ok(_) => if input_string.starts_with("c") {} else {return}, Err(_) => return };
    // solve naming conflict
    let mut citation = insert.entry.to_citation();
    let insert = 'citation_check: loop {
        match insert.check_citation(&citation) {
            Ok(insert) => break 'citation_check insert,
            Err(CitationError::Citation(insert_old, entry)) => {
                insert = insert_old;
                println!("[{}] naming conflict! {}\nexisting entry: {}\n{}bort; {}pdate entry; input new citation?",
                         fg!(Magenta, "Conflict"), citation, entry.to_str(), fg!(Red, "(a)"), fg!(Blue, "(u)"));
                match stdin.read_line(&mut input_string) {
                    Ok(_) => {
                        match input_string.as_ref() {
                            "a" => return,
                            "u" => {
                            },
                            x => { citation = x.to_owned(); }
                        }
                    },
                    Err(_) => return
                };
            },
            Err(_) => panic!("Database Error on item insertion!")
        }
    };
}

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
