use std::collections::HashSet;

use itertools::Itertools;

use crate::model::{Entry, Person};
use crate::util::ToTitleCase;

pub trait BibPrint {
    fn to_bib(&self) -> String;
}

impl BibPrint for i32 { fn to_bib(&self) -> String { self.to_string() } }

impl BibPrint for String { fn to_bib(&self) -> String { self.clone() } }

impl BibPrint for Person {
    fn to_bib(&self) -> String {
        format!("{}, {}", self.last_name.to_title(), self.first_name.to_title())
    }
}

impl BibPrint for Vec<Person> {
    fn to_bib(&self) -> String {
        self.iter().map(|x| x.to_bib()).collect::<Vec<String>>().join(" and ")
    }
}

impl BibPrint for HashSet<String> {
    fn to_bib(&self) -> String {
        self.iter().sorted().join(", ")
    }
}

macro_rules! insert_field {
    ($target:ident, $struct:ident, $($field_name:ident),+) => (
        $(if let Some(ref temp) =  $struct.$field_name {
            $target.push((stringify!($field_name), temp.to_bib()))
        })+
    )
}

macro_rules! insert_vec {
    ($target:ident, $struct:ident, $({$field:ident, $field_name:ident}),+) => (
        $(if $struct.$field.len() > 0 {
            $target.push((stringify!($field_name), $struct.$field.to_bib()))
        })+
    )
}

impl BibPrint for Entry {
    fn to_bib(&self) -> String {
        let mut output: Vec<String> = Vec::new();
        output.push(format!{"@{}{{{}", self.entry_type, self.citation});

        let mut fields: Vec<(&str, String)> = Vec::new();
        fields.push(("year", self.year.to_bib()));
        insert_field!(fields, self, booktitle, chapter, edition, month, volume, number, pages, journal);
        insert_vec!(fields, self, {editors, editor}, {authors, author}, {keywords, keyword});
        for (field, value) in self.extra_fields.iter() {fields.push((field, value.clone()))};

        for (field, value) in fields.into_iter() { output.push(format!{",\n\t{} = {{{}}}", field, value})};
        output.push("\n}".to_owned());
        output.concat().to_string()
    }
}
