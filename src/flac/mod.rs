use std;
use std::io::prelude::*;

use crate::Error;
use crate::Image;
use crate::TagOption;
use crate::Tags;

use crate::vorbis;

use crate::tools::decode_int_be_u32;
use crate::tools::encode_int_be_u32;
use crate::tools::tag_error;
use crate::tools::tags;

pub fn get<T: Read + Seek>(input: &mut T) -> Result<Tags, Error> {
    let mut buf: [u8; 4] = [0; 4];

    input.read(&mut buf)?;

    if &buf != b"fLaC" {
        return Err(tag_error(
            "FLAC stream marker not found (is this a valid FLAC file?)",
        ));
    }

    let mut image: Option<Image> = None;
    let mut found_best_cover = false;

    let mut tags = Tags::none();

    while input.read(&mut buf)? >= buf.len() {
        // considering that on failure to fill it,
        // we would return with an error
        // should i still zero out the buffer?
        let len = decode_int_be_u32(&buf[1..]);

        match buf[0] & 0b01111111 {
            4 => {
                // vorbis comment
                tags = vorbis::get_tags(input)?;
            }
            6 => {
                // picture
                let (img, apic_type) = read_image(input, len)?;

                if apic_type == 0x03 || !found_best_cover {
                    image = Some(img);
                    found_best_cover = apic_type == 0x03;
                }
            }
            127 => {
                return Err(tag_error(
                    "Invalid block type (127) detected when parsing FLAC metadata",
                ))
            }
            _ => {
                input.seek(std::io::SeekFrom::Current(len as i64))?;
            }
        }

        if buf[0] & 0b10000000 != 0 {
            break;
        }
    }

    // if we found an image metadata block, apply it here
    if let Some(x) = image {
        tags.front_cover = TagOption::Some(x);
    }

    Ok(tags)
}

// same loop as get mostly, so that we can do it all in one loop
pub fn set<R: Read + Seek, W: Write>(
    input: &mut R,
    output: &mut W,
    new: &Tags,
) -> Result<(), Error> {
    let mut buf: [u8; 4] = [0; 4];

    input.read(&mut buf)?;

    if &buf != b"fLaC" {
        return Err(tag_error(
            "FLAC stream marker not found (is this a valid FLAC file?)",
        ));
    }

    // write header
    output.write(&buf)?;

    let mut image: Option<Image> = None;
    let mut found_best_cover = false;

    let mut old = Tags::none();

    loop {
        // considering that on failure to fill it,
        // we would return with an error
        // should i still zero out the buffer?
        input.read(&mut buf)?;
        let len = decode_int_be_u32(&buf[1..]);

        match buf[0] & 0b01111111 {
            4 => {
                // vorbis comment
                old = vorbis::get_tags(input)?;
            }
            6 => {
                // picture
                let (img, apic_type) = read_image(input, len)?;

                if apic_type == 0x03 || !found_best_cover {
                    image = Some(img);
                    found_best_cover = apic_type == 0x03;
                }
            }
            127 => {
                return Err(tag_error(
                    "Invalid block type (127) detected when parsing FLAC metadata",
                ))
            }
            1 => {
                input.seek(std::io::SeekFrom::Current(len as i64))?;
            }

            // copy write all non-tag blocks
            _ => {
                // this should be possible with take,
                // but we don't own the reader
                let mut vec = Vec::with_capacity(4 + len as usize);
                vec.extend_from_slice(&buf);
                vec.resize(4 + len as usize, 0);
                input.read(&mut vec[4..])?;
                // last metadata block will be our content
                // so unset the last metadata block flag
                vec[0] &= 0b11111111;
                // copy the block
                output.write(&vec)?;
            }
        }

        if buf[0] & 0b10000000 != 0 {
            break;
        }
    }

    // if we found an image metadata block, apply it here
    if let Some(x) = image {
        old.front_cover = TagOption::Some(x);
    }

    let tags = tags::delta(&old, new); // obtain the final tags

    // get the vorbis comment
    let vc = vorbis::from_tags(&tags, false);

    if vc.len() > 0x00FFFFFF {
        return Err(tag_error(
            "Could not write FLAC comment as it is larger than 16,777,215 bytes",
        ));
    }

    let mut vc_header = encode_int_be_u32(vc.len() as u32);

    vc_header[0] = 0x04; // last comment header

    output.write(&vc_header)?;
    output.write(&vc)?;

    if let TagOption::Some(ref x) = tags.front_cover {
        let img_block = get_picture_block(x)?;

        if img_block.len() > 0x00FFFFFF {
            return Err(tag_error(
                "Could not write FLAC picture block as it is larger than 16,777,215 bytes",
            ));
        }

        let mut img_block_header = encode_int_be_u32(img_block.len() as u32);
        img_block_header[0] = 0x06; // last block

        output.write(&img_block_header)?;
        output.write(&img_block)?;
    }

    // minimal padding block
    output.write(&[0b10000001, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00])?;

    // now copy the rest of the file
    std::io::copy(input, output)?;
    output.flush()?;

    Ok(())
}

