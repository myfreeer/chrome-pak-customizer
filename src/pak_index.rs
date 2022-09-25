use std::str::FromStr;

use ini_core::Item;

use crate::pak_def::{PakAlias, PakBase};
use crate::pak_error::PakError;
use crate::pak_header::{PAK_VERSION_V4, PAK_VERSION_V5, PakHeader, PakHeaderV4, PakHeaderV5};

pub enum PakIndexCompression {
    Raw,
    BrotliCompressed,
}

pub const PAK_INDEX_BROTLI_COMPRESSED: &str = ":::BrotliCompressed";
pub const PAK_INDEX_RAW: &str = ":::Raw";

impl PakIndexCompression {
    fn to_suffix(&self) -> &str {
        match self {
            PakIndexCompression::Raw => PAK_INDEX_RAW,
            PakIndexCompression::BrotliCompressed => PAK_INDEX_BROTLI_COMPRESSED
        }
    }

    fn strip_suffix(file_name: &mut String) {
        if file_name.ends_with(PAK_INDEX_RAW) {
            file_name.truncate(
                file_name.len() - PAK_INDEX_RAW.len())
        } else if file_name.ends_with(PAK_INDEX_BROTLI_COMPRESSED) {
            file_name.truncate(
                file_name.len() - PAK_INDEX_BROTLI_COMPRESSED.len())
        }
    }

    fn of_file_name(file_name: &str) -> PakIndexCompression {
        if file_name.ends_with(PAK_INDEX_BROTLI_COMPRESSED) {
            PakIndexCompression::BrotliCompressed
        } else {
            PakIndexCompression::Raw
        }
    }
}

pub struct PakIndexEntry {
    pub resource_id: u16,
    pub file_name: String,
    pub compression: PakIndexCompression
}

pub struct PakIndexRef<'a> {
    pub header: &'a dyn PakHeader,
    pub entry_slice: &'a [PakIndexEntry],
    pub alias_slice: &'a [PakAlias],
}

pub const PAK_INDEX_GLOBAL_TAG: &str = "Global";
pub const PAK_INDEX_RES_TAG: &str = "Resources";
pub const PAK_INDEX_ALIAS_TAG: &str = "Alias";
pub const PAK_INDEX_GLOBAL_VERSION: &str = "version";
pub const PAK_INDEX_GLOBAL_ENCODING: &str = "encoding";
pub const PAK_INDEX_TAG_END: &str = "]\r\n";
pub const PAK_INDEX_CRLF: &str = "\r\n";

// naive but much benchmarked to be faster in 2022.09
// modified from https://stackoverflow.com/a/1489873
#[inline]
fn number_digit_count_u16(x: u16) -> usize {
    if x >= 1000u16 {
        if x >= 10000u16 {
            return 5;
        }
        return 4;
    }
    if x >= 10u16 {
        if x >= 100u16 {
            return 3;
        }
        return 2;
    }
    return 1;
}

#[derive(Clone, Copy, Debug)]
pub enum PakIndexStatus {
    Init,
    Global,
    Resource,
    Alias,
}

