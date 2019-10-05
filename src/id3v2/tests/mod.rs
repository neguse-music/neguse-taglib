mod tools;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::DateTime;
use crate::Image;
use crate::TagOption;
use crate::Tags;

#[test]
fn write_test() {
    let path = "testfiles/test-id3v2-write.mp3";
    let mut input = File::open("testfiles/id3v24-utf8-jpeg-unsynchronized.mp3").unwrap();
    let mut output = File::create(path).unwrap();
    let tags = Tags::mixed();
    super::set(&mut input, &mut output, &tags).unwrap();

    let mut input = File::open("testfiles/id3v24-utf8-jpeg-unsynchronized.mp3").unwrap();
    let mut output = File::open(path).unwrap();

    let ideal = super::get(&mut input).unwrap();
    let tags = super::get(&mut output).unwrap();

    fs::remove_file(path).unwrap();
    assert_eq!(tags, ideal);
}

#[test]
fn id3v22_read_test() {
    let (tags, image) = read_data_in("id3v22-utf16le-jpeg");

    if image == Image::None {
        panic!("Expected image");
    }

    let ideal = Tags {
        title: TagOption::Some("example song".to_string()),
        artist: TagOption::Some("example artist".to_string()),
        album: TagOption::Some("example album".to_string()),
        album_artist: TagOption::Some("example album artist".to_string()),
        front_cover: TagOption::Some(image),
        ..Default::default()
    };

    assert_eq!(tags, ideal);
}

#[test]
fn id3v23_read_test() {
    let (tags, image) = read_data_in("id3v23-utf16le-jpeg");

    if image == Image::None {
        panic!("Expected image");
    }

    let ideal = Tags {
        title: TagOption::Some("NEXT FLIP FLAPPING！".to_string()),
        artist: TagOption::Some("TO-MAS".to_string()),
        album: TagOption::Some("TVアニメ『フリップフラッパーズ』オリジナルサウンドトラック 「Welcome to Pure Illusion」".to_string()),
        album_artist: TagOption::Some("Flip Flappers".to_string()),
        composer: TagOption::Some("伊藤真澄".to_string()),

        genre: TagOption::Some("Anime".to_string()),

        date: DateTime::from_iso_8601("2017").into(),

        track_number: TagOption::Some(21),
        disc_number: TagOption::Some(1),
        disc_total: TagOption::Some(2),

        front_cover: TagOption::Some(image),
        ..Default::default()
    };
    assert_eq!(tags, ideal);
}

#[test]
fn id3v23_unsynch_read_test() {
    let (tags, image) = read_data_in("id3v23-utf16le-unsynchronized");

    if image != Image::None {
        panic!("Expected no image");
    }

    let ideal = Tags {
        title: TagOption::Some("My babe just cares for me".to_string()),
        artist: TagOption::Some("Nina Simone".to_string()),
        album: TagOption::Some("100% Jazz".to_string()),

        track_number: TagOption::Some(3),

        ..Default::default()
    };
    assert_eq!(tags, ideal);
}

#[test]
fn id3v24_read_test() {
    let (tags, image) = read_data_in("id3v24-utf8-png");

    if image == Image::None {
        panic!("Expected image");
    }

    let ideal = Tags {
        title: TagOption::Some("EyeCatch".to_string()),
        artist: TagOption::Some("伊賀拓郎".to_string()),
        album: TagOption::Some("TVアニメ「月がきれい」サウンドコレクション".to_string()),
        album_artist: TagOption::Some("月がきれい".to_string()),
        date: DateTime::from_iso_8601("2017").into(),
        track_number: TagOption::Some(5),
        genre: TagOption::Some("Anime".to_string()),
        front_cover: TagOption::Some(image),
        ..Default::default()
    };
    assert_eq!(tags, ideal);
}

#[test]
fn id3v24_unsynchronized_read_test() {
    let (tags, image) = read_data_in("id3v24-utf8-jpeg-unsynchronized");

    if image == Image::None {
        panic!("Expected image");
    }

    let ideal = Tags {
        title: TagOption::Some("Test Name".to_string()),

        artist: TagOption::Some("Test Artist".to_string()),
        album: TagOption::Some("Test Album".to_string()),
        album_artist: TagOption::Some("Test Album Artist".to_string()),
        composer: TagOption::Some("Test Composer".to_string()),

        grouping: TagOption::Some("Test Grouping".to_string()),

        bpm: TagOption::Some(96),
        is_compilation: TagOption::Some(true),

        date: DateTime::from_iso_8601("2008-12-29").into(),

        track_number: TagOption::Some(7),
        track_total: TagOption::Some(16),
        disc_number: TagOption::Some(3),
        disc_total: TagOption::Some(4),

        genre: TagOption::Some("Classical".to_string()),

        comment: TagOption::Some("Test Comments".to_string()),

        sort_title: TagOption::Some("Test Title Sort Order".to_string()),
        sort_album: TagOption::Some("Test Album Sort Order".to_string()),
        sort_artist: TagOption::Some("Test Artist Sort Order".to_string()),
        sort_album_artist: TagOption::Some("Test Alb.Art. Sort Order".to_string()),
        front_cover: TagOption::Some(image),
        ..Default::default()
    };
    assert_eq!(tags, ideal);
}

fn read_data_in(s: &str) -> (Tags, Image) {
    let mut file = File::open(format!("testfiles/{}.mp3", s)).unwrap();
    let tags = super::get(&mut file).unwrap();

    let image = {
        if Path::new(format!("testfiles/{}-cover.png", s).as_str()).exists() {
            let mut vec = Vec::new();

            let mut file = File::open(format!("testfiles/{}-cover.png", s)).unwrap();

            file.read_to_end(&mut vec).unwrap();
            Image::PNG(vec)
        } else if Path::new(format!("testfiles/{}-cover.jpg", s).as_str()).exists() {
            let mut vec = Vec::new();

            let mut file = File::open(format!("testfiles/{}-cover.jpg", s)).unwrap();

            file.read_to_end(&mut vec).unwrap();
            Image::JPEG(vec)
        } else {
            Image::None
        }
    };

    return (tags, image);
}
