use crate::pak_error::PakError;

pub enum PakFileCompression {
    None,
    Gzip,
    ChromiumBrotli,
}

pub struct PakFileType {
    pub ext_name: &'static str,
    pub identifier: &'static [u8],
    pub compression: PakFileCompression,
}

impl PakFileType {
    fn starts_with(&self, buf: &[u8]) -> bool {
        buf.starts_with(self.identifier)
    }
}

pub const BROTLI_HEADER_SIZE: usize = 8;
pub const BROTLI_CONST: [u8; 2] = [0x1e, 0x9b];

pub static PAK_FILE_TYPE_UNKNOWN: PakFileType = PakFileType {
    ext_name: "",
    identifier: &[],
    compression: PakFileCompression::None,
};

// maybe trie could be faster?
// https://en.wikipedia.org/wiki/Magic_number_(programming)
pub static PAK_FILE_TYPES: [PakFileType; 17] = [
    PakFileType {
        ext_name: ".png",
        identifier: &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".html",
        identifier: "<!doctype html>".as_bytes(),
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".html",
        identifier: "<!DOCTYPE html>".as_bytes(),
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".html",
        identifier: "<html>".as_bytes(),
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".html",
        identifier: "<!--".as_bytes(),
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".html",
        identifier: "<link ".as_bytes(),
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".svg",
        identifier: "<svg ".as_bytes(),
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".js",
        identifier: "// ".as_bytes(),
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".js",
        identifier: "(function".as_bytes(),
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".css",
        identifier: "/*".as_bytes(),
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".gz",
        identifier: &[0x1f, 0x8b],
        compression: PakFileCompression::Gzip,
    },
    PakFileType {
        ext_name: ".br",
        identifier: &BROTLI_CONST,
        compression: PakFileCompression::ChromiumBrotli,
    },
    PakFileType {
        ext_name: ".gif",
        identifier: "GIF89a".as_bytes(),
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".gif",
        identifier: "GIF87a".as_bytes(),
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".png",
        identifier: &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".jpg",
        identifier: &[0xff, 0xd8],
        compression: PakFileCompression::None,
    },
    PakFileType {
        ext_name: ".json",
        identifier: "{".as_bytes(),
        compression: PakFileCompression::None,
    },
];

pub fn pak_get_file_type(buf: &[u8]) -> &PakFileType {
    for x in &PAK_FILE_TYPES {
        if x.starts_with(buf) {
            return x;
        }
    }
    &PAK_FILE_TYPE_UNKNOWN
}

pub fn pak_get_chromium_brotli_decompressed_size(buf: &[u8]) -> Result<u64, PakError> {
    if !buf.starts_with(&BROTLI_CONST) {
        return Err(PakError::PakNotChromiumBrotli);
    }
    if buf.len() < BROTLI_HEADER_SIZE {
        return Err(PakError::PakChromiumBrotliSizeNotEnough(buf.len()));
    }
    // Get size of uncompressed resource from header.
    let mut uncompress_size: u64 = 0;
    let mut raw_input_offset = BROTLI_CONST.len();
    let byte_size = BROTLI_HEADER_SIZE - BROTLI_CONST.len();
    for i in 0..byte_size {
        uncompress_size |= (buf[raw_input_offset + i] as u64) << (i * 8);
    }
    Ok(uncompress_size)
}