impl PakIndexRef<'_> {
    fn calc_ini_byte_size(&self) -> usize {
        // 12: []\r\n * 2 + \r\n\r\n
        let mut buf_size: usize =
            PAK_INDEX_GLOBAL_TAG.len() + PAK_INDEX_RES_TAG.len() + 12;
        // 4: = \r\n + version number
        buf_size += PAK_INDEX_GLOBAL_VERSION.len() + 4;
        // 4: = \r\n + encoding number
        buf_size += PAK_INDEX_GLOBAL_ENCODING.len() + 4;
        if !self.alias_slice.is_empty() {
            // 8: \r\n\r\n + []\r\n
            buf_size += PAK_INDEX_ALIAS_TAG.len() + 8;
        }
        for entry in self.entry_slice {
            // 3: =\r\n
            buf_size += number_digit_count_u16(entry.resource_id) + 3;
            buf_size += entry.file_name.len();
            if matches!(entry.compression, PakIndexCompression::BrotliCompressed) {
                buf_size += PAK_INDEX_BROTLI_COMPRESSED.len();
            }
        }
        for alias in self.alias_slice {
            // 3: =\r\n
            buf_size += number_digit_count_u16(alias.resource_id) + 3;
            buf_size += number_digit_count_u16(alias.entry_index);
        }

        buf_size
    }

    pub fn to_ini_bytes(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::with_capacity(self.calc_ini_byte_size());
        // note: extend_from_slice is benchmarked to be faster in 2022.03
        // [Global]\r\n
        vec.push('[' as u8);
        vec.extend_from_slice(PAK_INDEX_GLOBAL_TAG.as_bytes());
        vec.extend_from_slice(PAK_INDEX_TAG_END.as_bytes());
        // version=?\r\n
        vec.extend_from_slice(PAK_INDEX_GLOBAL_VERSION.as_bytes());
        vec.push('=' as u8);
        vec.extend_from_slice(self.header.read_version().to_string().as_bytes());
        vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        // encoding=?\r\n
        vec.extend_from_slice(PAK_INDEX_GLOBAL_ENCODING.as_bytes());
        vec.push('=' as u8);
        vec.extend_from_slice(self.header.read_encoding().to_string().as_bytes());
        vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        // \r\n
        vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        // [Resources]\r\n
        vec.push('[' as u8);
        vec.extend_from_slice(PAK_INDEX_RES_TAG.as_bytes());
        vec.extend_from_slice(PAK_INDEX_TAG_END.as_bytes());
        // {resource_id}={file_name}\r\n
        for entry in self.entry_slice {
            vec.extend_from_slice(entry.resource_id.to_string().as_bytes());
            vec.push('=' as u8);
            vec.extend_from_slice(entry.file_name.as_bytes());
            if matches!(entry.compression, PakIndexCompression::BrotliCompressed) {
                vec.extend_from_slice(entry.compression.to_suffix().as_bytes());
            }
            vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        }

        if self.alias_slice.is_empty() {
            return vec;
        }
        // \r\n
        vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        // [Alias]\r\n
        vec.push('[' as u8);
        vec.extend_from_slice(PAK_INDEX_ALIAS_TAG.as_bytes());
        vec.extend_from_slice(PAK_INDEX_TAG_END.as_bytes());
        // {resource_id}={entry_index}\r\n
        for alias in self.alias_slice {
            vec.extend_from_slice(alias.read_resource_id().to_string().as_bytes());
            vec.push('=' as u8);
            vec.extend_from_slice(alias.read_entry_index().to_string().as_bytes());
            vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        }
        vec
    }
}

pub struct PakIndex {
    pub header: Box<dyn PakHeader>,
    pub entry_vec: Vec<PakIndexEntry>,
    pub alias_vec: Vec<PakAlias>,
}

impl PakIndex {
    #[inline]
    #[allow(dead_code)]
    pub fn as_pak_index_ref(&self) -> PakIndexRef {
        PakIndexRef {
            header: self.header.as_ref(),
            entry_slice: &self.entry_vec,
            alias_slice: &self.alias_vec,
        }
    }

