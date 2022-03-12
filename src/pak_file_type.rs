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

pub static PAK_FILE_TYPE_UNKNOWN: PakFileType = PakFileType {
    ext_name: "",
    identifier: &[],
    compression: PakFileCompression::None
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
        identifier: &[0x1e, 0x9b],
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
