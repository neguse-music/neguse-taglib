extern crate encoding;
use self::encoding::{DecoderTrap, EncoderTrap, Encoding};

pub fn encode_iso_8859_1(input: &str) -> Vec<u8> {
    use self::encoding::all::ISO_8859_1;
    ISO_8859_1
        .encode(input, EncoderTrap::Replace)
        .unwrap_or(Vec::new())
}

pub fn decode_iso_8859_1(input: &[u8]) -> String {
    use self::encoding::all::ISO_8859_1;
    ISO_8859_1
        .decode(input, DecoderTrap::Replace)
        .unwrap_or("".to_string())
        .trim_end_matches('\0')
        .to_string()
}

pub fn decode_utf8(input: &[u8]) -> String {
    use self::encoding::all::UTF_8;
    UTF_8
        .decode(input, DecoderTrap::Replace)
        .unwrap_or("".to_string())
        .trim_end_matches('\0')
        .to_string()
}

pub fn decode_utf16(input: &[u8]) -> String {
    use self::encoding::all::{UTF_16BE, UTF_16LE};
    match &input[0..2] {
        [0xFF, 0xFE] => UTF_16LE.decode(&input[2..], DecoderTrap::Replace),
        [0xFE, 0xFF] => UTF_16BE.decode(&input[2..], DecoderTrap::Replace),
        // in case of no BOM, assume big endian
        _ => UTF_16BE.decode(input, DecoderTrap::Replace),
    }
    .unwrap_or("".to_string())
    .trim_end_matches('\0')
    .to_string()
}
