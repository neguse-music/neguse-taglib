pub fn undo_unsynch(vec: &mut Vec<u8>) {
    let mut i = 0;
    // undo synchronization
    while i < (vec.len() - 1) {
        if vec[i] == 0xFF && vec[i + 1] == 0x00 {
            vec.remove(i + 1);
        }
        i += 1;
    }
}
pub fn decode_synch_int(input: &[u8]) -> Result<u32, String> {
    if input.len() > 5 {
        return Err(format!("Synchsafe ints are limited to 32 bits"));
    }
    let mut result: u32 = 0;
    for (i, b) in input.iter().enumerate() {
        // verify that this is a valid synchsafe int
        // (by checking that the msb of each byte is zero)
        if b & 0x80 != 0 {
            return Err(format!("Invalid synch-safe byte at position {}", i));
        }
        // if so, transform to proper uint by
        // moving the 7 bit parts to proper places
        // (0000 0001 0111 1111 => 1111 1111)
        result |= (*b as u32) << (7 * (input.len() - 1 - i));
    }
    Ok(result)
}

pub fn encode_synch_int(input: u32, use_fifth_bit: bool) -> Result<Vec<u8>, String> {
    // request >28 bit explicitly
    if input >= 0xF0000000 && !use_fifth_bit {
        Err("Input uses more than 28 bits, but use fifth bit option is not enabled.".to_string())
    } else {
        let mut result = Vec::new();

        if use_fifth_bit {
            result.push((input >> 28) as u8);
        }
        for i in 0..4 {
            let mut r = input & (0x0FE00000 >> (7 * i));
            r = r >> (7 * (3 - i));

            result.push(r as u8);
        }
        Ok(result)
    }
}

pub fn decode_frame_id(input: &[u8]) -> Result<String, String> {
    let mut s = String::new();
    for c in input.iter() {
        if (*c >= b'A' && *c <= b'Z') || (*c >= b'0' && *c <= b'9') {
            s.push(*c as char);
        } else {
            return Err(format!("Cannot decode {:X?}: Invalid frame ID (contains characters that are not A-Z or 0-9)", input));
        }
    }
    Ok(s)
}
pub fn encode_frame_id(input: &str) -> Result<Vec<u8>, String> {
    let mut v = Vec::new();
    for c in input.chars() {
        let c = c as u8;
        if (c >= b'A' && c <= b'Z') || (c >= b'0' && c <= b'9') {
            v.push(c as u8);
        } else {
            return Err(format!("Cannot encode \"{}\": Invalid frame ID (contains characters that are not A-Z or 0-9)", input));
        }
    }
    Ok(v)
}
