use std;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

use crate::Image;
use crate::TagOption;
use crate::Tags;

#[test]
fn m4a_rewrite_tag_test() {
    let ideal = super::get_tags("testfiles/id3v24-utf8-png.mp3").unwrap();

    let src_path = "testfiles/m4a-alac.m4a";
    let path = "testfiles/test-write-m4a.m4a";

    fs::copy(src_path, path).unwrap();

    // use std::time::Instant;
    // let now = Instant::now();
    super::set_tags(path, &ideal).unwrap();
    // let after = now.elapsed();
    // let after = after.as_secs() * 1000000000 + after.subsec_nanos() as u64;
    // println!("Nanoseconds: {}", after);
    let tags = super::get_tags(path).unwrap();

    fs::remove_file(path).unwrap();
    assert_eq!(ideal, tags);
}

#[test]
fn flac_rewrite_tag_test() {
    let ideal = super::get_tags("testfiles/id3v24-utf8-png.mp3").unwrap();

    let src_path = "testfiles/flac.flac";
    let path = "testfiles/test-write-flac.flac";

    fs::copy(src_path, path).unwrap();

    // use std::time::Instant;
    // let now = Instant::now();
    super::set_tags(path, &ideal).unwrap();
    // let after = now.elapsed();
    // let after = after.as_secs() * 1000000000 + after.subsec_nanos() as u64;
    // println!("Nanoseconds: {}", after);
    let tags = super::get_tags(path).unwrap();

    fs::remove_file(path).unwrap();
    assert_eq!(ideal, tags);
}

#[test]
fn vorbis_comment_encode_test() {
    use crate::vorbis;

    let path = "testfiles/id3v24-utf8-jpeg-unsynchronized.mp3";
    let tags = super::get_tags(path).unwrap();

    let vc = vorbis::from_tags(&tags, true);
    let recovered = vorbis::get_tags(&mut std::io::Cursor::new(&vc)).unwrap();

    assert_eq!(tags, recovered);
}

#[test]
fn flac_test() {
    let path = "testfiles/flac.flac";
    let tags = Tags {
        title: TagOption::Some("drippy".to_string()),
        artist: TagOption::Some("corsica".to_string()),
        album: TagOption::Some("test drips".to_string()),
        track_number: TagOption::Some(1),
        track_total: TagOption::Some(4),
        genre: TagOption::Some("recording".to_string()),
        front_cover: TagOption::Some({
            let mut vec = Vec::new();
            let mut file = File::open("testfiles/flac-cover.jpg").unwrap();
            file.read_to_end(&mut vec).unwrap();
            Image::JPEG(vec)
        }),
        ..Default::default()
    };

    assert_eq!(super::get_tags(path).unwrap(), tags);
}

#[test]
fn id3v2_write_test() {
    let src_path = "testfiles/id3v24-utf8-png.mp3";
    let path = "testfiles/test-write-id4v24.mp3";

    let tags = super::get_tags(src_path).unwrap();

    fs::copy(src_path, path).unwrap();

    // use std::time::Instant;
    // let now = Instant::now();
    super::set_tags(path, &Tags::mixed()).unwrap();
    // let after = now.elapsed();
    // let after = after.as_secs() * 1000000000 + after.subsec_nanos() as u64;
    // println!("Nanoseconds: {}", after);
    let new_tags = super::get_tags(path).unwrap();

    fs::remove_file(path).unwrap();
    assert_eq!(tags, new_tags);
}

#[test]
fn id3v1_test() {
    let path = "testfiles/id3v1.mp3";
    let mut file = File::open(path).unwrap();
    assert_eq!(
        super::get_tags(path).unwrap(),
        super::id3v1::get(&mut file).unwrap()
    );
}

#[test]
fn id3v2_test() {
    let path = "testfiles/id3v24-utf8-png.mp3";
    let mut file = File::open(path).unwrap();
    assert_eq!(
        super::get_tags(path).unwrap(),
        super::id3v2::get(&mut file).unwrap()
    );
}

#[test]
#[should_panic]
fn invalid_file_test() {
    super::get_tags("testfiles/asdfasdf.mp3").unwrap();
}
