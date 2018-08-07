#![feature(drain_filter)]
extern crate rusqlite;
extern crate termion;
#[macro_use] extern crate structopt;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;

mod action;
mod database;
mod reader;
mod formatter;

mod config;
mod util;
mod model;
mod entry_type;

use structopt::StructOpt;

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

fn main() {
    let opt = Bibrs::from_args();
    match opt {
        Bibrs::Search{authors, keywords} =>
            action::search(authors.join(" ").split(",").map(String::from).collect(),
                           keywords.join(" ").split(",").map(String::from).collect()),
        Bibrs::Open{id, comment, pdf} => action::open(id, comment, pdf),
        Bibrs::Add{keywords} => action::add_paper(keywords),
        Bibrs::Delete{id} => action::delete(id),
        Bibrs::Output{source, bibtex, simple} => {
            if bibtex ^ simple {
                action::output(source, if bibtex {"bib"} else {"str"});
            } else {
                println!("please select one output format!");
            }
        },
        Bibrs::Keywords{source, add, del} =>
            action::keywords(source, add.join(" ").split(",").map(String::from).collect(),
                             del.join(" ").split(",").map(String::from).collect()),
        Bibrs::Init => action::initialize()
    }
}

#[cfg(test)]
mod tests {
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
            _ => assert!(false)
        };
        let opt = Bibrs::from_iter(vec!["bibrs", "s", "-a", "casagrande", "rosa"]);
        match opt {
            Bibrs::Search{authors, keywords} => {
                assert_eq!(authors, vec!["casagrande", "rosa"]);
                assert_eq!(keywords, Vec::<&str>::new());
            },
            _ => assert!(false),
        }
    }
}
