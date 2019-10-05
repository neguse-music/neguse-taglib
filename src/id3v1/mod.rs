use std;
use std::io::prelude::*;

use crate::tools::encoding::decode_iso_8859_1;
use crate::tools::tag_error;
use crate::DateTime;
use crate::Error;
use crate::TagOption;
use crate::Tags;

pub fn get_genre(g: u8) -> String {
    if g >= 80 {
        return "".to_string();
    } else {
        return [
            "Blues",
            "Classic Rock",
            "Country",
            "Dance",
            "Disco",
            "Funk",
            "Grunge",
            "Hip-Hop",
            "Jazz",
            "Metal",
            "New Age",
            "Oldies",
            "Other",
            "Pop",
            "R&B",
            "Rap",
            "Reggae",
            "Rock",
            "Techno",
            "Industrial",
            "Alternative",
            "Ska",
            "Death Metal",
            "Pranks",
            "Soundtrack",
            "Euro-Techno",
            "Ambient",
            "Trip-Hop",
            "Vocal",
            "Jazz+Funk",
            "Fusion",
            "Trance",
            "Classical",
            "Instrumental",
            "Acid",
            "House",
            "Game",
            "Sound Clip",
            "Gospel",
            "Noise",
            "AlternRock",
            "Bass",
            "Soul",
            "Punk",
            "Space",
            "Meditative",
            "Instrumental Pop",
            "Instrumental Rock",
            "Ethnic",
            "Gothic",
            "Darkwave",
            "Techno-Industrial",
            "Electronic",
            "Pop-Folk",
            "Eurodance",
            "Dream",
            "Southern Rock",
            "Comedy",
            "Cult",
            "Gangsta",
            "Top 40",
            "Christian Rap",
            "Pop/Funk",
            "Jungle",
            "Native American",
            "Cabaret",
            "New Wave",
            "Psychedelic",
            "Rave",
            "Showtunes",
            "Trailer",
            "Lo-Fi",
            "Tribal",
            "Acid Punk",
            "Acid Jazz",
            "Polka",
            "Retro",
            "Musical",
            "Rock & Roll",
            "Hard Rock",
        ][g as usize]
            .to_string();
    }
}

pub fn has_id3v1<T: Read + Seek>(input: &mut T) -> bool {
    let mut arr: [u8; 3] = [0; 3];
    if let Err(_) = input.seek(std::io::SeekFrom::End(-128)) {
        return false;
    }
    if let Err(_) = input.read(&mut arr) {
        return false;
    }
    &arr == b"TAG"
}

pub fn get<T: Read + Seek>(input: &mut T) -> Result<Tags, Error> {
    input.seek(std::io::SeekFrom::End(-128))?;
    let mut arr: [u8; 128] = [0; 128];
    input.read(&mut arr)?;

    if &arr[0..3] == b"TAG" {
        Ok(Tags {
            title: TagOption::Some(decode_iso_8859_1(&arr[3..30 + 3])),
            artist: TagOption::Some(decode_iso_8859_1(&arr[33..33 + 30])),
            album: TagOption::Some(decode_iso_8859_1(&arr[63..63 + 30])),
            date: DateTime::from_iso_8601(decode_iso_8859_1(&arr[93..93 + 4]).as_str()).into(),
            comment: TagOption::Some(decode_iso_8859_1(&arr[97..97 + 28])),
            track_number: match arr[125] {
                0 => TagOption::Some(arr[126] as i64),
                _ => TagOption::None,
            },
            genre: TagOption::Some(get_genre(arr[127])),
            ..Default::default()
        })
    } else {
        Err(tag_error("ID3 tags not found"))
    }
}

#[cfg(test)]
mod tests;
