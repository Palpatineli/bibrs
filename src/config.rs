use std::fs::{File, copy, create_dir};
use std::io::{Read, stdin, Error as IOError};
use std::path::PathBuf;

use serde_derive::Deserialize;
use dirs::{home_dir, config_dir};
use rusqlite::Connection;
use lazy_static::lazy_static;

#[derive(Deserialize, Clone)]
pub struct FileHandler {
    pub folder: PathBuf,
    pub extension: Vec<String>,
    pub opener: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub database: PathBuf,
    pub journal_db: PathBuf,
    pub pdf: FileHandler,
    pub comment: FileHandler,
    pub temp_pdf: FileHandler,
    pub temp_bib: FileHandler,
}

lazy_static!{
    pub static ref DEFAULT_CONFIG: PathBuf = config_dir().unwrap().join("bibrs/bibrs.toml");
}

impl Config {
    pub fn new(config_path: Option<PathBuf>) -> Config {
        let mut config_file;
        let config_path = config_path.unwrap_or_else(|| DEFAULT_CONFIG.to_path_buf());
        config_file = File::open(&config_path).expect(
            &format!("Specified config file not found at {}!", config_path.to_string_lossy()));
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str).expect("Failed to read config");
        let mut output: Config = toml::from_str(&config_str).unwrap();
        output.database = home_dir().unwrap().join(&output.database);
        output.journal_db = home_dir().unwrap().join(&output.journal_db);
        output.pdf.folder = home_dir().unwrap().join(&output.pdf.folder);
        output.comment.folder = home_dir().unwrap().join(&output.comment.folder);
        output.temp_pdf.folder = home_dir().unwrap().join(&output.temp_pdf.folder);
        output.temp_bib.folder = home_dir().unwrap().join(&output.temp_pdf.folder);
        return output
    }
}

pub fn initialize() {
    println!("Do you want to initilize the bib database?");
    let mut answer = String::new();
    stdin().read_to_string(&mut answer).expect("Failed to read from stdin!");
    match answer.as_ref() {
        "y" | "1" | "Y" => {
            let config = init_config().unwrap();
            init_folders(&config).unwrap();
            init_database(&config).unwrap();
        },
        _ => { println!("Bib database initialization canceled."); }
    };
}

/// load config from xdg_config, if doesn't exist then copy default config from crate
fn init_config() -> Result<Config, IOError> {
    let config_path = DEFAULT_CONFIG.to_path_buf();
    if !config_path.exists() {
        println!("Moving config file to ~/.config/bibrs.toml");
        copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/bibrs.toml"), &config_path)?;
    }
    Ok(Config::new(Some(config_path)))
}

/// create pdf and comment folders if they do not exist
fn init_folders(config: &Config) -> Result<(), IOError> {
    for path in &[&config.pdf.folder, &config.comment.folder] {
        let target_path = PathBuf::from(path);
        if target_path.exists() {
            println!("pdf folder exists, not creaeting.");
        } else {
            create_dir(&target_path)?;
            println!("created pdf folder at {}", &target_path.to_string_lossy());
        }
    }
    Ok(())
}

/// copy journal database to database location. run full db set up script.
fn init_database(config: &Config) -> Result<(), IOError> {
    let journal_db_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/journal.sqlite");
    if !journal_db_path.exists() { copy(journal_db_path, &config.journal_db)?; }
    let mut sql_query = String::new();
    let sql_migration_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migration/20180516-full-db/up.sql");
    let mut f = File::open(&sql_migration_path).expect(&format!("sql migration file at {} not found!", &sql_migration_path.to_string_lossy()));
    f.read_to_string(&mut sql_query).expect("error reading sql migration file");
    let conn = Connection::open(&config.database).expect("cannot open database");
    conn.execute_batch(&sql_query).expect("Error applying migration code");
    Ok(())
}

lazy_static! {
    pub static ref CONFIG: Config = Config::new(None);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_config() {
        let temp_config = Config::new(Some("test/data/bibrs-test.toml".into()));
        assert_eq!(temp_config.comment.extension[0], "txt");
        assert_eq!(temp_config.pdf.folder, PathBuf::from("/home/palpatine/Sync/paper/pdf/"));
    }
}

