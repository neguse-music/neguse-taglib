use crate::TagOption;
use crate::Tags;

// if new has tag, use it
// if not, use tag from old
// but if old tag is Empty, set it to None
macro_rules! delta {
    ($field:ident, $new:ident, $old:ident) => {{
        match $new.$field {
            TagOption::Some(ref x) => TagOption::Some(x.clone()),
            TagOption::Mixed => match $old.$field {
                TagOption::Some(ref x) => TagOption::Some(x.clone()),
                _ => TagOption::None,
            },
            TagOption::None => TagOption::None,
        }
    }};
}

// set up the tags to write by merging old and new tags
// and replacing resulting Empty values with None
pub fn delta(old: &Tags, new: &Tags) -> Tags {
    Tags {
        title: delta!(title, new, old),
        album: delta!(album, new, old),
        artist: delta!(artist, new, old),
        album_artist: delta!(album_artist, new, old),
        composer: delta!(composer, new, old),
        grouping: delta!(grouping, new, old),
        genre: delta!(genre, new, old),
        date: delta!(date, new, old),
        track_number: delta!(track_number, new, old),
        track_total: delta!(track_total, new, old),
        disc_number: delta!(disc_number, new, old),
        disc_total: delta!(disc_total, new, old),
        bpm: delta!(bpm, new, old),
        is_compilation: delta!(is_compilation, new, old),
        comment: delta!(comment, new, old),
        sort_title: delta!(sort_title, new, old),
        sort_album: delta!(sort_album, new, old),
        sort_artist: delta!(sort_artist, new, old),
        sort_album_artist: delta!(sort_album_artist, new, old),
        sort_composer: delta!(sort_composer, new, old),
        front_cover: delta!(front_cover, new, old),
        ..Tags::none()
    }
}
