use std;

use std::io::prelude::*;

use crate::id3v2::tools::*;
use crate::id3v2::*;
use crate::tools::tag_error;

pub fn tags<T: Read + Seek>(input: &mut T, header: &structure::Header) -> Result<Tags, Error> {
    let mut t: Tags = Default::default();

    // handle getting date
    let mut date = "".to_string();
    let mut tdat = "".to_string();
    let mut time = "".to_string();

    let mut trck = "".to_string();
    let mut tpos = "".to_string();

    let mut found_best_cover = false;

    while input.seek(std::io::SeekFrom::Current(0))? <= (10 + header.size - 6) as u64 {
        // fail gracefully on invalid frames - we probably hit padding
        let f = match read::frame_header(input, header.version) {
            Ok(r) => r,
            Err(_) => break,
        };

        match f.name.as_str() {
            "TIT2" | "TT2" => t.title = TagOption::Some(read::string(input, f.size)),

            "TALB" | "TAL" => t.album = TagOption::Some(read::string(input, f.size)),
            "TPE1" | "TP1" => t.artist = TagOption::Some(read::string(input, f.size)),
            "TPE2" | "TP2" => t.album_artist = TagOption::Some(read::string(input, f.size)),
            "TCOM" | "TCM" => t.composer = TagOption::Some(read::string(input, f.size)),

            "TIT1" | "TT1" => t.grouping = TagOption::Some(read::string(input, f.size)),

            // give up on that ID3v1 compatibility
            "TCON" | "TCO" => t.genre = TagOption::Some(read::string(input, f.size)),

            // read in date for 2.4 or at least get the year
            "TDRC" | "TYER" | "TYE" => date = read::string(input, f.size),

            // read date for older to handle later on
            "TDAT" | "TDA" => tdat = read::string(input, f.size),
            "TIME" | "TIM" => time = read::string(input, f.size),

            "TRCK" | "TRK" => trck = read::string(input, f.size),
            "TPOS" | "TPA" => tpos = read::string(input, f.size),

            "TBPM" | "TBP" => {
                let s = read::string(input, f.size);
                if let Ok(i) = s.parse::<i64>() {
                    t.bpm = TagOption::Some(i);
                }
            }

            "TCMP" | "TCP" => {
                t.is_compilation = TagOption::Some(read::string(input, f.size).as_str() == "1")
            }

            "COMM" | "COM" => match read::comment(input, f.size) {
                Some(s) => t.comment = TagOption::Some(s),
                None => (),
            },

            "TSOT" | "TST" => t.sort_title = TagOption::Some(read::string(input, f.size)),
            "TSOA" | "TSA" => t.sort_album = TagOption::Some(read::string(input, f.size)),
            "TSOP" | "TSP" => t.sort_artist = TagOption::Some(read::string(input, f.size)),
            "TSO2" | "TS2" => t.sort_album_artist = TagOption::Some(read::string(input, f.size)),
            "TSOC" | "TSC" => t.sort_composer = TagOption::Some(read::string(input, f.size)),

            "APIC" => {
                let (img, apic_type) = read::image(input, f.size);
                if apic_type == 0x03 || !found_best_cover {
                    t.front_cover = TagOption::Some(img);
                    found_best_cover = apic_type == 0x03;
                }
            }
            "PIC" => {
                let (img, apic_type) = read::image_v2(input, f.size);
                if apic_type == 0x03 || !found_best_cover {
                    t.front_cover = TagOption::Some(img);
                    found_best_cover = apic_type == 0x03;
                }
            }

            // seek ahead if frame is not getting read in
            _ => {
                input.seek(std::io::SeekFrom::Current(f.size as i64))?;
            }
        }
    }
    // deal with dates on older id3 versions
    if header.version < 4 && tdat.len() == 4 {
        // TDAT is stored as DDMM
        date = format!("{}-{}-{}", date, &tdat.as_str()[2..4], &tdat.as_str()[0..2]);
        // only consider time if we have the month/day
        if time.len() == 4 {
            // TIME is stored as HHMM
            date = format!("{}T{}:{}", date, &time.as_str()[0..2], &time.as_str()[2..4]);
        }
    }

    // drop invalid dates
    t.date = DateTime::from_iso_8601(date.as_str()).into();

    // write track/disc
    let track = regex::get_track_number(&trck);
    if let Ok(x) = track.0.parse::<i64>() {
        t.track_number = TagOption::Some(x);
    }
    if let Ok(x) = track.1.parse::<i64>() {
        t.track_total = TagOption::Some(x);
    }

    let disc = regex::get_track_number(&tpos);
    if let Ok(x) = disc.0.parse::<i64>() {
        t.disc_number = TagOption::Some(x);
    }
    if let Ok(x) = disc.1.parse::<i64>() {
        t.disc_total = TagOption::Some(x);
    }

    Ok(t)
}

