use std;

use std::io::prelude::*;

use crate::id3v2::structure;
use crate::id3v2::tools::*;
use crate::tools::decode_int_be_u32;
use crate::Error;
use crate::Image;

use crate::tools::encoding::*;
use crate::tools::tag_error;

// allow some functions to fail silently here
// invalidate just a single frame instead of the whole tag
macro_rules! try_silent {
    ($statement:expr, $fail:expr) => {{
        match $statement {
            Ok(_) => (),
            Err(_) => return $fail,
        }
    }};
}

// this one tries to avoid loading images into memory if it can help it
pub fn image_v2<T: Read + Seek>(input: &mut T, length: u32) -> (Image, u8) {
    let length = length as usize;

    // no reasonable image is gonna be this small
    if length < 6 {
        try_silent!(
            input.seek(std::io::SeekFrom::Current(length as i64)),
            (Image::None, 0xFF)
        );
        return (Image::None, 0xFF);
    }

    // v2 largest size of ideal (no description) header:
    //     1 - encoding
    //     3 - MIME
    //     1 - picture type
    //     1 - 0x00
    // read in the first 14 bytes
    let mut header = [0; 6];
    try_silent!(input.read(&mut header), (Image::None, 0xFF));

    let t = header[4];
    if t != 0x00 && t != 0x03 {
        try_silent!(
            input.seek(std::io::SeekFrom::Current(length as i64 - 6)),
            (Image::None, 0xFF)
        );
        return (Image::None, 0xFF);
    }

    // check for headers with empty descriptions
    // we might be able to skip vector reallocations in that case
    match &header[1..] {
        b"JPG\x03\x00" | b"JPG\x00\x00" => {
            let mut vec = vec![0; length - 6];
            try_silent!(input.read(&mut vec), (Image::None, 0xFF));
            return (Image::JPEG(vec), t);
        }
        b"PNG\x03\x00" | b"PNG\x00\x00" => {
            let mut vec = vec![0; length - 6];
            try_silent!(input.read(&mut vec[1..]), (Image::None, 0xFF));
            return (Image::PNG(vec), t);
        }
        // if these pre-cooked headers didn't work, we'll have to try harder
        // and allocate more vectors in the end ;_;
        _ => {
            // make sure we have a JPEG or a PNG
            match &header[1..4] {
                b"PNG" | b"JPG" => {}
                _ => {
                    try_silent!(
                        input.seek(std::io::SeekFrom::Current(length as i64 - 6)),
                        (Image::None, 0xFF)
                    );
                    return (Image::None, 0xFF);
                }
            }

            // if we're here, we found a good cover image, so we should actually load it.
            let mut vec = vec![0; length - 5];
            vec[0] = header[5];

            try_silent!(input.read(&mut vec[1..]), (Image::None, 0xFF));

            // we can't just search for the next 0x00 because of UTF-16,
            // and we can't just rely on people to actually zero-terminate their strings,
            // so search for JPEG/PNG headers instead.

            for i in 1..vec.len() - 3 {
                if vec[i - 1] == 0x00 && vec[i] == 0xFF && vec[i + 1] == 0xD8 && vec[i + 2] == 0xFF
                {
                    return (Image::JPEG(vec.split_off(i as usize)), t);
                }
                if vec[i - 1] == 0x00 && vec[i] == 0x89 && vec[i + 1] == 0x50 && vec[i + 2] == 0x4E
                {
                    return (Image::PNG(vec.split_off(i as usize)), t);
                }
            }
        }
    };
    // if that evil failed, we did not find the header
    // so give up
    return (Image::None, 0x00);
}

