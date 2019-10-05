use crate::DateTime;
use crate::Image;

#[derive(PartialEq, Debug, Default)]
pub struct Tags {
    pub title: TagOption<String>,

    pub album: TagOption<String>,
    pub artist: TagOption<String>,
    pub album_artist: TagOption<String>,
    pub composer: TagOption<String>,

    pub grouping: TagOption<String>,

    pub genre: TagOption<String>,

    // ISO 8601
    pub date: TagOption<DateTime>,

    pub track_number: TagOption<i64>,
    pub track_total: TagOption<i64>,

    pub disc_number: TagOption<i64>,
    pub disc_total: TagOption<i64>,

    pub bpm: TagOption<i64>,

    pub is_compilation: TagOption<bool>,

    pub comment: TagOption<String>,

    pub sort_title: TagOption<String>,
    pub sort_album: TagOption<String>,
    pub sort_artist: TagOption<String>,
    pub sort_album_artist: TagOption<String>,
    pub sort_composer: TagOption<String>,

    pub front_cover: TagOption<Image>,

    pub rating: TagOption<u8>,
}

impl Tags {
    pub fn none() -> Tags {
        Default::default()
    }
    pub fn mixed() -> Tags {
        Tags {
            title: TagOption::Mixed,
            album: TagOption::Mixed,
            artist: TagOption::Mixed,
            album_artist: TagOption::Mixed,
            composer: TagOption::Mixed,
            grouping: TagOption::Mixed,
            genre: TagOption::Mixed,
            date: TagOption::Mixed,
            track_number: TagOption::Mixed,
            track_total: TagOption::Mixed,
            disc_number: TagOption::Mixed,
            disc_total: TagOption::Mixed,
            bpm: TagOption::Mixed,
            is_compilation: TagOption::Mixed,
            comment: TagOption::Mixed,
            sort_title: TagOption::Mixed,
            sort_album: TagOption::Mixed,
            sort_artist: TagOption::Mixed,
            sort_album_artist: TagOption::Mixed,
            sort_composer: TagOption::Mixed,
            front_cover: TagOption::Mixed,
            rating: TagOption::Mixed,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum TagOption<T> {
    Some(T),
    Mixed, // the "do not overwrite" option
    None,
}

impl<T> From<Option<T>> for TagOption<T> {
    fn from(option: Option<T>) -> Self {
        match option {
            Some(x) => TagOption::Some(x),
            None => TagOption::None,
        }
    }
}

impl<T> TagOption<T> {
    pub fn to_option(self) -> Option<T> {
        match self {
            TagOption::Some(x) => Some(x),
            _ => None,
        }
    }

    pub fn unwrap_or(self, def: T) -> T {
        match self {
            TagOption::Some(x) => x,
            _ => def,
        }
    }
    pub fn unwrap(self) -> T {
        match self {
            TagOption::Some(val) => val,
            _ => panic!("called `TagOption::unwrap()` on a `None` or `Mixed` value"),
        }
    }
    pub fn is_some(&self) -> bool {
        match *self {
            TagOption::Some(_) => true,
            _ => false,
        }
    }
    pub fn is_none(&self) -> bool {
        match *self {
            TagOption::None => true,
            _ => false,
        }
    }
    pub fn as_ref(&self) -> TagOption<&T> {
        match *self {
            TagOption::Some(ref x) => TagOption::Some(x),
            TagOption::Mixed => TagOption::Mixed,
            TagOption::None => TagOption::None,
        }
    }
}

impl<T> Default for TagOption<T> {
    fn default() -> TagOption<T> {
        TagOption::None
    }
}
