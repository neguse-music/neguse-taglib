use std;

use std::io::prelude::*;

use crate::DateTime;
use crate::Image;
use crate::TagOption;
use crate::Tags;

use crate::Error;

use crate::tools::decode_int_be_u32;
use crate::tools::encode_int_be_u16;
use crate::tools::encode_int_be_u32;
use crate::tools::encoding::decode_iso_8859_1;
use crate::tools::encoding::encode_iso_8859_1;
use crate::tools::tag_error;
use crate::tools::tags;

#[allow(dead_code)]
struct Atom {
    name: String,
    size: u64,
    start: u64,
    end: u64,
}
impl Atom {
    fn new(input: &[u8], start: u64) -> Atom {
        let size = decode_int_be_u32(&input[0..4]) as u64;
        Atom {
            name: decode_iso_8859_1(&input[4..8]),
            size: size,
            start: start,
            end: start + size,
        }
    }
    #[allow(dead_code)]
    fn print(&self) {
        println!(
            "{}: {} ({}..{})",
            self.name, self.size, self.start, self.end
        );
    }
}

pub fn get<T: Read + Seek>(input: &mut T) -> Result<Tags, Error> {
    let mut buf: [u8; 8] = [0; 8];

    let mut pos = 0;

    let ilst;

    // locate moov.udta.meta.ilst
    loop {
        if input.read(&mut buf)? == 0 {
            // if we hit EOF without locating the tags atom
            return Ok(Tags::none());
        }
        let atom = Atom::new(&buf, pos);
        pos += buf.len() as u64;

        if atom.size < 8 && atom.size > 1 {
            return Err(tag_error("Found invalid atom while parsing m4a structure"));
        }

        match atom.name.as_str() {
            "moov" | "udta" => (),
            // skip the four bytes at the start of meta
            "meta" => pos = input.seek(std::io::SeekFrom::Current(4))?,
            "ilst" => {
                ilst = atom;
                break;
            }
            _ => pos = input.seek(std::io::SeekFrom::Current(atom.size as i64 - 8))?,
        }
    }

    collect_tags(input, &mut pos, &ilst)
}

macro_rules! write_data {
    ($vec:ident, $data:expr, $name:expr, $flags:expr) => {{
        // atom header
        $vec.extend_from_slice(&encode_int_be_u32(
            $data.len() as u32 + $flags.len() as u32 + 16,
        ));
        $vec.extend_from_slice(&encode_iso_8859_1($name));

        // data atom header
        $vec.extend_from_slice(&encode_int_be_u32(
            $data.len() as u32 + $flags.len() as u32 + 8,
        ));
        $vec.extend_from_slice(&encode_iso_8859_1("data"));

        // flags
        $vec.extend_from_slice($flags);

        // data
        $vec.extend_from_slice($data);
    }};
}
// only write if the comment is not empty
macro_rules! write_text {
    ($vec:ident, $text:expr, $name:expr) => {{
        if let TagOption::Some(ref t) = $text {
            if t.as_str() != "" {
                write_data!(
                    $vec,
                    t.as_str().as_bytes(),
                    $name,
                    &[0, 0, 0, 1, 0, 0, 0, 0]
                );
            }
        }
    }};
}

macro_rules! tag_to_u16_encoded {
    ($tag:expr) => {{
        encode_int_be_u16(*$tag.as_ref().unwrap_or(&0) as u16)
    }};
}

macro_rules! overwrite_u32 {
    ($src:expr, $dest:expr) => {{
        for i in 0..4 {
            $dest[i] = $src[i];
        }
    }};
}

