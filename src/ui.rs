pub mod add_item;

use std::io;
use std::marker::PhantomData;
use std::convert::{TryFrom, AsRef};
use termion::color;
use derive_more::Display;
use lazy_static::lazy_static;

pub trait Controls {
    fn controls() -> &'static Vec<(Option<usize>, String)>;
    fn abort() -> Self;
}

pub trait UIResponse = TryFrom<String, Error=io::Error> + Controls;

#[derive(Display)]
pub enum MsgType {
    Conflict,
    #[display(fmt="not found")]
    Missing,
    Info,
}

pub struct UI<T> where T: UIResponse {
    msg_type: MsgType,
    phantom: PhantomData<T>,
}

macro_rules! fg {
    ($col:ident, $content:expr) => {
        format!("{}{}{}", color::Fg(color::$col), $content, color::Fg(color::Reset))
    }
}

impl<T> UI<T> where T: UIResponse {
    #[inline]
    pub fn new(msg_type: MsgType) -> Self {
        Self{msg_type, phantom: PhantomData}
    }

    pub fn prompt<M>(&self, msg: M) -> io::Result<T> where M: AsRef<str> {
        let control: String = T::controls().iter().cloned().map(|(switch_idx, text)|
            match switch_idx {
                Some(x) => format!("{}{}{}", &text[..x], fg!(Red, &text[x..x + 1]), &text[x + 1..]),
                None => text,
            }
        ).collect::<Vec<String>>().join(", ");
        println!("[{}] {}\n{}", fg!(Magenta, self.msg_type), msg.as_ref(), control);
        let mut input_string = String::new();
        io::stdin().read_line(&mut input_string)?;
        T::try_from(input_string)
    }
}

pub enum SimpleInputs {
    Abort,
    Continue,
}

impl TryFrom<String> for SimpleInputs {
    type Error = io::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "a" => Self::Abort,
            "c" => Self::Continue,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("invalid input: {}", value)))
        })
    }
}

lazy_static!{static ref CONTROLS_STR: Vec<(Option<usize>, String)> = vec![(Some(0), "abort".to_owned()), (Some(0), "continue".to_owned())];}

impl Controls for SimpleInputs {
    #[inline]
    fn controls() -> &'static Vec<(Option<usize>, String)> { &CONTROLS_STR }
    #[inline]
    fn abort() -> Self { SimpleInputs::Abort }
}

pub enum CitationInputs {
    Abort,
    Update,
    Changed(String),
}

lazy_static!{static ref CITATION_STR: Vec<(Option<usize>, String)>
    = vec![(Some(0), "abort".to_owned()), (Some(0), "update entry".to_owned()), (None, "input new citation".to_owned())];}

impl Controls for CitationInputs {
    #[inline]
    fn controls() -> &'static Vec<(Option<usize>, String)> { &CITATION_STR }
    #[inline]
    fn abort() -> Self { CitationInputs::Abort }
}

impl TryFrom<String> for CitationInputs {
    type Error = io::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "a" => Self::Abort,
            "u" => Self::Update,
            _ => Self::Changed(value)
        })
    }
}

pub enum JournalInputs {
    Abort,
    Update((String, String, String)),
}

lazy_static!{
    static ref JOURNAL_STR: Vec<(Option<usize>, String)> = vec![(Some(0), "abort".to_owned()), (None, "type new name in [full, abbreviation, abbreviation without dots]".to_owned())];
}

impl Controls for JournalInputs {
    #[inline]
    fn controls() -> &'static Vec<(Option<usize>, String)> { &JOURNAL_STR }
    #[inline]
    fn abort() -> Self { JournalInputs::Abort }
}

impl TryFrom<String> for JournalInputs {
    type Error = io::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "a" => Ok(Self::Abort),
            _ => { match value.split(",").map(|x| x.trim()).collect::<Vec<&str>>()[..] {
                [one, two, three] => Ok(Self::Update((one.to_owned(), two.to_owned(), three.to_owned()))),
                _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "please type three strings separated with comma"))
            }}
        }
    }
}
