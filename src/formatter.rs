pub mod bibtex;

use termion::color;
use crate::model::{Entry, Person};
use crate::util::ToTitleCase;

pub trait ToString { fn to_str(&self) -> String; }

impl ToString for Person {
    fn to_str(&self) -> String {
        format!("{} {}", self.first_name.to_title(), self.last_name.to_title())
    }
}

impl ToString for Vec<Person> {
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
}

pub trait LabeledPrint { fn labeled_to_str(&self, searched: &[String]) -> String; }

impl Person {
    pub fn labeled_to_str(&self) -> String {
        format!("{}{}{} {}{}", color::Fg(color::Red), self.first_name.to_title(), color::Fg(color::Blue),
            self.last_name.to_title(), color::Fg(color::Reset))
    }
}

impl LabeledPrint for Vec<Person> {
    fn labeled_to_str(&self, searched: &[String]) -> String {
        match self.len() {
            0 => "".to_owned(),
            1 => self[0].to_str(),
            _ => {
                let (head, tail) = self.split_at(self.len() - 1);
                let output = head.iter().map(
                    |x| {
                        if searched.contains(&x.search_term) {
                            x.labeled_to_str()
                        } else { x.to_str() }
                    })
                    .collect::<Vec<String>>().join(", ");
                format!("{} & {}", output,
                        if searched.contains(&tail[0].search_term) {
                            tail[0].labeled_to_str()
                        } else {tail[0].to_str()})
            }
        }
    }
}

impl ToString for Entry {
    fn to_str(&self) -> String {
        let mut output: Vec<String> = Vec::new();
        if !self.authors.is_empty() {
            output.push(self.authors.to_str());
        } else if !self.editors.is_empty() {
            output.push(self.editors.to_str());
        };
        output.push(format!(". ({}).{}. ", self.year, self.title.to_title()));
        if let Some(ref journal) = self.journal { output.push(journal.clone()); }
        else if let Some(ref booktitle) = self.booktitle { output.push(booktitle.clone()); };
        output.concat()
    }
}

impl LabeledPrint for Entry {
    fn labeled_to_str(&self, searched: &[String]) -> String {
        let mut output: Vec<String> = Vec::new();
        if !self.authors.is_empty() {
            output.push(self.authors.labeled_to_str(searched));
        } else if !self.editors.is_empty() {
            output.push(self.editors.labeled_to_str(searched));
        };
        output.push(format!(". ({}) {}. ", self.year, self.title.to_title()));
        if let Some(ref journal) = self.journal { output.push(journal.clone()); }
        else if let Some(ref booktitle) = self.booktitle { output.push(booktitle.clone()); };
        output.concat().trim_str()
    }
}

impl Entry {
    pub fn generate_citation(&self) -> String {
        let last_name: &str = match self.authors.first().or_else(|| self.editors.first()) {
            Some(x) => &x.search_term,
            None => self.title.split_whitespace().next().expect("Error: Nothing in title!")
        };
        format!("{}{}", last_name, self.year)
    }
}

pub trait TrimStr { fn trim_str(&self) -> String; }

impl TrimStr for String { fn trim_str(&self) -> String {
    let length = self.trim_end().len();
    let mut result = self.clone();
    result.truncate(length);
    result
} }

#[cfg(test)]
pub mod tests {
    use super::bibtex::BibPrint;
    use super::*;
    #[test]
    fn test_person() {
        let persons = vec![Person{id: None, search_term: "abcd".to_owned(),
                                  last_name: "âbcd".to_owned(), first_name: "ėfgh".to_owned()},
                           Person{id: None, search_term: "bbcd".to_owned(),
                                  last_name: "bcdê".to_owned(), first_name: "ėfgh".to_owned()},
                           Person{id: None, search_term: "bbc3".to_owned(),
                                  last_name: "b3dê".to_owned(), first_name: "ėfgh".to_owned()}];
        assert_eq!("ėfgh âbcd, ėfgh Bcdê & ėfgh B3dê", persons.to_str());
        assert_eq!("âbcd, ėfgh and Bcdê, ėfgh and B3dê, ėfgh", persons.to_bib());
    }
    use crate::reader::bibtex;
    use std::path::PathBuf;
    #[test]
    fn test_item() {
        let mut test_bib = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_bib.push("test/data/test.bib");
        let item = bibtex::read_entries(&(test_bib));
        println!("title: \n{}", item[0].title.to_title());
        let correct_bib = ["@article{einstein,",
            "\n\tyear = {1905},",
            "\n\tvolume = {322},",
            "\n\tnumber = {10},",
            "\n\tpages = {891-921},",
            "\n\tjournal = {Annalen der Physik},",
            "\n\tauthor = {Einstein, Albert}\n}"].concat();
        let correct_str = [
            "Albert Einstein. (1905).{Zur Elektrodynamik bewegter K{\\\"o}rper}. ({German})\n        ",
            "[{On} The Electrodynamics Of Moving Bodies]. Annalen der Physik"].concat();
        assert_eq!(item[0].to_str(), correct_str);
        assert_eq!(item[0].to_bib(), correct_bib);
    }
    #[test]
    fn test_labeled_item() {
        let mut test_bib = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_bib.push("test/data/test.bib");
        let item = bibtex::read_entries(&test_bib);
        let correct_labeled: &str = "\u{1b}[38;5;1mMichel\u{1b}[38;5;4m Goossens\u{1b}[39m, Frank Mittelbach & \u{1b}\
                                     [38;5;1mAlexander\u{1b}[38;5;4m Samarin\u{1b}[39m. (1993)\
                                     The \\latex\\ Companion.";
        assert_eq!(item[1].labeled_to_str(&["samarin".to_owned(), "goossens".to_owned()]), correct_labeled);
    }
}