pub fn set<R: Read + Seek, W: Write>(
    input: &mut R,
    output: &mut W,
    new: &Tags,
) -> Result<(), Error> {
    let mut buf: [u8; 8] = [0; 8];

    // construct new file in memory
    let mut vec: Vec<u8> = Vec::new();

    let mut pos = 0;

    // to adjust sizes
    let mut moov_offset = 0;
    let mut udta_offset = 0;
    let mut meta_offset = 0;

    // to fix media data offsets
    let mut stco_offset = 0;

    // locate moov.udta.meta.ilst
    // and moov.trak.mdia.minf.stbl.stco
    // at the same time, copy all other data
    loop {
        input.read(&mut buf)?;

        let atom = Atom::new(&buf, pos);
        pos += buf.len() as u64;

        if atom.size < 8 && atom.size > 1 {
            return Err(tag_error("Found invalid atom while parsing m4a structure"));
        }

        match atom.name.as_str() {
            "moov" => {
                // remember location of moov atom start
                moov_offset = vec.len();
                // and add the atom marker
                vec.extend_from_slice(&buf);
            }

            // stco route - write the atom start, but then traverse the atom
            "trak" | "mdia" | "minf" | "stbl" => {
                vec.extend_from_slice(&buf);
            }
            // finally, stco itself
            "stco" => {
                stco_offset = vec.len();
                vec.extend_from_slice(&buf);
                let s = vec.len();
                vec.resize(s + atom.size as usize - 8, 0);
                pos += input.read(&mut vec[s..])? as u64;
            }

            // conclusion
            "mdat" => {
                // if we hit mdat and we have no stco, don't write
                if stco_offset == 0 {
                    return Err(tag_error("Could not find stco atom"));
                }
                let mdat_original = input.seek(std::io::SeekFrom::Current(0))? as i64;
                let mdat_offset = vec.len() as i64;
                let mdat_delta = (mdat_offset - mdat_original) as i32;

                // we're sitting on top of the data.
                // finalize the new structure and write

                // fix stco with the mdat offset we calculated
                let stco_count = decode_int_be_u32(&vec[stco_offset + 12..stco_offset + 12 + 4]);
                for i in 0..stco_count {
                    let offset = stco_offset + 12 + 4 + (4 * i) as usize;
                    overwrite_u32!(
                        &encode_int_be_u32(
                            (decode_int_be_u32(&vec[offset..offset + 4]) as i64 + mdat_delta as i64)
                                as u32
                        ),
                        &mut vec[offset..offset + 4]
                    );
                }
                // write the thing
                output.write(&vec)?;
                std::io::copy(input, output)?;
                break;
            }

            // ilst route
            "udta" => {
                // etc for all ancestors of ilst
                udta_offset = vec.len();
                vec.extend_from_slice(&buf);
            }
            "meta" => {
                meta_offset = vec.len();
                vec.extend_from_slice(&buf);
                vec.extend_from_slice(&[0, 0, 0, 0]);
                pos = input.seek(std::io::SeekFrom::Current(4))?;
            }
            "ilst" => {
                let ilst_offset = vec.len();

                vec.extend_from_slice(&buf);
                let ilst_size_orig = decode_int_be_u32(&buf[0..4]);

                let tags = tags::delta(&collect_tags(input, &mut pos, &atom)?, new);

                // write tags
                write_text!(vec, tags.title, "©nam");
                write_text!(vec, tags.album, "©alb");
                write_text!(vec, tags.artist, "©art");
                write_text!(vec, tags.album_artist, "aART");
                write_text!(vec, tags.composer, "©wrt");
                write_text!(vec, tags.grouping, "©grp");
                write_text!(vec, tags.genre, "©gen");
                if let TagOption::Some(ref t) = tags.date {
                    let text = t.to_iso_8601();
                    if text.as_str() != "" {
                        write_data!(
                            vec,
                            text.as_str().as_bytes(),
                            "©day",
                            &[0, 0, 0, 1, 0, 0, 0, 0]
                        );
                    }
                }

                // track + track total
                vec.extend_from_slice(b"\x00\x00\x00\x1Etrkn");
                vec.extend_from_slice(b"\x00\x00\x00\x16data");
                vec.extend_from_slice(b"\x00\x00\x00\x00\x00\x00\x00\x00");
                vec.extend_from_slice(b"\x00\x00");
                vec.extend_from_slice(&tag_to_u16_encoded!(tags.track_number));
                vec.extend_from_slice(&tag_to_u16_encoded!(tags.track_total));

                // disc + disc total
                vec.extend_from_slice(b"\x00\x00\x00\x1Edisk");
                vec.extend_from_slice(b"\x00\x00\x00\x16data");
                vec.extend_from_slice(b"\x00\x00\x00\x00\x00\x00\x00\x00");
                vec.extend_from_slice(b"\x00\x00");
                vec.extend_from_slice(&tag_to_u16_encoded!(tags.disc_number));
                vec.extend_from_slice(&tag_to_u16_encoded!(tags.disc_total));

                write_data!(
                    vec,
                    &tag_to_u16_encoded!(tags.bpm),
                    "tmpo",
                    b"\x00\x00\x00\x15\x00\x00\x00\x00"
                );

                if let TagOption::Some(true) = tags.is_compilation {
                    write_data!(vec, &[0x01], "cpil", b"\x00\x00\x00\x15\x00\x00\x00\x00");
                }

                write_text!(vec, tags.comment, "©cmt");
                write_text!(vec, tags.sort_title, "sonm");
                write_text!(vec, tags.sort_album, "soal");
                write_text!(vec, tags.sort_artist, "soar");
                write_text!(vec, tags.sort_album_artist, "soaa");
                write_text!(vec, tags.sort_composer, "soco");

                // if we have image
                if let TagOption::Some(ref i) = tags.front_cover {
                    if i.is_some() {
                        write_data!(
                            vec,
                            &match i {
                                Image::PNG(ref x) | Image::JPEG(ref x) => x,
                                _ => return Err(tag_error("Expected an image")),
                            },
                            "covr",
                            // flag jpeg or png
                            &[
                                0,
                                0,
                                0,
                                match i {
                                    Image::JPEG(_) => 13,
                                    Image::PNG(_) => 14,
                                    _ => 0,
                                },
                                0,
                                0,
                                0,
                                0
                            ]
                        );
                    }
                }

                let ilst_delta =
                    ((vec.len() as i64 - ilst_offset as i64) - ilst_size_orig as i64) as i32;

                overwrite_u32!(
                    &encode_int_be_u32(
                        (decode_int_be_u32(&vec[moov_offset..moov_offset + 4]) as i64
                            + ilst_delta as i64) as u32
                    ),
                    &mut vec[moov_offset..moov_offset + 4]
                );
                overwrite_u32!(
                    &encode_int_be_u32(
                        (decode_int_be_u32(&vec[udta_offset..udta_offset + 4]) as i64
                            + ilst_delta as i64) as u32
                    ),
                    &mut vec[udta_offset..udta_offset + 4]
                );
                overwrite_u32!(
                    &encode_int_be_u32(
                        (decode_int_be_u32(&vec[meta_offset..meta_offset + 4]) as i64
                            + ilst_delta as i64) as u32
                    ),
                    &mut vec[meta_offset..meta_offset + 4]
                );
                overwrite_u32!(
                    &encode_int_be_u32(
                        (decode_int_be_u32(&vec[ilst_offset..ilst_offset + 4]) as i64
                            + ilst_delta as i64) as u32
                    ),
                    &mut vec[ilst_offset..ilst_offset + 4]
                );
            }
            // copy unknown atoms
            _ => {
                // read in the atom
                vec.extend_from_slice(&buf);
                let s = vec.len();
                vec.resize(s + atom.size as usize - 8, 0);
                pos += input.read(&mut vec[s..])? as u64;
            }
        }
    }

    Ok(())
}

