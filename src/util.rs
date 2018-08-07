pub trait ToTitleCase {
    fn to_title(&self) -> String;
}

impl ToTitleCase for String {
    fn to_title(&self) -> String {
        let mut in_curly = 0;
        let mut space = true;
        self.chars().map(|cur| 
            if cur.is_whitespace() { space = true; cur }
            else if cur == '{' { space = false; in_curly += 1; cur }
            else if cur == '}' { space = false; in_curly -= 1; cur }
            else if in_curly > 0 { space = false; cur }
            else { if space {space = false; cur.to_ascii_uppercase()} else {cur.to_ascii_lowercase()} }
        ).collect::<String>()
    }
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
