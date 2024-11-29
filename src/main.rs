#![feature(trait_alias)]
#[doc=include_str!("../README.md")]
use std::iter::FromIterator;
use structopt::StructOpt;
use crate::formatter::ToString;

mod action;
mod config;
mod database;
mod entry_type;
mod file;
mod formatter;
mod model;
mod reader;
mod util;

#[derive(StructOpt, Debug, PartialEq)]
#[structopt(name = "bibrs")]
enum Bibrs {
    #[structopt(name = "s", about = "search")]
    Search {
        #[structopt(short = "a", long = "author")]
        authors: Vec<String>,
        #[structopt(short = "k", long = "keyword")]
        keywords: Vec<String>,
    },
    #[structopt(name = "o", about = "open pdf files and comments")]
    Open {
        #[structopt()]
        id: String,
        #[structopt(short = "c", long = "comment")]
        comment: bool,
        #[structopt(short = "p", long = "pdf")]
        pdf: bool,
    },
    #[structopt(name = "a", about = "add entry")]
    Add {
        #[structopt()]
        keywords: Vec<String>,
    },
    #[structopt(name = "d", about = "delete entry")]
    Delete {
        #[structopt()]
        id: String,
    },
    #[structopt(name = "u", about = "output info")]
    Output {
        #[structopt()]
        source: String,
        #[structopt(short = "b", long = "bibtex")]
        bibtex: bool,
        #[structopt(short = "s", long = "string")]
        simple: bool,
    },
    #[structopt(name = "k", about = "add or delete keywords")]
    Keywords {
        #[structopt()]
        source: String,
        #[structopt(short = "a", long = "add")]
        add: Vec<String>,
        #[structopt(short = "d", long = "del")]
        del: Vec<String>,
    },
    #[structopt(name = "init", about = "initialize folders and datebase")]
    Init,
}

fn comma_separate_args<T: FromIterator<String>>(input: Vec<String>) -> T {
    input.join(" ").split(',').map(|x| x.trim().to_string().to_lowercase()).filter(|x| !x.is_empty()).collect()
}

fn main() {
    let opt = Bibrs::from_args();
    if let Bibrs::Init = opt {
        config::initialize();
        return
    }
    let conn = database::SqliteBibDB::new(None);
    match opt {
        Bibrs::Search{authors, keywords} =>
            println!("{}", action::search(&conn, comma_separate_args(authors), comma_separate_args(keywords))),
        Bibrs::Open{id, comment, pdf} => action::open(&conn, &id, comment, pdf),
        Bibrs::Add{keywords} => action::add_item(comma_separate_args(keywords)),
        Bibrs::Delete{id} => action::delete(&conn, &id),
        Bibrs::Output{source, bibtex, simple} => {
            if bibtex { println!("{}", action::output_bib(&conn, &source)); }
            if simple || !bibtex { println!("{}", action::output_str(&conn, &source)); }
        },
        Bibrs::Keywords{source, add, del} => {
            let (entry, keywords) = action::keywords(&conn, &source, comma_separate_args(add),
                                                     comma_separate_args(del));
            print!("{}\n\t{}", entry.to_str(), keywords);
        },
        Bibrs::Init => (),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;
    #[test]
    fn test_opt() {
        let opt = Bibrs::from_iter(vec!["bibrs", "init"]);
        assert_eq!(opt, Bibrs::Init);
        let opt = Bibrs::from_iter(vec!["bibrs", "k", "li2013", "-a", "bullshit", "weird", "-d", "master"]);
        match opt {
            Bibrs::Keywords{source, add, del} => {
                assert_eq!(source, "li2013");
                assert_eq!(add, vec!["bullshit", "weird"]);
                assert_eq!(del, vec!["master"]);
            },
            _ => panic!("Keywords not matched")
        };
        let opt = Bibrs::from_iter(vec!["bibrs", "s", "-a", "casagrande", "rosa"]);
        match opt {
            Bibrs::Search{authors, keywords} => {
                assert_eq!(authors, vec!["casagrande", "rosa"]);
                assert_eq!(keywords, Vec::<&str>::new());
            },
            _ => panic!("authors not matched"),
        }
    }

    #[test]
    fn test_util() {
        let res: HashSet<String> = comma_separate_args(["this", "good,that", "bad,this", "good"].iter().map(
                |x| (*x).to_owned()).collect());
        assert_eq!(res, ["this good", "that bad"].iter().map(|x| (*x).to_owned()).collect::<HashSet<String>>());
        let res2: Vec<String> = comma_separate_args(["this", "good,that", "bad,this", "good"].iter().map(
                |x| (*x).to_owned()).collect());
        assert_eq!(res2, ["this good", "that bad", "this good"].iter().map(|x| (*x).to_owned())
            .collect::<Vec<String>>());
        let res3: Vec<String> = comma_separate_args(vec![]);
        assert_eq!(res3.len(), 0);
        let res4: HashSet<String> = comma_separate_args(vec![]);
        assert_eq!(res4.len(), 0);
    }
}