    pub fn from_ini_buf(buf: &[u8]) -> Result<PakIndex, PakError> {
        // SAFETY: ini_core only uses as_bytes internally, the utf8 format has no effect
        let str: &str = unsafe { std::str::from_utf8_unchecked(buf) };
        let parser = ini_core::Parser::new(str);
        let mut status = PakIndexStatus::Init;
        let mut entry_vec: Vec<PakIndexEntry> = Vec::new();
        let mut alias_vec: Vec<PakAlias> = Vec::new();
        let mut version: u32 = 0;
        let mut encoding: u8 = 0;

        // parsing
        for item in parser {
            match item {
                Item::Error(err) => {
                    return Err(PakError::PakIndexParseError(String::from(err)));
                }
                Item::Section(section) => match section {
                    PAK_INDEX_GLOBAL_TAG => {
                        status = PakIndexStatus::Global;
                    }
                    PAK_INDEX_RES_TAG => {
                        status = PakIndexStatus::Resource;
                    }
                    PAK_INDEX_ALIAS_TAG => {
                        status = PakIndexStatus::Alias;
                    }
                    other => {
                        return Err(PakError::PakIndexUnknownTag(String::from(other)));
                    }
                },
                Item::Property(key, value) => match status {
                    PakIndexStatus::Init => {
                        return Err(PakError::PakIndexUnknownProperty(
                            status,
                            String::from(key),
                            String::from(value),
                        ));
                    }
                    PakIndexStatus::Global => match key {
                        PAK_INDEX_GLOBAL_VERSION => match u32::from_str(value) {
                            Ok(value) => {
                                if value == PAK_VERSION_V4 || value == PAK_VERSION_V5 {
                                    version = value;
                                } else {
                                    return Err(PakError::UnsupportedVersion(value));
                                }
                            }
                            Err(err) => {
                                return Err(PakError::PakIndexBadVersion(
                                    String::from(value),
                                    err,
                                ));
                            }
                        },
                        PAK_INDEX_GLOBAL_ENCODING => match u8::from_str(value) {
                            Ok(value) => {
                                encoding = value;
                            }
                            Err(err) => {
                                return Err(PakError::PakIndexBadEncoding(
                                    String::from(value),
                                    err,
                                ));
                            }
                        }
                        _ => {
                            return Err(PakError::PakIndexUnknownProperty(
                                status,
                                String::from(key),
                                String::from(value),
                            ));
                        }
                    },
                    PakIndexStatus::Resource => {
                        let resource_id = match u16::from_str(key) {
                            Ok(num) => num,
                            Err(err) => {
                                return Err(PakError::PakIndexBadResourceId(
                                    String::from(key), err));
                            }
                        };
                        let mut file_name: String = String::from(value);
                        let compression =
                            PakIndexCompression::of_file_name(&file_name);
                        PakIndexCompression::strip_suffix(&mut file_name);
                        entry_vec.push(PakIndexEntry {
                            resource_id,
                            file_name,
                            compression
                        });
                    }
                    PakIndexStatus::Alias => {
                        if version == PAK_VERSION_V4 {
                            return Err(PakError::PakIndexAliasNotSupported(version));
                        }
                        let resource_id = match u16::from_str(key) {
                            Ok(num) => num,
                            Err(err) => {
                                return Err(PakError::PakIndexAliasBadResourceId(
                                    String::from(key),
                                    String::from(value), err));
                            }
                        };
                        let entry_index = match u16::from_str(value) {
                            Ok(num) => num,
                            Err(err) => {
                                return Err(PakError::PakIndexAliasBadEntryIndex(
                                    String::from(key),
                                    String::from(value), err));
                            }
                        };
                        alias_vec.push(PakAlias { resource_id, entry_index });
                    }
                }
                Item::Action(action) => {
                    return Err(PakError::PakIndexUnknownAction(
                        status, String::from(action)));
                }
                // ignore this?
                Item::Comment(_) => {}
                Item::Blank => {}
            }
        }

        let mut header: Box<dyn PakHeader> = if version == PAK_VERSION_V5 {
            Box::new(PakHeaderV5::new())
        } else {
            // must be 4 here
            Box::new(PakHeaderV4::new())
        };
        entry_vec.shrink_to_fit();
        header.write_encoding(encoding);
        header.write_resource_count(entry_vec.len() as u32);
        if alias_vec.len() > 0 {
            header.write_alias_count(alias_vec.len() as u16);
        }
        Ok(PakIndex {
            header,
            entry_vec,
            alias_vec,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_digit_count_u16_test() {
        for i in 0..u16::MAX {
            let digits = number_digit_count_u16(i);
            assert_eq!(i.to_string().len(), digits);
        }
    }

}