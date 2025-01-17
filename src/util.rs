use std::borrow::Cow;

pub trait ToTitleCase {
    fn to_title(&self) -> Cow<str>;
}


#[macro_export]
macro_rules! str_hashset {
    ($($item:expr),*) => {{
        let mut temp_set = HashSet::new();
        $(temp_set.insert($item.to_owned());)*
        temp_set
    }};
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_titlecase() {
        let temp_str = "this iS crazy YEAH. {NoT}, {yeAs}".to_owned();
        assert_eq!(temp_str.to_title(), "This Is Crazy Yeah. {NoT}, {yeAs}")
    }
}
