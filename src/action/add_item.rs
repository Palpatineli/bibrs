use std::io;
use machine::{machine, transitions};
use termion::color;
use crate::reader::bibtex::read_entries;
use crate::file::{File, BibFile};
use crate::database::{BibDataBase, journal::Journal};
use crate::ui::add_item::{InsertionStart, InsertionWithName, InsertionWithJournal, InsertionWithPeople};
use crate::ui::add_item::{JournalError, CitationError, PersonError};
use crate::config::CONFIG;
use crate::ui::{self, UI, UIResponse, MsgType, JournalInputs};

// machine!(
//     #[derive(Clone, Debug, PartialEq)]
//     enum EntryState {
//         StageStart,
//         StageCitation,
//         StageJournal,
//         StagePeople,
//         StageEnd,
//         StageFile,
//         StageInsert,
//     }
// );
//
// #[derive(Clone, Debug, PartialEq)]
// pub struct Command { cmd: String, }
//
// transitions!(EntryState,
//     [
//         (StageStart, Command) => [StageCitation, StageEnd],
//         (StageCitation, Command) => [StageJournal, StageEnd],
//         (StageJournal, Command) => [StagePeople, StageEnd],
//         (StagePeople, Command) => [StageFile, StageEnd],
//         (StageFile, Command) => [StageFile, StageEnd, StageInsert]
//     ]
// );

pub fn add_item(keywords: Vec<String>) {
    let bib_file = File::temp("temp_bib").expect(&format!("Cannot find bibtex file in {:?}", CONFIG.temp_bib.folder));
    let pdf_file: Option<File> = File::temp("temp_pdf").ok();
    let pdf = File::temp("temp_pdf").ok();
    let mut entries = read_entries(&bib_file.path());
    let mut entry = entries.pop().expect("empty bibtext file in download folder, or error in the bibtex file");
    entry.citation = entry.to_citation();
    entry.keywords.extend(keywords.into_iter());
    let mut insert = InsertionStart::new(entry, None);
    let ui: UI<ui::SimpleInputs> = UI::new(MsgType::Info);
    match ui.prompt(format!("New Item: {}", insert.entry.to_str())) {
        Ok(ui::SimpleInputs::Continue) => {},
        Ok(ui::SimpleInputs::Abort) | Err(_) => { println!("Aborted."); return }
    };
    let mut insert = 'citation_check: loop {
        match insert.check_citation() {
            Ok(insert) => break 'citation_check insert,
            Err(CitationError::Citation(insert_old, entry)) => {
                insert = insert_old;
                let citation_check_ui: UI<ui::CitationInputs> = UI::new(MsgType::Conflict);
                match citation_check_ui.prompt(format!("naming conflict: {}\nvs. existing entry: {}", insert.entry.citation, entry.to_str())) {
                    Ok(ui::CitationInputs::Changed(x)) => { insert.entry.citation = x; }
                    Ok(ui::CitationInputs::Update) => { insert.entry.update(&entry); break 'citation_check insert.update() },
                    Ok(ui::CitationInputs::Abort) | Err(_) => { println!("Aborted."); return },
                }
            },
            Err(_) => panic!("Database Error on item insertion!")
        }
    };
    let mut insert = 'journal_check: loop {
        match insert.check_journal() {
            Ok(insert) => break 'journal_check insert,
            Err(JournalError::Journal(insert_old, journal_name)) => {
                insert = insert_old;
                let journal_check_ui: UI<ui::JournalInputs> = UI::new(MsgType::Missing);
                match journal_check_ui.prompt(format!("unknown journal! {}", insert.entry.journal.clone().unwrap())) {
                    Ok(ui::JournalInputs::Update((x, y, z))) => {
                        let journal_id = insert.conn.add_journal(Journal{id: None, name: x.clone(), abbr: y, abbr_no_dot: z});
                        if let Ok(id) = journal_id {
                            insert.entry.journal = Some(x);
                            break 'journal_check insert.check_journal().expect("inserted journal not found!");
                        }
                    }
                    Ok(ui::JournalInputs::Abort) | Err(_) => {println!("Aborted."); return },
                }
            },
            Err(_) => panic!("Database Error in journal query!")
        }
    };
    let mut insert = 'person_check: loop {
        match insert.check_people() {
            Ok(insert) => break 'person_check insert,
            Err(PersonError::Person(old_insert, persons)) => {
                insert = old_insert;
            },
            Err(_) => panic!("Database Error in author query!")
        }
    };
    insert.insert().unwrap();
}
