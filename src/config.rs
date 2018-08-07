extern crate toml;
extern crate dirs;

use std::fs::File;
use std::io::Read;

use self::dirs::home_dir;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct FileHandler {
    folder: PathBuf,
    extension: Vec<String>,
    opener: String,
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

pub fn read_config(config_path: Option<PathBuf>) -> Config {
    let mut config_file = File::open(config_path.unwrap_or(home_dir().unwrap().join(".config/bibrs.toml")))
        .expect("Configuration File not found!");
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

lazy_static! {
    pub static ref CONFIG: Config = read_config(None);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_config() {
        let temp_config = read_config(Some(PathBuf::from("test/data/bibrs-test.toml")));
        assert_eq!(temp_config.comment.extension[0], ".txt");
        assert_eq!(temp_config.pdf.folder, PathBuf::from("/home/palpatine/Dropbox/Paper_test/pdf/"));
    }
}
