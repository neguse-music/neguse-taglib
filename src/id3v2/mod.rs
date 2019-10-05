use std;

use std::io::prelude::*;

use crate::DateTime;
use crate::Error;
use crate::TagOption;
use crate::Tags;

mod get;
mod read;
mod regex;
mod structure;
mod tools;

pub fn get<T: Read + Seek>(input: &mut T) -> Result<Tags, Error> {
    let header = r#try!(read::header(input));
    get_internal(input, &header)
}

macro_rules! write_string {
    ($vec:ident, $str:expr, $id:expr) => {{
        // frame id
        $vec.append(&mut tools::encode_frame_id($id)?);
        // size, $item + 2 (encoding & null)
        $vec.append(&mut tools::encode_synch_int($str.len() as u32 + 2, false)?);
        // no flags
        $vec.extend_from_slice(b"\x00\x00");
        // encoding (utf-8 always)
        $vec.push(0x03);
        // push the content
        $vec.extend_from_slice($str.as_str().as_bytes());
        // null-terminate
        $vec.push(0x00);
    }};
}
macro_rules! write_text_frame {
    ($vec:ident, $item:expr, $id:expr) => {{
        if let TagOption::Some(x) = $item {
            write_string!($vec, x, $id);
        }
    }};
}

// a good idea to trim id3v1 headers later
pub fn set<R: Read + Seek, W: Write>(
    input: &mut R,
    output: &mut W,
    new: &Tags,
) -> Result<(), Error> {
    use crate::tools::tags::delta;
    use std::io::SeekFrom;
    let (old_size, tags) = match read::header(input) {
        // id3v2 tag found
        Ok(h) => {
            let old = r#try!(get_internal(input, &h));
            (h.size as u64 + 10, delta(&old, new))
        }
        // id3v2 not found, and it's not an I/O error
        Err(Error::TagError(_)) => {
            use crate::id3v1;
            let old = match id3v1::get(input) {
                // found id3v1 tag
                Ok(t) => t,
                Err(_) => Default::default(),
            };
            (0, delta(&old, new))
        }
        Err(e) => return Err(e),
    };

    // construct the new tag
    let mut vec: Vec<u8> = Vec::with_capacity(old_size as usize); // at least size of old tag
    vec.extend_from_slice(b"ID3\x04\x00\x00"); // [0..6] id3v24; no flags
    vec.extend_from_slice(b"\x00\x00\x00\x00"); // [6..10] - reserve for size

    write_text_frame!(vec, tags.title, "TIT2");
    write_text_frame!(vec, tags.artist, "TPE1");
    write_text_frame!(vec, tags.album, "TALB");
    write_text_frame!(vec, tags.album_artist, "TPE2");
    write_text_frame!(vec, tags.composer, "TCOM");

    write_text_frame!(vec, tags.grouping, "TIT1");

    write_text_frame!(vec, tags.genre, "TCON");

    if let TagOption::Some(x) = tags.date {
        write_string!(vec, x.to_iso_8601(), "TDRC");
    }

    // track
    if tags.track_number.is_some() || tags.track_total.is_some() {
        let mut s = tags.track_number.unwrap().to_string();
        if let TagOption::Some(x) = tags.track_total {
            s.push_str(format!("/{}", x).as_str());
        }

        write_string!(vec, s, "TRCK");
    }
    // disc
    if tags.disc_number.is_some() || tags.disc_total.is_some() {
        let mut s = tags.disc_number.unwrap().to_string();
        if let TagOption::Some(x) = tags.disc_total {
            s.push_str(format!("/{}", x).as_str());
        }

        write_string!(vec, s, "TPOS");
    }

    if let TagOption::Some(i) = tags.bpm {
        write_string!(vec, i.to_string(), "TBPM");
    }

    // just push the whole compilation frame, there's only one possible.
    if let TagOption::Some(true) = tags.is_compilation {
        vec.extend_from_slice(b"TCMP\x00\x00\x00\x03\x00\x00\x031\x00");
    }

    write_text_frame!(vec, tags.sort_title, "TSOT");
    write_text_frame!(vec, tags.sort_artist, "TSOP");
    write_text_frame!(vec, tags.sort_album, "TSOA");
    write_text_frame!(vec, tags.sort_album_artist, "TSO2");
    write_text_frame!(vec, tags.sort_composer, "TSOC");

    // comment
    if let TagOption::Some(x) = tags.comment {
        // frame id
        vec.append(&mut tools::encode_frame_id("COMM")?);
        // size, text + 2 (encoding & null) + 3 (language) + 1 (null)
        vec.append(&mut tools::encode_synch_int(x.len() as u32 + 6, false)?);
        // no flags
        vec.extend_from_slice(b"\x00\x00");
        // encoding (utf-8 always)
        vec.push(0x03);
        // language and null description
        // iTunes needs this set to eng
        vec.extend_from_slice(b"eng\x00");
        // push the content
        vec.extend_from_slice(x.as_bytes());
        // null-terminate
        vec.push(0x00);
    }

    // finally, image.
    if let TagOption::Some(x) = tags.front_cover {
        let mime = x.mime();
        let image = x.unwrap();

        // frame id
        vec.append(&mut tools::encode_frame_id("APIC")?);
        // size
        vec.append(&mut tools::encode_synch_int(
            1 +                     // encoding
                    mime.len() as u32 + 1 + // mime type, null
                    1 +                     // picture type
                    1 +                     // empty description
                    image.len() as u32, // image length
            false,
        )?);

        // no flags
        vec.extend_from_slice(b"\x00\x00");
        // encoding (utf-8 always)
        vec.push(0x03);
        // mime
        vec.extend_from_slice(mime.as_bytes());
        // null terminate ^, image type 3 (front cover), null description
        vec.extend_from_slice(b"\x00\x03\x00");

        // push the content
        vec.extend_from_slice(&image);
    }
    // one extra padding byte, for luck
    vec.push(0x00);

    // we now have our final tag to write - let's maybe give it some padding
    let new_size = vec.len() + (vec.len() % 128);
    vec.resize(new_size, 0);

    // calculate final size
    let size = tools::encode_synch_int(vec.len() as u32 - 10, false)?;
    for i in 0..4 {
        vec[6 + i] = size[i];
    }

    // move to the start of music data in the input
    input.seek(SeekFrom::Start(old_size + 10))?;

    // start writing
    output.write(&vec)?;
    std::io::copy(input, output)?;
    output.flush()?;
    Ok(())
}

fn get_internal<T: Read + Seek>(input: &mut T, header: &structure::Header) -> Result<Tags, Error> {
    // with older id3 versions, run unsynch on the whole tag
    if header.is_unsynchronized && header.version < 4 {
        // read the tag into memory
        let mut vec = vec![0; header.size as usize];
        input.read(&mut vec)?;

        tools::undo_unsynch(&mut vec);

        // then use cursor to read and seek across the tags
        let mut unsynch_input = std::io::Cursor::new(vec);

        return get::tags(&mut unsynch_input, header);
    } else if header.is_unsynchronized && header.version == 4 {
        return get::tags_unsynch_v4(input, header);
    } else {
        return get::tags(input, header);
    }
}

#[cfg(test)]
mod tests;
