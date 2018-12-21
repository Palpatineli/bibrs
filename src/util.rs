use std::borrow::Cow;
use std::io::Result;

pub trait ToTitleCase {
    fn to_title<'a>(&'a self) -> Cow<'a, str>;
}

/// Allow raw string to be converted to title case if it's the first letter after braces, spaces
/// and periods.
impl ToTitleCase for String {
    fn to_title<'a>(&'a self) -> Cow<'a, str> {
        let mut in_curly = 0;
        let mut space = true;
        self.chars().map(|cur|  // depends on state, cannot convert to parallel
            match cur {
                ' ' | '.' | ',' | '?' => { space = true; cur },
                '{' => { space = false; in_curly += 1; cur },
                '}' => { space = false; in_curly -= 1; cur },
                _ if in_curly > 0 => { space = false; cur },
                _ => {
                    if space {space = false; cur.to_ascii_uppercase()}
                    else {cur.to_ascii_lowercase()}
                }
            }
        ).collect::<Cow<'a, str>>()
    }
}

pub trait BibFileExt {
    fn spawn(&mut self) -> Result<()>;
}

pub struct CommentFile { file_path: String, }
impl CommentFile {
    pub fn new(file_path: String) -> Self { CommentFile{ file_path: file_path.clone() }}
}
pub struct PdfFile { file_path: String, }

impl BibFileExt for CommentFile {
    fn spawn(&mut self) -> Result<()> {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_titlecase() {
        let temp_str = "this iS crazy YEAH. {NoT}, {yeAs}".to_owned();
        assert_eq!(temp_str.to_title(), "This Is Crazy Yeah. {NoT}, {yeAs}")
    }
}
