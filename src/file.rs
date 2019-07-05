use std::process::Command;
use std::{io::Result, path::PathBuf};
use std::fs::{remove_file, rename};

pub use crate::config::{FileHandler, CONFIG};

impl FileHandler {
    pub fn open(&self, file_name: &str) -> Result<()> {
        Command::new(&self.opener).arg(self.path(file_name)).spawn().map(|_| ())
    }

    pub fn path(&self, file_name: &str) -> PathBuf {
        let mut target_path = self.folder.clone();
        target_path.push(&file_name);
        for ext in self.extension.iter() {
            target_path.set_extension(ext);
            if target_path.exists() {
                return target_path
            }
        }
        target_path.set_extension(&self.extension[0]);
        target_path
    }

    pub fn remove(&self, file_name: &str) -> Result<()> {
        let mut target_path = self.folder.clone();
        target_path.push(&file_name);
        remove_file(target_path)
    }
}

pub struct File<'a> { name: String, handler: &'a FileHandler, }

pub trait BibFile {
    fn open(&self) -> Result<()>;
    fn remove(self) -> Result<()>;
}

impl<'a> BibFile for File<'a> {
    fn open(&self) -> Result<()> { self.handler.open(&self.name) }
    fn remove(self) -> Result<()> { self.handler.remove(&self.name) }
}

impl<'a> File<'a> {
    pub fn new(name: &str, file_type: &str) -> Self {
        let handler = match file_type {
            "pdf" => &CONFIG.pdf,
            "comment" => &CONFIG.comment,
            "bib" | "temp_bib" => &CONFIG.temp_bib,
            "temp_pdf" => &CONFIG.temp_pdf,
            _ => panic!(format!("Wrong file type {}", file_type))
        };
        File{name: name.to_owned(), handler}
    }
    pub fn mv(&self, other_handler: &FileHandler) -> Result<()> {
        rename(self.handler.path(&self.name), other_handler.path(&self.name))
    }
}


#[cfg(test)]
mod tests {
    use dirs::home_dir;
    use super::*;
    #[test]
    fn test_hander() {
        let target = CONFIG.pdf.path("testing");
        println!("target: {:?}", target);
        assert_eq!(target, home_dir().unwrap().join(r"Sync/paper/pdf/testing.pdf"));
    }
}
