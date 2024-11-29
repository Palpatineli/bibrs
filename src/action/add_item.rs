use std::error::Error;

use inquire::{Confirm, Text, Select};
use rusqlite::Error::QueryReturnedNoRows;
use crate::reader::bibtex::read_entries;
use crate::file::{File, BibFile};
use crate::database::{BibDataBase, journal::{Journal, JournalDB}};
use crate::formatter::ToString;
use crate::config::CONFIG;

pub fn add_item(conn: BibDataBase, journal_conn: JournalDB, keywords: Vec<String>) -> Result<(), Error> {
    let bib_file = File::temp("temp_bib").unwrap_or_else(
        |_| panic!("Cannot find bibtex file in {:?}", CONFIG.temp_bib.folder));
    let pdf_file: Option<File> = File::temp("temp_pdf").ok();
    let pdf = File::temp("temp_pdf").ok();
    let mut entries = read_entries(bib_file.path());
    let mut entry = entries.pop().expect("empty bibtext file in download folder, or error in the bibtex file");
    entry.citation = entry.generate_citation();
    entry.keywords.extend(keywords);
    println!("New Item: \n{}", entry.to_str());

    // Ask citation
    'citation_check: loop {
        match conn.get_item(&entry.citation) {
            Ok(existing_entry) => {
                println!("Conflicting citation: \n{}", existing_entry.to_str());
                match Text("Input suffix, input nothing to update the existing entry").prompt()? {
                    "" => {
                        entry.update(&existing_entry);
                        break 'citation_check;
                    },
                    new_citation => {
                        entry.citation = entry.citation.push_str(new_citation);
                    }
                }
            },
            Err(_) => break 'citation_check
        }
    }

    // Ask Journal
    if let Some(journal_name) = entry.journal {
        entry.journal = 'journal_check: loop {
            match conn.search_journal(&journal_name) {
                Ok(insert) => break 'journal_check Some(insert),
                Err(QueryReturnedNoRows) => {
                    match journal_conn.search(&journal_name) {
                        Ok(new_journal) => {
                            conn.add_journal(new_journal);
                            break 'journal_check Some(new_journal.name)
                        },
                        Err(QueryReturnedNoRows) => {
                            match Text("Journal not found, please add a new entry:\n\tSeparated by commas: \
                                full name, abbreviation, abbreviation without dots").prompt()? {
                                Ok(result) => {
                                    let new_journal = Journal::from_list(result.split(",")
                                        .map(|x| x.trim()).collect::<Vec<String>>());
                                    entry.journal = new_journal.name.clone();
                                    conn.add_journal(new_journal);
                                }
                            }
                        },
                        Err(err) => return Err(err)
                    }
                },
                Err(err) => return Err(err)
            }
        }
    }

    // Ask Author
    for person in entry.authors.iter() {
        match conn.search_lastname(&person.last_name) {
            Ok(authors) => {
                Select(authors.map(|x| x.first_names)).prompt()
            },
            Err(_) => {}
        }
    }
    'person_check: loop {
        match conn.search_lastname(&entry.authors).check_people() {
            Ok(insert) => break 'person_check insert,
            Err(PersonError::Person(old_insert, _)) => {
                insert = old_insert;
            },
            Err(_) => panic!("Database Error in author query!")
        }
    };
    match Confim::new("(c)ontinue or (a)bort?").prompt() {
        Ok(true) => { insert.insert().unwrap() },
        Ok(false) | Err(_) => { println!("Aborted.") }
    };
    return Ok(())
}
