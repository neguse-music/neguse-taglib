use std;

use std::io::prelude::*;

use crate::DateTime;
use crate::Error;
use crate::TagOption;
use crate::Tags;

use crate::tools::decode_int_le_u32;
use crate::tools::encode_int_le_u32;

extern crate base64;

macro_rules! write_text {
    ($vec:ident, $text:expr, $cc:ident) => {{
        $vec.extend_from_slice(&encode_int_le_u32($text.len() as u32));
        $vec.extend_from_slice($text.as_bytes());
        $cc += 1;
    }};
}
macro_rules! write_comment {
    ($vec:ident, $text:expr, $id:expr, $cc:ident) => {{
        if let TagOption::Some(ref t) = $text {
            if t.as_str() != "" {
                write_text!($vec, format!("{}={}", $id, t), $cc);
            }
        }
    }};
}
macro_rules! write_num {
    ($vec:ident, $text:expr, $id:expr, $cc:ident) => {{
        if let TagOption::Some(ref t) = $text {
            if t.to_string().as_str() != "" {
                write_text!($vec, format!("{}={}", $id, t), $cc);
            }
        }
    }};
}

pub fn from_tags(tags: &Tags, include_image: bool) -> Vec<u8> {
    let mut vec = Vec::new();

    lazy_static! {
        static ref VERSION: String = format!(
            "neguse-taglib {}",
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")
        );
    }

    // write vendor info
    vec.extend_from_slice(&encode_int_le_u32(VERSION.len() as u32));
    vec.extend_from_slice(VERSION.as_bytes());

    // save offset where comment length will be
    let cc_offset = vec.len();
    // and create the counter
    let mut cc: u32 = 0;
    // reserve space for comment list length
    vec.extend_from_slice(&[0, 0, 0, 0]);

    // start writing comments
    write_comment!(vec, tags.title, "TITLE", cc);
    write_comment!(vec, tags.album, "ALBUM", cc);
    write_comment!(vec, tags.artist, "ARTIST", cc);
    write_comment!(vec, tags.album_artist, "ALBUMARTIST", cc);
    write_comment!(vec, tags.composer, "COMPOSER", cc);
    write_comment!(vec, tags.grouping, "GROUPING", cc);
    write_comment!(vec, tags.genre, "GENRE", cc);

    if let TagOption::Some(ref t) = tags.date {
        if t.to_iso_8601().as_str() != "" {
            write_text!(vec, format!("DATE={}", t), cc);
        }
    }

    write_num!(vec, tags.track_number, "TRACKNUMBER", cc);
    write_num!(vec, tags.track_total, "TRACKTOTAL", cc);
    write_num!(vec, tags.disc_number, "DISCNUMBER", cc);
    write_num!(vec, tags.disc_total, "DISCTOTAL", cc);
    write_num!(vec, tags.bpm, "BPM", cc);

    if let crate::TagOption::Some(true) = tags.is_compilation {
        write_text!(vec, "COMPILATION=1", cc);
    }

    write_comment!(vec, tags.comment, "COMMENT", cc);
    write_comment!(vec, tags.sort_title, "TITLESORT", cc);
    write_comment!(vec, tags.sort_album, "ALBUMSORT", cc);
    write_comment!(vec, tags.sort_artist, "ARTISTSORT", cc);
    write_comment!(vec, tags.sort_album_artist, "ALBUMARTISTSORT", cc);
    write_comment!(vec, tags.sort_composer, "COMPOSERSORT", cc);

    // if we have image
    if let crate::TagOption::Some(ref i) = tags.front_cover {
        // check if we want to write one first
        if include_image && i.is_some() {
            // then get the metadata block
            use crate::flac::get_picture_block;
            let img = base64::encode(&get_picture_block(i).unwrap());
            let string = format!("{}={}", "METADATA_BLOCK_PICTURE", img);
            vec.extend_from_slice(&encode_int_le_u32(string.len() as u32));
            vec.extend_from_slice(string.as_bytes());
            cc += 1;
        }
    }

    let count = encode_int_le_u32(cc);
    for i in 0..4 {
        vec[cc_offset + i] = count[i];
    }

    vec
}