// this one tries to avoid loading images into memory if it can help it
pub fn image<T: Read + Seek>(input: &mut T, length: u32) -> (Image, u8) {
    let length = length as usize;

    // no reasonable image is gonna be this small
    if length < 14 {
        try_silent!(
            input.seek(std::io::SeekFrom::Current(length as i64)),
            (Image::None, 0xFF)
        );
        return (Image::None, 0xFF);
    }

    // largest size of ideal (no description) header:
    //     1 - encoding
    //    10 - MIME
    //     1 - 0x00
    //     1 - picture type
    //     1 - 0x00
    // read in the first 14 bytes
    let mut header = [0; 14];
    try_silent!(input.read(&mut header), (Image::None, 0xFF));

    // check for headers with empty descriptions
    // we might be able to skip vector reallocations in that case
    match &header[1..] {
        b"image/jpeg\x00\x03\x00" | b"image/jpeg\x00\x00\x00" => {
            let mut vec = vec![0; length - 14];
            try_silent!(input.read(&mut vec), (Image::None, 0xFF));
            return (Image::JPEG(vec), header[12]);
        }
        b"image/jpg\x00\x03\x00\xFF" | b"image/jpg\x00\x00\x00\xFF" => {
            let mut vec = vec![0; length - 13];
            vec[0] = 0xFF; // first byte of jpeg bled into the header
            try_silent!(input.read(&mut vec[1..]), (Image::None, 0xFF));
            return (Image::PNG(vec), header[11]);
        }
        b"image/png\x00\x03\x00\x89" | b"image/png\x00\x00\x00\x89" => {
            let mut vec = vec![0; length - 13];
            vec[0] = 0x89; // first byte of png bled into the header
            try_silent!(input.read(&mut vec[1..]), (Image::None, 0xFF));
            return (Image::PNG(vec), header[11]);
        }
        // if these pre-cooked headers didn't work, we'll have to try harder
        // and allocate more vectors in the end ;_;
        _ => {
            // find where the first zero
            let zero1 = (&header[1..])
                .iter()
                .position(|&x| x == 0x00)
                .unwrap_or(header.len() - 1)
                + 1;

            // make sure we have a JPEG or a PNG
            match &header[1..zero1] {
                b"image/png" | b"image/jpeg" | b"image/jpg" | b"png" | b"jpeg" => {}
                _ => {
                    println!("go home");
                    try_silent!(
                        input.seek(std::io::SeekFrom::Current(length as i64 - 14)),
                        (Image::None, 0xFF)
                    );
                    return (Image::None, 0xFF);
                }
            }
            // implicit from above: 1 < zero1 <= 11

            let t = header[zero1 + 1];

            // only consider Cover (front) or Other
            if t != 0x03 && t != 0x00 {
                try_silent!(
                    input.seek(std::io::SeekFrom::Current(length as i64 - 14)),
                    (Image::None, 0xFF)
                );
                return (Image::None, 0xFF);
            }

            // if we're here, we found a good cover image, so we should actually load it.
            let mut vec = Vec::with_capacity(length);
            vec.extend_from_slice(&header); // fill first 14 bytes we already have
            vec.resize(length, 0); // fill the rest with zeroes

            try_silent!(input.read(&mut vec[14..]), (Image::None, 0xFF));

            // we can't just search for the next 0x00 because of UTF-16,
            // and we can't just rely on people to actually zero-terminate their strings,
            // so search for JPEG/PNG headers instead.

            for i in zero1..length - 4 {
                if vec[i - 1] == 0x00 && vec[i] == 0xFF && vec[i + 1] == 0xD8 && vec[i + 2] == 0xFF
                {
                    return (Image::JPEG(vec.split_off(i as usize)), t);
                }
                if vec[i - 1] == 0x00 && vec[i] == 0x89 && vec[i + 1] == 0x50 && vec[i + 2] == 0x4E
                {
                    return (Image::PNG(vec.split_off(i as usize)), t);
                }
            }
        }
    };
    // if that evil failed, we did not find the header
    // so give up
    return (Image::None, 0x00);
}

