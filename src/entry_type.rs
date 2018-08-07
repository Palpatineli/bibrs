use std::fmt;

#[derive(Debug, PartialEq)]
pub enum EntryType {
    Article,
    Book,
    Booklet,
    Inbook,
    Incollection,
    Inproceedings,
    Manual,
    Masterthesis,
    Misc,
    Phdthesis,
    Proceedings,
    Techreport,
    Unpublished,
}

impl EntryType {
    pub fn parse(input: &str) -> EntryType {
        match input {
            "article" => EntryType::Article,
            "book" => EntryType::Book,
            "booklet" => EntryType::Booklet,
            "inbook" => EntryType::Inbook,
            "incollection" => EntryType::Incollection,
            "inproceedings" => EntryType::Inproceedings,
            "manual" => EntryType::Manual,
            "masterthesis" => EntryType::Masterthesis,
            "misc" => EntryType::Misc,
            "phdthesis" => EntryType::Phdthesis,
            "proceedings" => EntryType::Proceedings,
            "techreport" => EntryType::Techreport,
            "unpublished" => EntryType::Unpublished,
            _ => EntryType::Misc,
        }
    }
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            EntryType::Article => "article",
            EntryType::Book => "book",
            EntryType::Booklet => "booklet",
            EntryType::Inbook => "inbook",
            EntryType::Incollection => "incollection",
            EntryType::Inproceedings => "inproceedings",
            EntryType::Manual => "manual",
            EntryType::Masterthesis => "masterthesis",
            EntryType::Misc => "misc",
            EntryType::Phdthesis => "phdthesis",
            EntryType::Proceedings => "proceedings",
            EntryType::Techreport => "techreport",
            EntryType::Unpublished => "unpublished",
        };
        write!(f, "{}", printable)
    }
}

impl Default for EntryType {
    fn default() -> EntryType { EntryType::Article }
}
