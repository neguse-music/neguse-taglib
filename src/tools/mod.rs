pub mod encoding;
pub mod tags;

use crate::Error;
pub fn tag_error(err: &str) -> Error {
    Error::TagError(err.to_string())
}

// is there a way to enforce slice length at compile time?
pub fn decode_int_be_u32(input: &[u8]) -> u32 {
    if input.len() > 5 {
        panic!(
            "decode_int_be_u32 expected a slice with max length 4, got slice with length {}",
            input.len()
        );
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
pub fn decode_int_le_u32(input: &[u8]) -> u32 {
    if input.len() > 5 {
        panic!(
            "decode_int_le_u32 expected a slice with max length 4, got slice with length {}",
            input.len()
        );
    }
    let mut result: u32 = 0;
    for (i, b) in input.iter().rev().enumerate() {
        // if so, transform to proper uint by
        // moving the 7 bit parts to proper places
        // (0000 0001 0111 1111 => 1111 1111)
        result |= (*b as u32) << (8 * (input.len() - 1 - i));
    }
    result
}
pub fn encode_int_be_u16(input: u16) -> Vec<u8> {
    let mut result = vec![0; 2];
    result[0] = (input >> 8) as u8;
    result[1] = input as u8;
    result
}
pub fn encode_int_be_u32(input: u32) -> Vec<u8> {
    let mut result = vec![0; 4];
    for i in 0..4 {
        result[i] = ((input & 0xFF000000 >> 8 * i) >> 8 * (3 - i)) as u8;
    }
    result
}
pub fn encode_int_le_u32(input: u32) -> Vec<u8> {
    let mut result = vec![0; 4];
    for i in 0..4 {
        result[i] = ((input & 0x000000FF << 8 * i) >> 8 * i) as u8;
    }
    result
}

#[cfg(test)]
mod tests;