macro_rules! if_let_text {
    ($tag:expr, $input:ident, $pos: ident) => {{
        if let Ok(x) = collect_atom_text($input, $pos) {
            $tag = TagOption::Some(x);
        }
    }};
}
fn collect_tags<T: Read + Seek>(input: &mut T, pos: &mut u64, ilst: &Atom) -> Result<Tags, Error> {
    let mut buf: [u8; 8] = [0; 8];

    let mut tags = Tags::none();

    // explore ilst
    while *pos < ilst.end {
        if input.read(&mut buf)? < buf.len() {
            break;
        }

        *pos += buf.len() as u64;

        let atom = Atom::new(&buf, *pos);
        if atom.size == 0 {
            break;
        }

        match atom.name.as_str() {
            "©nam" => if_let_text!(tags.title, input, pos),
            "©alb" => if_let_text!(tags.album, input, pos),
            "©art" | "©ART" => if_let_text!(tags.artist, input, pos),
            "aART" => if_let_text!(tags.album_artist, input, pos),
            "©wrt" => if_let_text!(tags.composer, input, pos),

            "©grp" => if_let_text!(tags.grouping, input, pos),

            "©gen" => if_let_text!(tags.genre, input, pos),
            "gnre" => {
                if tags.genre == TagOption::None {
                    if let Ok(i) = collect_atom_num(input, pos) {
                        use crate::id3v1::get_genre;
                        tags.genre = TagOption::Some(get_genre(i as u8 - 1));
                    }
                }
            }
            "©day" => {
                if let Ok(date) = collect_atom_text(input, pos) {
                    tags.date = DateTime::from_iso_8601(date.as_str()).into();
                }
            }

            "trkn" => {
                if let Ok(num) = collect_atom_track(input, pos) {
                    tags.track_number = match num.0 {
                        0 => TagOption::None,
                        _ => TagOption::Some(num.0 as i64),
                    };
                    tags.track_total = match num.1 {
                        0 => TagOption::None,
                        _ => TagOption::Some(num.1 as i64),
                    };
                }
            }
            "disk" => {
                if let Ok(num) = collect_atom_track(input, pos) {
                    tags.disc_number = match num.0 {
                        0 => TagOption::None,
                        _ => TagOption::Some(num.0 as i64),
                    };
                    tags.disc_total = match num.1 {
                        0 => TagOption::None,
                        _ => TagOption::Some(num.1 as i64),
                    };
                }
            }

            // if the cpil data is 1, this is a compilation
            "cpil" => {
                if let Ok(i) = collect_atom_num(input, pos) {
                    if i == 1 {
                        tags.is_compilation = TagOption::Some(true);
                    }
                }
            }

            "tmpo" => {
                if let Ok(num) = collect_atom_num(input, pos) {
                    if num != 0 {
                        tags.bpm = TagOption::Some(num as i64)
                    }
                }
            }

            "©cmt" => if_let_text!(tags.comment, input, pos),

            "sonm" => if_let_text!(tags.sort_title, input, pos),
            "soal" => if_let_text!(tags.sort_album, input, pos),
            "soar" => if_let_text!(tags.sort_artist, input, pos),
            "soaa" => if_let_text!(tags.sort_album_artist, input, pos),
            "soco" => if_let_text!(tags.sort_composer, input, pos),

            // collect the first image only
            "covr" => {
                if tags.front_cover == TagOption::None {
                    // skip data header
                    *pos = input.seek(std::io::SeekFrom::Current(8))?;

                    // collect flags
                    input.read(&mut buf)?;
                    *pos += buf.len() as u64;

                    // collect image
                    let mut vec = vec![0; atom.size as usize - 24];
                    input.read(&mut vec)?;
                    *pos += vec.len() as u64;

                    match buf[3] {
                        13 => tags.front_cover = TagOption::Some(Image::JPEG(vec)),
                        14 => tags.front_cover = TagOption::Some(Image::PNG(vec)),
                        _ => (), // unknown image type
                    }
                } else {
                    *pos = input.seek(std::io::SeekFrom::Current(atom.size as i64 - 8))?;
                }
            }

            // skip secret itunes metadata
            // maybe collect it later
            "----" => *pos = input.seek(std::io::SeekFrom::Current(atom.size as i64 - 8))?,
            // skip unknown atoms
            _ => *pos = input.seek(std::io::SeekFrom::Current(atom.size as i64 - 8))?,
        }
    }

    Ok(tags)
}

