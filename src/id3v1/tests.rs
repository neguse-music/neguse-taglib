use crate::DateTime;
use crate::TagOption;
use crate::Tags;
use std::fs::File;

#[test]
fn id3v1_tags_test() {
    let mut file = File::open("testfiles/id3v1.mp3").unwrap();
    let tag = super::get(&mut file).unwrap();
    let orig = Tags {
        title: TagOption::Some("ID3v1 Test Track".to_string()),
        album: TagOption::Some("Album Name".to_string()),
        artist: TagOption::Some("Artist Name".to_string()),
        genre: TagOption::Some("Classical".to_string()),
        date: TagOption::Some(DateTime {
            year: Some(2017),
            ..Default::default()
        }),
        track_number: TagOption::Some(1),
        // test some ISO 8859-1 characters
        comment: TagOption::Some("Comment æÖÆ¶¦àçàö".to_string()),
        ..Default::default()
    };
    assert_eq!(tag, orig);
}
