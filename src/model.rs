use std::collections::{HashSet, HashMap};
use std::iter::Extend;
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
    pub extra_fields: HashMap<String, String>,
    pub files: Vec<(String, String)>,
}

fn update_option<T>(this: &mut Option<T>, that: &Option<T>) where T: Clone {
    if this.is_none() && that.is_some() { this.replace(that.as_ref().unwrap().clone()); }
}

impl Entry {
    /// Unlike what the operation suggests, it updates the other way: add stuff to self only if self 
    /// does not have it.
    pub fn update(&mut self, other: &Entry) {
        update_option(&mut self.booktitle, &other.booktitle);
        update_option(&mut self.month, &other.month);
        update_option(&mut self.chapter, &other.chapter);
        update_option(&mut self.edition, &other.edition);
        update_option(&mut self.volume, &other.volume);
        update_option(&mut self.number, &other.number);
        update_option(&mut self.pages, &other.pages);
        update_option(&mut self.journal, &other.journal);
        if self.authors.len() == 0 {self.authors.extend(other.authors.iter().map(|x| x.clone()))}
        if self.editors.len() == 0 {self.editors.extend(other.editors.iter().map(|x| x.clone()))}
        let to_add: Vec<(String, String)> = other.extra_fields.iter().filter_map(
            |(k, v)| if self.extra_fields.contains_key(k) { None } else { Some((k.clone(), v.clone()))})
            .collect();
        self.extra_fields.extend(to_add);
    }
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