fn collect_atom_text<T: Read + Seek>(input: &mut T, pos: &mut u64) -> Result<String, Error> {
    use crate::tools::encoding::decode_utf8;
    let vec = collect_atom_data(input, pos)?;

    if vec.len() < 9 {
        return Err(Error::TagError(format!(
            "Data too short in atom ending at {}",
            *pos
        )));
    }
    // check that we have text data
    if vec[3] == 1 {
        Ok(decode_utf8(&vec[8..])) // after flags
    } else {
        Err(Error::TagError(format!(
            "Expected text data in atom ending at {}",
            *pos
        )))
    }
}

fn collect_atom_num<T: Read + Seek>(input: &mut T, pos: &mut u64) -> Result<u32, Error> {
    let vec = collect_atom_data(input, pos)?;
    if vec.len() < 9 {
        return Err(Error::TagError(format!(
            "Data too short in atom ending at {}",
            *pos
        )));
    }

    match vec[3] {
        0 | 15 | 21 => {
            if vec.len() <= 12 {
                Ok(decode_int_be_u32(&vec[8..]))
            } else {
                Err(Error::TagError(format!(
                    "Expected a smaller number at {}",
                    *pos
                )))
            }
        }
        _ => Err(Error::TagError(format!(
            "Expected a single integer in atom ending at {}",
            *pos
        ))),
    }
}

fn collect_atom_track<T: Read + Seek>(input: &mut T, pos: &mut u64) -> Result<(u8, u8), Error> {
    let vec = collect_atom_data(input, pos)?;
    if vec.len() < 13 {
        return Err(Error::TagError(format!(
            "Data too short in atom ending at {}",
            *pos
        )));
    }

    match vec[3] {
        0 | 21 => Ok((vec[11], vec[13])),
        _ => Err(Error::TagError(format!(
            "Expected a single integer in atom ending at {}",
            *pos
        ))),
    }
}

fn collect_atom_data<T: Read + Seek>(input: &mut T, pos: &mut u64) -> Result<Vec<u8>, Error> {
    // get data atom stats
    let mut buf: [u8; 8] = [0; 8];
    input.read(&mut buf)?;
    let atom = Atom::new(&buf, *pos);
    *pos += buf.len() as u64;

    if atom.name.as_str() != "data" {
        return Err(Error::TagError(format!(
            "Expected data atom at position {}; found {}",
            pos, atom.name
        )));
    }

    // collect the data
    let mut vec = vec![0; atom.size as usize - 8];
    input.read(&mut vec)?;
    *pos += vec.len() as u64;
    Ok(vec)
}

#[cfg(test)]
mod tests;
