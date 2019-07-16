use std::collections::HashSet;
use lazy_static::lazy_static;
use crate::entry_type::EntryType;

#[derive(Default, Debug, Clone)]
pub struct Person {
    pub id: Option<i32>,
    pub last_name: String,
    pub first_name: String,
    pub search_term: String,
}

#[derive(Default, Debug, Clone)]
pub struct Entry {
    pub citation: String,
    pub entry_type: EntryType,
    pub title: String,
    pub booktitle: Option<String>, 
    pub year: i32,
    pub month: Option<i32>,
    pub chapter: Option<i32>,
    pub edition: Option<i32>,
    pub volume: Option<i32>,
    pub number: Option<i32>,
    pub pages: Option<String>,
    pub journal: Option<String>,
    pub authors: Vec<Person>,
    pub editors: Vec<Person>,
    pub keywords: HashSet<String>,
    pub extra_fields: Vec<(String, String)>,
    pub files: Vec<(String, String)>,
}

macro_rules! str_hashset {
    ($($item:expr),*) => {{
        let mut temp_set = HashSet::new();
        $(temp_set.insert($item.to_owned());)*
        temp_set
    }};
}

lazy_static! {
    pub static ref EXTRA_FIELDS: HashSet<String> = str_hashset!{
        "howpublished", "institution", "organization", "address", "note", "publisher",
        "school", "series", "doi", "eprint"};
}