fn string_from_slice(arr: &[u8]) -> String {
    match arr[0] {
        0x00 => decode_iso_8859_1(&arr[1..]),
        0x01 | 0x02 => decode_utf16(&arr[1..]),
        0x03 | _ => decode_utf8(&arr[1..]),
    }
}
// only care about collecting user comments
pub fn comment<T: Read + Seek>(input: &mut T, length: u32) -> Option<String> {
    // too small to be a proper comment
    if length < 5 {
        try_silent!(input.seek(std::io::SeekFrom::Current(length as i64)), None);
        return None;
    }

    let mut header = [0; 5];
    if let Err(_) = input.read(&mut header) {
        return None;
    }

    // make sure this is just a normal comment
    if header[4] != 0x00 {
        try_silent!(
            input.seek(std::io::SeekFrom::Current(length as i64 - 5)),
            None
        );
        return None;
    }

    let mut vec = vec![0; length as usize - 5 + 1];
    if let Err(_) = input.read(&mut vec[1..]) {
        return None;
    }
    vec[0] = header[0];
    Some(string_from_slice(&vec).replace("\0", " / "))
}

// return empty string on fail here
pub fn string<T: Read + Seek>(input: &mut T, length: u32) -> String {
    let mut vec = vec![0; length as usize];
    if let Err(_) = input.read(&mut vec) {
        return "".to_string();
    }
    string_from_slice(&vec).replace("\0", " / ")
}

pub fn frame_header<T: Read + Seek>(
    input: &mut T,
    version: u8,
) -> Result<structure::FrameHeader, Error> {
    // first, deal with id3v2.2
    if version == 2 {
        let mut arr: [u8; 3 + 3] = [0; 6];
        input.read(&mut arr)?;
        return Ok(structure::FrameHeader {
            name: decode_frame_id(&arr[0..3])?,
            size: decode_int_be_u32(&arr[3..6]),
            ..Default::default()
        });
    }

    // 4: Frame ID      $xx xx xx xx  (four characters)
    // 4: Size      4 * %0xxxxxxx in 2.4 / $xx in 2.3
    // 2: Flags         $xx xx
    let mut arr: [u8; 4 + 4 + 2] = [0; 10];
    input.read(&mut arr)?;

    let size = match version {
        3 => decode_int_be_u32(&arr[4..8]),
        4 => decode_synch_int(&arr[4..8])?,
        _ => return Err(tag_error("Unknown ID3 version")),
    };

    // TODO: Extended header bytes do exist
    let flags1 = arr[8];
    let flags2 = arr[9];

    Ok(structure::FrameHeader {
        name: decode_frame_id(&arr[0..4])?,
        size: size,

        drop_after_tag_alteration: flags1 & 0b01000000 != 0,
        drop_after_file_alteration: flags1 & 0b00100000 != 0,

        is_unsynchronized: flags2 & 0b00000010 != 0,
        is_compressed: flags2 & 0b00001000 != 0,
    })
}

pub fn header<T: Read + Seek>(input: &mut T) -> Result<structure::Header, Error> {
    input.seek(std::io::SeekFrom::Start(0))?;

    let mut arr: [u8; 10] = [0; 10];
    input.read(&mut arr)?;

    // ID3v2/input identifier      "ID3"
    if &arr[0..3] != b"ID3" {
        return Err(tag_error("ID3v2 header not found"));
    }

    let mut header = structure::Header {
        version: arr[3],
        ..Default::default()
    };

    // ID3v2 version              $0X 00
    if header.version == 0xFF {
        return Err(tag_error("Invalid ID3v2 version"));
    }

    if header.version > 4 || header.version < 2 {
        return Err(tag_error(&format!(
            "ID3v2.{} is not supported",
            header.version
        )));
    }

    // ID3v2 flags                %abcd0000
    let flags = arr[5];

    header.is_unsynchronized = flags & 0b10000000 != 0;
    let has_extended_header = flags & 0b01000000 != 0;
    header.is_experimental = flags & 0b00100000 != 0;
    header.has_footer = flags & 0b00010000 != 0;

    if flags & 0x0F != 0 {
        return Err(tag_error("Unsupported flags found in ID3 header"));
    }

    // TODO: Find examples of MP3s with extended headers to add support for this
    if has_extended_header {
        return Err(tag_error(
            "Extended ID3 headers are currently not supported",
        ));
    }

    header.size = decode_synch_int(&arr[6..10])?;

    Ok(header)
}
