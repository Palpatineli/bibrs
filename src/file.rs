use std::process::Command;
use std::{io::{Result, Error, ErrorKind}, path::{PathBuf, Path}};
use std::fs::{remove_file, rename, DirEntry};

use crate::model::Entry;
use crate::util::ToTitleCase;
use crate::formatter::BibPrint;
pub use crate::config::{FileHandler, CONFIG};

impl FileHandler {
    fn open(&self, file_path: &Path) -> Result<()> {
        Command::new(&self.opener).arg(file_path).spawn().map(|_| ())
    }

    /// search for a file in self.folder and has file_name while having ext in self.extension
    /// if not found then just use the first one in self.extension
    fn search(&self, file_name: &str) -> PathBuf {
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

    /// Find the most recently accessed file in folder, searched by extension by list order
    pub fn search_temp(&self) -> Result<PathBuf> {
        let extensions = &self.extension;
        let paths = self.folder.read_dir()?.filter_map(Result::ok)
            .filter( |x| match x.file_type() {Ok(y) => y.is_file(), Err(_) => false}); 
        let mut found_path: Vec<Vec<DirEntry>> = Vec::new();
        for _ in 0..extensions.len() { found_path.push(Vec::new()) }
        for path in paths {
            match path.file_name().to_str() {
                Some(y) => {
                    for (i, ext) in extensions.iter().enumerate() {
                        if y.ends_with(ext) { found_path[i].push(path); break }
                    }
                },
                None => continue
            }
        }
        for mut list in found_path.into_iter() {
            if list.len() > 0 {
                list.sort_by_key(|x| x.metadata().unwrap().accessed().unwrap());
                return Ok(self.folder.join(list[list.len() - 1].path().file_name().ok_or_else(
                            || Error::new(ErrorKind::InvalidData,
                                          format!("File in folder {} not readable. Please Try again", self.folder.to_str().unwrap())))?));
            }
        }
        Err(Error::new(ErrorKind::NotFound, format!("{} files not found in {}", self.extension[0], self.folder.to_str().unwrap())))
    }
}

pub struct File<'a> { pub path: PathBuf, handler: &'a FileHandler, }

pub trait BibFile {
    fn open(&self) -> Result<()>;
    fn path(&self) -> &Path;
    fn remove(self) -> Result<()>;
}

impl<'a> BibFile for File<'a> {
    fn open(&self) -> Result<()> { self.handler.open(&self.path) }
    fn path(&self) -> &Path { &self.path }
    fn remove(self) -> Result<()> { remove_file(&self.path) }
}

impl<'a> File<'a> {
    pub fn new(name: &str, file_type: &str) -> Self {
        let handler = match file_type {
            "pdf" => &CONFIG.pdf,
            "comment" => &CONFIG.comment,
            "bib" | "temp_bib" | "temp_pdf" => { panic!("use File::temp to create temp files.") },
            _ => panic!(format!("Wrong file type {}", file_type))
        };
        File{path: handler.search(name), handler}
    }

    pub fn temp(file_type: &str) -> Result<Self> { 
        let handler = match file_type {
            "bib" | "temp_bib" => &CONFIG.temp_bib,
            "temp_pdf" => &CONFIG.temp_pdf,
            "pdf" | "comment" => { return Err(Error::new(ErrorKind::InvalidInput, "use File::New to create none temp files that have specified file names.")) },
            _ => panic!(format!("Wrong file type {}", file_type))
        };
        Ok(File{path: handler.search_temp()?, handler})
    }

    pub fn mv(&self, other_handler: &FileHandler) -> Result<()> {
        let target_path = other_handler.folder.join(self.path.file_name().ok_or_else(
                || Error::new(ErrorKind::InvalidInput, "file path has no file name."))?);
        rename(&self.path, target_path)
    }
}

impl Entry {
    pub fn to_comment(&self) -> String {
        let author_str = if self.authors.len() > 0 {
            self.authors.to_bib()
        } else if self.editors.len() > 0 {
            self.editors.to_bib()
        } else {
            "".to_owned()
        };
        format!("% {}\n% {}\n% {}", self.title.to_title(), author_str, self.year)
    }
}
