use std::fs::File;
use std::io::prelude::*;

use crate::DateTime;
use crate::Image;
use crate::TagOption;
use crate::Tags;

#[test]
fn aac_test() {
    let mut file = File::open("testfiles/m4a-aac.m4a").unwrap();
    let tags = super::get(&mut file).unwrap();
    let ideal = Tags {
        title: TagOption::Some("(Segue) - Nathan Adler, Pt. 2".to_string()),
        artist: TagOption::Some("David Bowie".to_string()),
        album: TagOption::Some("Outside".to_string()),
        album_artist: TagOption::Some("David Bowie".to_string()),

        genre: TagOption::Some("Rock".to_string()),

        date: DateTime::from_iso_8601("1995-09-26T08:00:00").into(),

        track_number: TagOption::Some(18),
        track_total: TagOption::Some(20),
        disc_number: TagOption::Some(1),
        disc_total: TagOption::Some(1),

        front_cover: TagOption::Some({
            let mut vec = Vec::new();
            let mut file = File::open("testfiles/m4a-aac-cover.jpg").unwrap();
            file.read_to_end(&mut vec).unwrap();
            Image::JPEG(vec)
        }),
        ..Default::default()
    };

    assert_eq!(tags, ideal);
}
#[test]
fn alac_test() {
    let mut file = File::open("testfiles/m4a-alac.m4a").unwrap();
    let tags = super::get(&mut file).unwrap();
    let ideal = Tags {
        title: TagOption::Some("Various Jingles".to_string()),
        artist: TagOption::Some("Ludvig Forssell".to_string()),
        album: TagOption::Some("Metal Gear Solid Ⅴ: The Phantom Pain".to_string()),
        album_artist: TagOption::Some("Metal Gear Series".to_string()),
        composer: TagOption::Some("Ludvig Forssell".to_string()),

        genre: TagOption::Some("Game Soundtrack".to_string()),

        date: DateTime::from_iso_8601("2015").into(),

        track_number: TagOption::Some(28),
        track_total: TagOption::Some(28),
        disc_number: TagOption::Some(2),
        disc_total: TagOption::Some(2),

        sort_album: TagOption::Some("Metal Gear Solid Ⅴ: The Phantom Pain".to_string()),
        sort_album_artist: TagOption::Some("Metal Gear Series".to_string()),

        front_cover: TagOption::Some({
            let mut vec = Vec::new();
            let mut file = File::open("testfiles/m4a-alac-cover.png").unwrap();
            file.read_to_end(&mut vec).unwrap();
            Image::PNG(vec)
        }),
        ..Default::default()
    };

    assert_eq!(tags, ideal);
}