macro_rules! parse_num {
    ($tag:expr, $str:expr, $type:ty) => {{
        if let Ok(x) = $str.parse::<$type>() {
            $tag = TagOption::Some(x);
        }
    }};
}

pub fn get_tags<T: Read + Seek>(input: &mut T) -> Result<Tags, Error> {
    let mut buf: [u8; 4] = [0; 4];

    input.read(&mut buf)?;

    // skip description
    {
        let len = decode_int_le_u32(&buf);
        input.seek(std::io::SeekFrom::Current(len as i64))?;
    }

    // get comment count
    input.read(&mut buf)?;
    let count = decode_int_le_u32(&buf);

    let mut tags: Tags = Default::default();
    let mut found_best_cover = false;

    for _ in 0..count {
        input.read(&mut buf)?;
        let len = decode_int_le_u32(&buf) as usize;
        let mut vec: Vec<u8> = vec![0; len];
        input.read(&mut vec)?;
        let (tag, value) = match collect_tag(&vec) {
            Some(x) => x,
            None => continue,
        };

        match tag.as_str() {
            "TITLE" => tags.title = TagOption::Some(value),
            "ALBUM" => tags.album = TagOption::Some(value),
            "ARTIST" => tags.artist = TagOption::Some(value),
            "ALBUMARTIST" => tags.album_artist = TagOption::Some(value),
            "COMPOSER" => tags.composer = TagOption::Some(value),
            "GROUPING" => tags.grouping = TagOption::Some(value),
            "GENRE" => tags.genre = TagOption::Some(value),

            "DATE" => tags.date = DateTime::from_iso_8601(value.as_str()).into(),

            "TRACKNUMBER" => parse_num!(tags.track_number, value, i64),
            "TRACKTOTAL" => parse_num!(tags.track_total, value, i64),
            "DISCNUMBER" => parse_num!(tags.disc_number, value, i64),
            "DISCTOTAL" => parse_num!(tags.disc_total, value, i64),
            "BPM" => parse_num!(tags.bpm, value, i64),

            "COMPILATION" => {
                tags.is_compilation = TagOption::Some(match value.to_lowercase().as_str() {
                    "true" | "1" => true,
                    _ => false,
                })
            }

            "COMMENT" => tags.comment = TagOption::Some(value),
            "TITLESORT" => tags.sort_title = TagOption::Some(value),
            "ALBUMSORT" => tags.sort_album = TagOption::Some(value),
            "ARTISTSORT" => tags.sort_artist = TagOption::Some(value),
            "ALBUMARTISTSORT" => tags.sort_album_artist = TagOption::Some(value),
            "COMPOSERSORT" => tags.sort_composer = TagOption::Some(value),

            "METADATA_BLOCK_PICTURE" => {
                let vec = match base64::decode(&value) {
                    Ok(v) => v,
                    Err(x) => {
                        println!("{:?}", x);
                        continue;
                    }
                };

                use crate::flac::read_image;
                let l = vec.len() as u32;
                let (img, apic_type) = read_image(&mut std::io::Cursor::new(&vec), l)?;

                if apic_type == 0x03 || !found_best_cover {
                    tags.front_cover = TagOption::Some(img);
                    found_best_cover = apic_type == 0x03;
                }
            }

            _ => (),
        }
    }

    // do not check framing bit

    Ok(tags)
}

extern crate regex;
fn collect_tag(input: &[u8]) -> Option<(String, String)> {
    let input = String::from_utf8_lossy(&input);

    use self::regex::Regex;
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^([\x20-\x3C\x3E-\x7D]+)=(.*)$").unwrap();
    }
    match RE.captures(&input.into_owned()) {
        None => None,
        Some(c) => {
            Some((
                match c.get(1) {
                    None => return None, //shouldn't get hit
                    Some(s) => s.as_str().to_string().to_uppercase(),
                },
                match c.get(2) {
                    None => "".to_string(),
                    Some(s) => s.as_str().to_string(),
                },
            ))
        }
    }
}