macro_rules! write_u32 {
    ($vec:ident, $num:expr) => {{
        $vec.extend_from_slice(&encode_int_be_u32($num));
    }};
}
macro_rules! write_string {
    ($vec:ident, $str:expr) => {{
        write_u32!($vec, $str.len() as u32);
        $vec.extend_from_slice($str.as_bytes());
    }};
}
pub fn get_picture_block(image: &Image) -> Result<Vec<u8>, String> {
    if let Image::None = image {
        return Err("Not an image".to_string());
    }

    let img_vec = match image {
        Image::PNG(ref v) | Image::JPEG(ref v) => v,
        Image::None => return Err("Not an image".to_string()),
    };

    let mut vec = Vec::with_capacity(
        4 + // apic type
        4 + // mime length
        image.mime().len() +
        4 + // description length (empty)
        4 + // width
        4 + // height
        4 + // bpp
        4 + // index color
        4 + // image length
        img_vec.len(),
    );

    // image type
    write_u32!(vec, 0x03);

    // mime
    write_string!(vec, image.mime());

    // empty description
    write_u32!(vec, 0);

    // image dimensions
    // we checked for Image::None earlier
    let (w, h, bpp) = match image.dimensions() {
        Some(x) => x,
        None => return Err("Could not determine image dimensions to generate the FLAC picture block (is the image a valid JPG/PNG file?)".to_string()),
    };
    write_u32!(vec, w);
    write_u32!(vec, h);
    write_u32!(vec, bpp as u32);

    // empty description
    write_u32!(vec, 0);

    write_u32!(vec, img_vec.len() as u32);
    vec.extend_from_slice(img_vec);

    Ok(vec)
}

macro_rules! skip {
    ($input:ident, $length:ident, $bytes:ident) => {{
        $input.seek(std::io::SeekFrom::Current($length as i64 - $bytes))?;
    }};
}

pub fn read_image<T: Read + Seek>(input: &mut T, length: u32) -> Result<(Image, u32), Error> {
    // 4 - APIC type
    let mut bytes_read = 0;
    let mut buf: [u8; 4] = [0; 4];
    input.read(&mut buf)?;
    bytes_read += 4;

    let apic_type = decode_int_be_u32(&buf);

    // only collect front cover and other
    if apic_type != 0x00 && apic_type != 0x03 {
        skip!(input, length, bytes_read);
        return Ok((Image::None, 0xFF));
    }

    // 4 - MIME length
    // ^ - MIME
    input.read(&mut buf)?;
    bytes_read += 4;
    let mime_length = decode_int_be_u32(&buf);
    let mime = read_string(input, mime_length)?.to_lowercase();
    bytes_read += mime_length as i64;
    // ignore non-jpeg/png
    match mime.as_str() {
        "image/jpeg" | "image/jpg" | "jpg" | "jpeg" | "image/png" | "png" => (),
        _ => {
            skip!(input, length, bytes_read);
            return Ok((Image::None, 0xFF));
        }
    }

    // 4 - description length
    // ^ - description
    input.read(&mut buf)?;
    // bytes_read += 4;
    let description_length = decode_int_be_u32(&buf);
    // skip description
    input.seek(std::io::SeekFrom::Current(description_length as i64))?;
    // bytes_read += description_length as i64;

    // 4 - width
    // 4 - height
    // 4 - bpp
    // 4 - number of colors in indexed color (0 for jpg/png)
    // skip all that
    input.seek(std::io::SeekFrom::Current(16))?;
    // bytes_read += 16;
    // 4 - length of picture data
    input.read(&mut buf)?;
    // bytes_read += 4;
    let len = decode_int_be_u32(&buf) as usize;
    let mut vec: Vec<u8> = vec![9; len];
    input.read(&mut vec)?;

    Ok((
        match mime.as_str() {
            "image/jpeg" | "image/jpg" | "jpg" | "jpeg" => Image::JPEG(vec),
            "image/png" | "png" => Image::PNG(vec),
            _ => return Err(tag_error("Non-JPEG/PNG image files are not supported")),
        },
        apic_type,
    ))
}

pub fn read_string<T: Read + Seek>(input: &mut T, length: u32) -> Result<String, Error> {
    use crate::tools::encoding::decode_utf8;
    let mut vec = vec![0; length as usize];
    input.read(&mut vec)?;
    Ok(decode_utf8(&vec))
}

#[cfg(test)]
mod tests;
