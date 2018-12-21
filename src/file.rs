use std::process::Command;
use std::io::Result;
use std::fs::{remove_file, rename};
use std::path::PathBuf;

pub trait BibFile {
    fn open(&self) -> Result<()> { Command::new(self.get_opener()).arg(self.get_file_path()).spawn().map(|_| ()) }
    fn remove(&self) -> Result<()> { remove_file(PathBuf::from(self.get_file_path())) }
    fn store(&self) -> Result<()> { rename(self.get_file_path(), self.get_target_path())}
    fn get_opener(&self) -> &str;
    fn get_file_path(&self) -> &str;
    fn get_target_path(&self) -> &str;
}

pub struct PdfFile { file_path: String, }
pub struct CommentFile { file_path: String, citation: String, }
pub struct BibtexFile { file_path: String, }

impl BibFile for PdfFile {
}