// use a separate function for unsynch v2.4 to avoid dynamic dispatch
pub fn tags_unsynch_v4<T: Read + Seek>(
    input: &mut T,
    header: &structure::Header,
) -> Result<Tags, Error> {
    // in case something that is never supposed to happen happens
    if header.version != 0x04 || !header.is_unsynchronized {
        return Err(tag_error(
            "Tried to parse a synchronized or a non-ID3v2.4 tag with tags_unsynch_v4",
        ));
    }

    let mut t: Tags = Default::default();

    // handle getting date
    let mut date = "".to_string();

    let mut trck = "".to_string();
    let mut tpos = "".to_string();

    let mut found_best_cover = false;

    while input.seek(std::io::SeekFrom::Current(0))? <= (10 + header.size - 6) as u64 {
        // fail gracefully on invalid frames - we probably hit padding
        let f = match read::frame_header(input, 0x04) {
            Ok(r) => r,
            Err(_) => break,
        };

        let u = f.is_unsynchronized;

        // lose non-v2.4 stuff
        match f.name.as_str() {
            "TIT2" => t.title = TagOption::Some(read_string_shim_v4(input, f.size, u)),

            "TALB" => t.album = TagOption::Some(read_string_shim_v4(input, f.size, u)),
            "TPE1" => t.artist = TagOption::Some(read_string_shim_v4(input, f.size, u)),
            "TPE2" => t.album_artist = TagOption::Some(read_string_shim_v4(input, f.size, u)),
            "TCOM" => t.composer = TagOption::Some(read_string_shim_v4(input, f.size, u)),

            "TIT1" => t.grouping = TagOption::Some(read_string_shim_v4(input, f.size, u)),

            // give up on that ID3v1 compatibility
            "TCON" => t.genre = TagOption::Some(read_string_shim_v4(input, f.size, u)),

            // read in date for 2.4 or at least get the year
            "TDRC" => date = read_string_shim_v4(input, f.size, u),

            "TRCK" => trck = read_string_shim_v4(input, f.size, u),
            "TPOS" => tpos = read_string_shim_v4(input, f.size, u),

            "TBPM" => {
                let s = read_string_shim_v4(input, f.size, u);
                if let Ok(i) = s.parse::<i64>() {
                    t.bpm = TagOption::Some(i);
                }
            }

            "TCMP" => {
                t.is_compilation =
                    TagOption::Some(read_string_shim_v4(input, f.size, u).as_str() == "1")
            }

            "COMM" => match u {
                false => match read::comment(input, f.size) {
                    Some(s) => t.comment = TagOption::Some(s),
                    None => (),
                },
                true => {
                    let mut vec = vec![0; f.size as usize - 4];

                    input.seek(std::io::SeekFrom::Current(4))?;
                    input.read(&mut vec)?;
                    undo_unsynch(&mut vec);

                    let l = vec.len() as u32;
                    match read::comment(&mut (std::io::Cursor::new(vec)), l) {
                        Some(s) => t.comment = TagOption::Some(s),
                        None => (),
                    }
                }
            },

            "TSOT" => t.sort_title = TagOption::Some(read_string_shim_v4(input, f.size, u)),
            "TSOA" => t.sort_album = TagOption::Some(read_string_shim_v4(input, f.size, u)),
            "TSOP" => t.sort_artist = TagOption::Some(read_string_shim_v4(input, f.size, u)),
            "TSO2" => t.sort_album_artist = TagOption::Some(read_string_shim_v4(input, f.size, u)),
            "TSOC" => t.sort_composer = TagOption::Some(read_string_shim_v4(input, f.size, u)),

            "APIC" => {
                let (img, apic_type) = match u {
                    false => read::image(input, f.size),
                    true => {
                        let mut vec = vec![0; f.size as usize - 4];

                        input.seek(std::io::SeekFrom::Current(4))?;
                        input.read(&mut vec)?;
                        undo_unsynch(&mut vec);

                        let l = vec.len() as u32;
                        read::image(&mut (std::io::Cursor::new(vec)), l)
                    }
                };

                if apic_type == 0x03 || !found_best_cover {
                    t.front_cover = TagOption::Some(img);
                    found_best_cover = apic_type == 0x03;
                }
            }

            // seek ahead if frame is not getting read in
            _ => {
                input.seek(std::io::SeekFrom::Current(f.size as i64))?;
            }
        }
    }

    // drop invalid dates
    t.date = DateTime::from_iso_8601(date.as_str()).into();

    // write track/disc
    let track = regex::get_track_number(&trck);
    if let Ok(x) = track.0.parse::<i64>() {
        t.track_number = TagOption::Some(x);
    }
    if let Ok(x) = track.1.parse::<i64>() {
        t.track_total = TagOption::Some(x);
    }

    let disc = regex::get_track_number(&tpos);
    if let Ok(x) = disc.0.parse::<i64>() {
        t.disc_number = TagOption::Some(x);
    }
    if let Ok(x) = disc.1.parse::<i64>() {
        t.disc_total = TagOption::Some(x);
    }

    Ok(t)
}

fn read_string_shim_v4<T: Read + Seek>(
    input: &mut T,
    length: u32,
    is_unsynchronized: bool,
) -> String {
    if is_unsynchronized {
        let mut vec = vec![0; length as usize - 4];
        // skip first four bytes - that's the expanded size
        match input.seek(std::io::SeekFrom::Current(4)) {
            Ok(_) => (),
            Err(_) => return "".to_string(),
        }
        match input.read(&mut vec) {
            Ok(_) => (),
            Err(_) => return "".to_string(),
        }
        undo_unsynch(&mut vec);

        let l = vec.len() as u32;
        read::string(&mut (std::io::Cursor::new(&vec)), l)
    } else {
        read::string(input, length)
    }
}
