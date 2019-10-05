#[derive(PartialEq, Clone)]
pub enum Image {
    PNG(Vec<u8>),
    JPEG(Vec<u8>),
    None,
}

use crc::crc32::checksum_ieee;

impl Image {
    pub fn mime(&self) -> String {
        match self {
            Image::PNG(_) => "image/png".to_string(),
            Image::JPEG(_) => "image/jpeg".to_string(),
            Image::None => "".to_string(),
        }
    }
    pub fn is_some(&self) -> bool {
        match self {
            Image::None => false,
            _ => true
        }
    }
    pub fn is_none(&self) -> bool { !self.is_some() }

    pub fn unwrap(self) -> Vec<u8> {
        match self {
            Image::PNG(v) => v,
            Image::JPEG(v) => v,
            Image::None => panic!("Attempted to unwrap a non-existent image"),
        }
    }
    pub fn crc32(&self) -> Option<u32> {
        match self {
            Image::JPEG(x) | Image::PNG(x) => Some(checksum_ieee(x)),
            Image::None => None,
        }
    }
                                      // w    h   bpp
    pub fn dimensions(&self) -> Option<(u32, u32, u8)> {
        match self {
            Image::JPEG(ref v) => {
                if v[0..2] != [0xFF, 0xD8] {
                    return None
                }

                let mut pos = 2;

                while pos+8 < v.len() {
                    // find SOFX
                    if v[pos] == 0xFF && v[pos+1] & 0xF0 == 0xC0 {
                        // [FF CX] [XX XX] [XX] [XX XX] [XX XX] 
                        // SOF id   size   bpp   width   height
                        return Some((
                                decode_int_be_u32(&v[pos+7..pos+9]),
                                decode_int_be_u32(&v[pos+5..pos+7]),
                                v[pos+4]
                            ))
                    }
                    pos += 2 + decode_int_be_u32(&v[pos+2..pos+4]) as usize;
                }

                None
            },
            Image::PNG(ref v) => {
                if v[0..8] != [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] || v.len() < 25 {
                    None
                } else {
                    Some((
                        decode_int_be_u32(&v[16..20]),
                        decode_int_be_u32(&v[20..24]),
                        v[24]
                    ))
                }
            },
            Image::None => None,
        }
    } 
}


use std::fmt;

extern crate crc;
impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = self.dimensions().unwrap_or((0, 0, 0));
        let t = match self {
            Image::PNG(x) => 
                format!("PNG 0x{:X?} ({} bytes, {}x{}, {}bpp)", 
                checksum_ieee(x), x.len(), res.0, res.1, res.2),
            Image::JPEG(x) => 
                format!("JPEG 0x{:X?} ({} bytes, {}x{}, {}bpp)", 
                checksum_ieee(x), x.len(), res.0, res.1, res.2),
            Image::None => format!("None"),
        };
        write!(f, "{}", t)
    }
}

pub fn decode_int_be_u32(input: &[u8]) -> u32 {
    if input.len() > 5 {
        panic!("decode_int_be_u32 expected a slice with max length 4, got slice with length {}", input.len());
    }
    let mut result: u32 = 0;
    for (i, b) in input.iter().enumerate() {
        // if so, transform to proper uint by
        // moving the 7 bit parts to proper places
        // (0000 0001 0111 1111 => 1111 1111)
        result |= (*b as u32) << (8 * (input.len() - 1 - i));
    }
    result
}
