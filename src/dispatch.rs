use std;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::tools::tag_error;
use crate::Error;
use crate::Image;
use crate::Tags;

use crate::flac;
use crate::id3v1;
use crate::id3v2;
use crate::m4a;

macro_rules! unsupported {
    ($path:ident) => {{
        match $path.extension().unwrap_or_default().to_str() {
            Some("mp3") | Some("flac") | Some("m4a") => (),
            None | Some(_) => return Err(tag_error("Unsupported file format")),
        }
    }};
}

pub fn get_tags<P: AsRef<Path>>(path: P) -> Result<Tags, Error> {
    let path = path.as_ref();
    unsupported!(path);

    let mut file = File::open(path)?;

    match path.extension().unwrap_or_default().to_str() {
        Some("mp3") => {
            // try id3v2 first; try falling back on id3v1
            match id3v2::get(&mut file) {
                Ok(t) => Ok(t),
                Err(Error::IOError(x)) => Err(Error::IOError(x)),
                Err(Error::TagError(_)) => id3v1::get(&mut file),
            }
        }
        Some("flac") => flac::get(&mut file),
        Some("m4a") => m4a::get(&mut file),
        None | Some(_) => Err(tag_error("Unsupported file format")),
    }
}

pub fn get_front_cover<P: AsRef<Path>>(path: P) -> Result<Image, Error> {
    let tags = get_tags(path)?;
    match tags.front_cover {
        crate::TagOption::Some(img) => Ok(img),
        _ => Ok(Image::None),
    }
}

pub fn set_tags<P: AsRef<Path>>(path: P, tags: &Tags) -> Result<(), Error> {
    // check path validity
    let path = path.as_ref();
    unsupported!(path);

    // whyyyy
    let tmp_path = {
        let mut p = path.to_path_buf();
        let mut e = std::ffi::OsString::from(p.extension().unwrap_or_default());
        e.push("tmp");
        p.set_extension(e);
        p
    };

    {
        let mut file = File::open(path)?;
        // create temporary file to write to
        let mut tmp_file = BufWriter::new(
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&tmp_path)?,
        );

        match path.extension().unwrap_or_default().to_str() {
            Some("mp3") => {
                // write id3v2
                if let Err(x) = id3v2::set(&mut file, &mut tmp_file, tags) {
                    // on failure, delete temporary file
                    std::fs::remove_file(tmp_path)?;
                    return Err(x);
                }

                // get back the file from buffered writer
                let mut tmp_file = match tmp_file.into_inner() {
                    Ok(tmp_file) => tmp_file,
                    Err(x) => {
                        std::fs::remove_file(tmp_path)?;
                        return Err(Error::TagError(format!(
                            "Could not unwrap buffered writer: {:?}",
                            x
                        )));
                    }
                };

                if id3v1::has_id3v1(&mut tmp_file) {
                    if let Ok(x) = tmp_file.metadata() {
                        tmp_file.set_len(x.len() - 128).ok();
                    }
                }
            }

            Some("flac") => {
                if let Err(x) = flac::set(&mut file, &mut tmp_file, tags) {
                    // on failure, delete temporary file
                    std::fs::remove_file(tmp_path)?;
                    return Err(x);
                }
            }
            Some("m4a") => {
                if let Err(x) = m4a::set(&mut file, &mut tmp_file, tags) {
                    // on failure, delete temporary file
                    std::fs::remove_file(tmp_path)?;
                    return Err(x);
                }
            }
            None | Some(_) => {
                std::fs::remove_file(tmp_path)?;
                return Err(tag_error("Unsupported file format"));
            }
        }
    }

    // replace original file
    std::fs::rename(tmp_path, path)?;
    Ok(())
}
