use termion::{color, style};
use crate::database::{SqliteBibDB, BibDataBase};

/// add or delete keywords
/// print the resulting entry, with new keywords in red and deleted keywords in invert color
pub fn keywords(citation: &str, mut add: Vec<String>, mut del: Vec<String>) {
    let conn = SqliteBibDB::new(None);
    let old_entry = conn.get_item(citation).expect(&format!("Cannot find entry {}", &citation));
    if add.len() > 0 {  // non-exsitng keywords to add
        add = add.into_iter().filter(
            |x| !old_entry.keywords.contains(x)
        ).collect::<Vec<String>>();
    }
    let has_new = (add.len() > 0);
    if del.len() > 0 {  // existing keywords to delete
        del = del.into_iter().filter(
            |x| old_entry.keywords.contains(x)
        ).collect::<Vec<String>>();
    }
    let has_del = (del.len() > 0);
    if !(has_new || has_del) {
        println!("{}\n\tKeywords: {}", old_entry.to_str(), old_entry.keywords.join(", "));
        return
    }
    if has_new {conn.add_keywords(&old_entry.citation, &add);}
    if has_del {conn.del_keywords(&old_entry.citation, &del);}
    let new_entry = conn.get_item(citation).unwrap();
    let (retained_terms, new_terms): (Vec<String>, Vec<String>) = new_entry.keywords.clone().into_iter()
        .partition(|x| old_entry.keywords.contains(&x));
    let deleted_terms = old_entry.keywords.into_iter().filter(|x| !new_entry.keywords.contains(x)).collect::<Vec<String>>();
    let mut keywords: Vec<(String, String)> = new_terms.into_iter().map(
        move |x| (x.clone(), format!("{}{}{}", color::Fg(color::Red), x, color::Fg(color::Reset)))
    ).collect();
    keywords.extend(deleted_terms.into_iter().map(
        move |x| (x.clone(), format!("{}{}{}{}{}", color::Fg(color::Blue), style::Invert, x, style::Reset, color::Fg(color::Reset)))
    ));
    keywords.extend(retained_terms.into_iter().map(move |x| (x.clone(), x)));
    keywords.sort();
    let keywords_str = keywords.into_iter().map(|x| x.1).collect::<Vec<String>>().join(", ");
    println!("{}\n\tKeywords: {}", new_entry.to_str(), keywords_str);
}
