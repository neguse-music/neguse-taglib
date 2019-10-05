use crate::Image;
use std;
use std::fs::File;
use std::io::prelude::*;

#[test]
fn picture_block_encode() {
    //jpeg
    let image = {
        let mut vec = Vec::new();
        let mut file = File::open("testfiles/flac-cover.jpg").unwrap();
        file.read_to_end(&mut vec).unwrap();
        Image::JPEG(vec)
    };
    let encoded = super::get_picture_block(&image).unwrap();
    let (image2, _) =
        super::read_image(&mut std::io::Cursor::new(&encoded), encoded.len() as u32).unwrap();

    assert_eq!(image, image2);

    //png
    let image = {
        let mut vec = Vec::new();
        let mut file = File::open("testfiles/id3v24-utf8-png-cover.png").unwrap();
        file.read_to_end(&mut vec).unwrap();
        Image::PNG(vec)
    };
    let encoded = super::get_picture_block(&image).unwrap();
    let (image2, _) =
        super::read_image(&mut std::io::Cursor::new(&encoded), encoded.len() as u32).unwrap();

    assert_eq!(image, image2);
}
