use std::result::Result;
use std::fmt;
use std::convert::From;
use rusqlite::Error;
use super::{SqliteBibDB, BibDataBase, ConfigOrPath};
use crate::model::{Person, Entry};
use crate::formatter::BibPrint;

pub enum CitationError { Citation(InsertionStart, Entry), DBError(Error), } 
pub enum JournalError { Journal(InsertionWithName, String), DBError(Error), }
pub enum PersonError { Person(InsertionWithJournal, Persons), DBError(Error), } 

impl fmt::Display for CitationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CitationError::Citation(_, e) => {write!(f, "naming conflict with {}", e.citation)},
            CitationError::DBError(err) => err.fmt(f)
        }
    }
}

impl fmt::Debug for CitationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CitationError::Citation(_, e) => {write!(f, "citation error with {}. {}, {}", e.citation, file!(), line!())},
            CitationError::DBError(err) => err.fmt(f)
        }
    }
}

impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JournalError::Journal(_, s) => {write!(f, "journal [{}] not found", s)},
            JournalError::DBError(err) => err.fmt(f)
        }
    }
}

impl fmt::Debug for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JournalError::Journal(_, e) => {write!(f, "journal error with {}. {}, {}", e, file!(), line!())},
            JournalError::DBError(err) => err.fmt(f)
        }
    }
}

impl fmt::Display for PersonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PersonError::Person(_, s) => {write!(f, "people conflict: {}", s.iter().map(|x| x.0.to_str()).collect::<Vec<String>>().join(", "))},
            PersonError::DBError(err) => err.fmt(f)
        }
    }
}

impl fmt::Debug for PersonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PersonError::Person(_, e) => {write!(f, "people error with {:?}. {}, {}", e, file!(), line!())},
            PersonError::DBError(err) => err.fmt(f)
        }
    }
}
impl From<Error> for CitationError { fn from(error: Error) -> Self { CitationError::DBError(error) } }
impl From<Error> for PersonError { fn from(error: Error) -> Self { PersonError::DBError(error) } }
impl From<Error> for JournalError { fn from(error: Error) -> Self { JournalError::DBError(error) } }

pub type Persons = Vec<(Person, Vec<Person>)>;

pub struct InsertionStart { pub entry: Entry, conn: SqliteBibDB, }

pub struct InsertionWithName { pub entry: Entry, pub conn: SqliteBibDB, update: bool}

pub struct InsertionWithJournal { pub entry: Entry, conn: SqliteBibDB, journal_id: Option<i32>, update: bool }

pub struct InsertionWithPeople { pub entry: Entry, conn: SqliteBibDB, journal_id: Option<i32>, update: bool }

impl InsertionStart {
    pub fn new(entry: Entry, inputs: Option<ConfigOrPath>) -> Self { Self{entry, conn: SqliteBibDB::new(inputs)} }
    pub fn check_citation(self) -> Result<InsertionWithName, CitationError> {
        match self.conn.get_item(&self.entry.citation) {
            Ok(entry) => Err(CitationError::Citation(self, entry)),
            Err(Error::QueryReturnedNoRows) => Ok(InsertionWithName{entry: self.entry, conn: self.conn, update: false}),
            Err(x) => Err(CitationError::DBError(x))
        }
    }
    pub fn update(self) -> InsertionWithName {
        InsertionWithName{conn: self.conn, update: true, entry: self.entry}
    }
}

impl InsertionWithName {
    pub fn check_journal(self) -> Result<InsertionWithJournal, JournalError> {
        let journal_id: Option<i32> = match self.entry.journal {
            Some(ref journal) => {
                match self.conn.query_journal(journal) {
                    Ok(x) => Some(x),
                    Err(_) => {
                        let journal_str = journal.to_owned();
                        return Err(JournalError::Journal(self, journal_str))
                    }
                }
            },
            None => None
        };
        Ok(InsertionWithJournal{entry: self.entry, conn: self.conn, journal_id, update: self.update})
    }
}

impl InsertionWithJournal {
    pub fn check_people(self) -> Result<InsertionWithPeople, PersonError> {
        let mut conflict_list: Vec<(Person, Vec<Person>)> = Vec::new();
        for input_person in self.entry.authors.clone().into_iter().chain(self.entry.editors.clone().into_iter()) {
            let mut found: bool = false;
            let exist_people = self.conn.search_lastname(&input_person.search_term)?;
            for exist_person in exist_people.iter() {
                if exist_person.first_name == input_person.first_name { found = true; break; }
            }
            if !found { conflict_list.push((input_person, exist_people)); }
        }
        if conflict_list.len() == 0 { Ok(InsertionWithPeople{entry: self.entry, conn: self.conn, journal_id: self.journal_id,
            update: self.update})
        } else { Err(PersonError::Person(self, conflict_list)) }
    }
}

impl InsertionWithPeople {
    pub fn insert(&self) -> Result<(), Error> {
        if self.update { self.conn.delete(&self.entry.citation)?; }
        self.conn.add_item(&self.entry, self.journal_id)?;
        Ok(())
    }
}
