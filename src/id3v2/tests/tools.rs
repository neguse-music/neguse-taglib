use crate::id3v2::tools::*;

#[test]
fn synch_int_test() {
    assert_eq!(decode_synch_int(&[0x7F, 0x7F, 0x7F, 0x7F]), Ok(0x0FFFFFFF));
    assert_eq!(decode_synch_int(&[0x01, 0x7F]), Ok(0xFF));
    assert_eq!(
        decode_synch_int(&[0x7F, 0x7F, 0x7F, 0x7F, 0x7F]),
        Ok(0xFFFFFFFF)
    );
    assert!(
        decode_synch_int(&[0xFF]).is_err(),
        "Not a valid synchsafe integer"
    );

    assert_eq!(
        encode_synch_int(0x0FFFFFFF, false),
        Ok(vec![0x7F, 0x7F, 0x7F, 0x7F])
    );
    assert_eq!(
        encode_synch_int(0xFF, false),
        Ok(vec![0x00, 0x00, 0x01, 0x7F])
    );
    assert!(
        encode_synch_int(0xFFFFFFFF, false).is_err(),
        "Not a 28 bit integer"
    );
    assert_eq!(
        encode_synch_int(0xFFFFFFFF, true),
        Ok(vec![0x0F, 0x7F, 0x7F, 0x7F, 0x7F])
    );

    assert_eq!(
        decode_synch_int(&encode_synch_int(0x80FF00, false).unwrap()),
        Ok(0x80FF00)
    );
    assert_eq!(
        decode_synch_int(&encode_synch_int(0x1D80FF00, true).unwrap()),
        Ok(0x1D80FF00)
    );
}

#[test]
fn frame_id_test() {
    assert_eq!(
        decode_frame_id(&[0x54, 0x49, 0x54, 0x32]),
        Ok("TIT2".to_string())
    );
    assert_eq!(
        decode_frame_id(&[0x54, 0x52, 0x43, 0x4B]),
        Ok("TRCK".to_string())
    );
    assert!(
        decode_frame_id(&[0x73, 0x52, 0x47, 0x42]).is_err(),
        "Need to have A-Z and 0-9 characters only"
    );

    assert_eq!(encode_frame_id("TIT2"), Ok(vec![0x54, 0x49, 0x54, 0x32]));
    assert_eq!(encode_frame_id("TRCK"), Ok(vec![0x54, 0x52, 0x43, 0x4B]));
    assert!(
        encode_frame_id("sRGB").is_err(),
        "Need to have A-Z and 0-9 characters only"
    );

    assert_eq!(
        decode_frame_id(&encode_frame_id("TEST1234").unwrap()),
        Ok("TEST1234".to_string())
    );
}
