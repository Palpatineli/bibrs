#![allow(unused)]
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

use nom_bibtex::Bibtex;
use nom_bibtex::model::Bibliography;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use lazy_static::lazy_static;

use crate::model::{Entry, Person};
use crate::entry_type::EntryType;

fn strip_accent(input: &str) -> String {
    input.nfd().filter(|x| x.is_ascii_alphanumeric()).collect::<String>()
}

impl Person {
    pub fn load(input: &str) -> Self {
        if input.contains(", ") {
            let mut substr_iter = input.split(", ");
            let last_name = substr_iter.next().unwrap().to_owned().to_lowercase();
            let mut first_name = substr_iter.next().unwrap().to_owned().to_lowercase();
            let search_term = strip_accent(&last_name);
            Person { id: None, last_name, first_name, search_term }
        } else {
            let mut substr_iter = input.rsplitn(2, " ");  // ! return word order is reversed too
            let mut last_name = substr_iter.next().unwrap().to_owned().to_lowercase();
            let mut first_name = substr_iter.next().unwrap().to_owned().to_lowercase();
            let search_term = strip_accent(&last_name);
            Person { id: None, last_name, first_name, search_term }
        }
    }
}

fn load_people(input: &str) -> Vec<Person> {
    let mut result = Vec::new();
    for person_str in input.split(" and ") { result.push(Person::load(person_str)); }
    result
}

fn load_title(input: &str) -> String {
    lazy_static!{ static ref ITALIC_RE: Regex = Regex::new(r#"<\s*i\s*>([\w\s]+)<\s*/i>"#).unwrap(); }
    ITALIC_RE.replace(input, r#"\\textit{\1}"#).to_string()
}

fn load_pages(input: &str) -> String {
    /// force pages formatting 123-126, 123-6, 123:126, 123--126, 123_126 to 123-126
    lazy_static!{ static ref PAGE_RE: Regex = Regex::new(r#"^(\w*)(\d+)[-:_]{1,2}(\d+)$"#).unwrap();}
    // don't change pages in the form xx.xxx/xx.xxx
    if input.contains('.') || input.contains('/') {
        input.to_owned()
    } else {
        match PAGE_RE.captures(input) {
            Some(caps) => {  // some reference would have stuff like 12345-51 to mean 12345-12351
                let start = caps.get(2).unwrap().as_str().to_owned();
                let mut end = caps.get(3).unwrap().as_str().to_owned();
                if start.len() > end.len() { end = start[0..start.len() - end.len()].to_owned() + &end; }
                format!{"{}{}-{}", caps.get(1).unwrap().as_str(), start.parse::<u32>().unwrap(), end.parse::<u32>().unwrap()}
            },
            None => input.to_owned()
        }
    }
}

fn load_keywords(input: &str) -> Vec<String> {
    input.split(", ").map(|s| s.to_owned()).collect::<Vec<String>>()
}

fn read_file(filename: &Path) -> String {
    let mut content = String::new();
    let mut file = File::open(filename).unwrap().read_to_string(&mut content).unwrap();
    content
}

pub fn read_entries(filename: &Path) -> Vec<Entry> {
    let mut results: Vec<Entry> = Vec::new();
    let file_content = read_file(filename);
    let bibtex = Bibtex::parse(&file_content).unwrap();
    for bib_entry in bibtex.bibliographies().into_iter() {
        results.push(Entry::from_bib(bib_entry))
    }
    results
}

impl Entry {
    pub fn from_bib(bib_entry: &Bibliography) -> Self {
        let citation = strip_accent(bib_entry.citation_key());
        let entry_type = EntryType::parse(bib_entry.entry_type());
        let mut entry = Entry{citation: citation, entry_type: entry_type, ..Default::default()};
        for (field_name, content) in bib_entry.tags().into_iter() {
            match field_name.as_ref() {
                "title" => entry.title = load_title(content),
                "booktitle" => entry.booktitle = Some(load_title(content)),
                "pages" => entry.pages = Some(load_pages(content)),
                "author" => entry.authors = load_people(content),
                "editor" => entry.editors = load_people(content),
                "keywords" => entry.keywords = load_keywords(content),
                "year" => entry.year = content.parse::<i32>().unwrap(),
                "chapter" => entry.chapter = Some(content.parse::<i32>().unwrap()),
                "edition" => entry.edition = Some(content.parse::<i32>().unwrap()),
                "month" => entry.month = Some(content.parse::<i32>().unwrap()),
                "number" => entry.number = Some(content.parse::<i32>().unwrap()),
                "volume" => entry.volume = Some(content.parse::<i32>().unwrap()),
                "journal" => entry.journal = Some(content.to_owned()),
                "id" | "publisher" | "school" | "insititution" | "note" | "url" | "series" | "address" | "howpublished" |
                     "organization" => entry.extra_fields.push((field_name.to_owned(), content.to_owned())),
                _ => continue,
            }
        }
        entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bib() {
        let bib_str = read_file(Path::new("test/data/test.bib"));
        let bibtex = Bibtex::parse(&bib_str).unwrap();

        let vars = bibtex.variables();
        assert_eq!(vars[0], ("donald".into(), "Donald Knuth".into()));
        assert_eq!(vars[1], ("mass".into(), "Massachusetts".into()));

        assert_eq!(bibtex.preambles()[0], "Why not a preamble".to_string());

        let b0 = &bibtex.bibliographies()[0];
        assert_eq!(b0.entry_type(), "article");
        assert_eq!(b0.citation_key(), "einstein");
        assert_eq!(b0.tags()[0], ("author".into(), "Albert Einstein".into()));
        assert_eq!(b0.tags()[4], ("number".into(), "10".into()));

        let b1 = &bibtex.bibliographies()[1];
        assert_eq!(b1.citation_key(), "latexcompanion");
        assert_eq!(
            b1.tags()[4],
            ("address".into(), "Reading, Massachusetts".into())
        );

        let b2 = &bibtex.bibliographies()[2];
        assert_eq!(b2.citation_key(), "knuthwebsite");
        assert_eq!(b2.tags()[0], ("author".into(), "Donald Knuth".into()));
    }
    #[test]
    fn test_strip_accent() {
        let a = "βaèâbcd";
        assert_eq!(strip_accent(a), "aeabcd");
        let b = "bcdefg";
        assert_eq!(strip_accent(b), "bcdefg");
    }
    #[test]
    fn test_from_parser() {
        let entries = read_entries(Path::new("test/data/test.bib"));
        assert_eq!(entries[0].citation, "einstein");
        assert_eq!(entries[0].authors[0].last_name, "einstein");
        assert_eq!(entries[0].authors[0].first_name, "albert");
        assert_eq!(entries[0].journal, Some("Annalen der Physik".to_owned()));
        assert_eq!(entries[0].year, 1905);
        assert_eq!(entries[1].extra_fields.iter().filter(|x| x.0 == "address")
            .collect::<Vec<&(String, String)>>()[0].1, "Reading, Massachusetts");
    }
}
