#[derive(Debug, Default)]
pub struct Header {
    // footer is essentially the same as the header
    pub version: u8,
    //pub revision: u8,
    pub size: u32, // in bytes, goes up to 256 mb

    pub is_unsynchronized: bool,
    pub is_experimental: bool,
    pub has_footer: bool,

    pub extended_header: Option<ExtendedHeader>,
}

#[derive(Debug)]
pub struct ExtendedHeader {
    pub size: u32,
    pub tag_is_update: bool,
    pub crc32: Option<u32>,
    pub restrictions: Option<u8>,
}

#[derive(Debug, Default)]
pub struct FrameHeader {
    pub name: String,

    // flags
    pub drop_after_tag_alteration: bool,
    pub drop_after_file_alteration: bool,
    pub is_unsynchronized: bool,
    pub is_compressed: bool,

    pub size: u32,
    //pub offset: u32,
}
