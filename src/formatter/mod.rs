use std::fmt;
use model::{Entry, Person};
use util::ToTitleCase;
use termion::style;

trait BibPrint {
    fn to_bib(&self) -> String;
    fn to_str(&self) -> String;
}

impl BibPrint for i32 {
    fn to_bib(&self) -> String {self.to_string()}
    fn to_str(&self) -> String {self.to_string()}
}

impl BibPrint for String {
    fn to_bib(&self) -> String {self.clone()}
    fn to_str(&self) -> String {self.clone()}
}

impl BibPrint for Person {
    fn to_bib(&self) -> String {format!("{}, {}", self.last_name, self.first_name)}
    fn to_str(&self) -> String {format!("{} {}", self.first_name, self.last_name)}
}

impl BibPrint for Vec<Person> {
    fn to_str(&self) -> String {
        match self.len() {
            0 => "".to_owned(),
            1 => self[0].to_str(),
            _ => {
                let (head, tail) = self.split_at(self.len() - 1);
                let output = head.iter().map(|x| x.to_str()).collect::<Vec<String>>().join(", ");
                format!("{} & {}", output, tail[0].to_str())
            }
        }
    }
    fn to_bib(&self) -> String { self.iter().map(|x| x.to_bib()).collect::<Vec<String>>().join(" and ") }
}

impl BibPrint for Vec<String> {
    fn to_str(&self) -> String { self.join(", ") }
    fn to_bib(&self) -> String { self.join(", ")}
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

impl Entry {
    pub fn to_bib(&self) -> String {
        let mut output: Vec<String> = Vec::new();
        output.push(format!{"@{}{{{}", self.entry_type.to_string(), String::from_utf8_lossy(&self.citation)});

        let mut fields: Vec<(&str, String)> = Vec::new();
        fields.push(("year", self.year.to_bib()));
        insert_field!(fields, self, booktitle, chapter, edition, month, volume, number, pages, journal);
        insert_vec!(fields, self, {editors, editor}, {authors, author}, {keywords, keyword});
        for (field, value) in self.extra_fields.iter() {fields.push((field, value.clone()))};

        for (field, value) in fields.into_iter() { output.push(format!{",\n\t{} = {{{}}}", field, value})};
        output.push("\n}".to_owned());
        output.concat().to_string()
    }
    pub fn to_str(&self) -> String {
        let mut output: Vec<String> = Vec::new();
        if self.authors.len() > 0 {
            output.push(self.authors.to_str());
        } else if self.editors.len() > 0 {
            output.push(self.editors.to_str());
        };
        output.push(". ".to_owned());
        output.push(format!("({}).", self.year));
        output.push(self.title.to_title());
        output.push(". ".to_owned());
        if let Some(ref journal) = self.journal { output.push(journal.clone()); }
        else if let Some(ref booktitle) = self.booktitle { output.push(booktitle.clone()); };
        output.concat()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn test_display() {
        let persons = vec![Person{id: None, search_term: "abcd".to_owned().into_bytes(),
                                  last_name: "âbcd".to_owned(), first_name: "ėfgh".to_owned()},
                           Person{id: None, search_term: "bbcd".to_owned().into_bytes(),
                                  last_name: "bcdê".to_owned(), first_name: "ėfgh".to_owned()},
                           Person{id: None, search_term: "bbc3".to_owned().into_bytes(),
                                  last_name: "b3dê".to_owned(), first_name: "ėfgh".to_owned()}];
        print!("{}", persons.to_str());
        assert_eq!("ėfgh âbcd, ėfgh bcdê & ėfgh b3dê", persons.to_str());
        print!("{}", persons.to_bib());
        assert_eq!("âbcd, ėfgh and bcdê, ėfgh and b3dê, ėfgh", persons.to_bib());
    }
}
